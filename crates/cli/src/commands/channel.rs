use crate::util::require_dijiang_dir;
use dijiang_task::store;
use std::sync::Mutex;
use std::thread;

pub fn cmd_channel_spawn(agent: &str, task: Option<&str>, dir: Option<&str>) -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    let base_dir = dir.map(std::path::PathBuf::from).unwrap_or(cwd);
    let dijiang_dir = {
        let d = store::find_dijiang_dir(&base_dir)
            .ok_or_else(|| anyhow::anyhow!("No .dijiang/ found. Run `dijiang init` first."))?;
        d
    };
    let agents_dir = dijiang_dir.parent().map(|p| p.join(".pi").join("agents")).unwrap_or_default();
    let agent_file = agents_dir.join(format!("dijiang-{}.md", agent));
    if !agent_file.exists() { anyhow::bail!("Agent '{}' not found at {}", agent, agent_file.display()); }
    let agent_def = std::fs::read_to_string(&agent_file)?;
    let channel_id = format!("{}-{}", agent, std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?.as_secs());
    let channel_dir = dijiang_dir.join("channels").join(&channel_id);
    std::fs::create_dir_all(&channel_dir)?;
    std::fs::write(channel_dir.join("agent.md"), &agent_def)?;
    let inbox_content = match task { Some(t) => format!("当前任务: {}\n", t), None => format!("当前任务: {}\n", base_dir.display()) };
    std::fs::write(channel_dir.join("inbox"), &inbox_content)?;
    let timestamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    let metadata = format!(
        "id = {:?}\nagent = {:?}\nstatus = \"active\"\ncreated = {:?}\n\"task\" = {:?}\n\"dir\" = {:?}\n",
        channel_id, agent, timestamp, task.unwrap_or(""), base_dir.display()
    );
    std::fs::write(channel_dir.join("channel.toml"), &metadata)?;
    println!("  Agent '{}' spawned", agent);
    println!("  Channel ID: {}", channel_id);
    println!("  Channel dir: {}", channel_dir.display());
    println!("  To execute, run: dijiang channel execute {}", channel_id);
    Ok(())
}

pub fn cmd_channel_list() -> anyhow::Result<()> {
    let dijiang_dir = require_dijiang_dir()?;
    let channels_dir = dijiang_dir.join("channels");
    if !channels_dir.exists() { println!("  No channels found."); return Ok(()); }
    let mut channels = Vec::new();
    for entry in std::fs::read_dir(&channels_dir)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            let channel_id = entry.file_name().to_string_lossy().to_string();
            let channel_toml = entry.path().join("channel.toml");
            if channel_toml.exists() {
                let content = std::fs::read_to_string(&channel_toml)?;
                let agent = content.lines().find(|l| l.contains("agent"))
                    .and_then(|l| l.split('=').nth(1)).map(|s| s.trim().trim_matches('"').to_string())
                    .unwrap_or_else(|| "unknown".to_string());
                let status = content.lines().find(|l| l.contains("status"))
                    .and_then(|l| l.split('=').nth(1)).map(|s| s.trim().trim_matches('"').to_string())
                    .unwrap_or_else(|| "unknown".to_string());
                channels.push((channel_id, agent, status));
            }
        }
    }
    if channels.is_empty() { println!("  未找到通道。"); }
    else {
        println!("  {} 个活跃通道:", channels.len());
        for (id, agent, status) in &channels { println!("  {} - {} ({})", id, agent, status); }
    }
    Ok(())
}

pub fn cmd_channel_send(channel_id: &str, message: &str) -> anyhow::Result<()> {
    let dijiang_dir = require_dijiang_dir()?;
    let channel_dir = dijiang_dir.join("channels").join(channel_id);
    if !channel_dir.exists() { anyhow::bail!("Channel '{}' not found", channel_id); }
    let inbox_path = channel_dir.join("inbox");
    let mut inbox = std::fs::read_to_string(&inbox_path).unwrap_or_default();
    inbox.push_str(message); inbox.push('\n');
    std::fs::write(&inbox_path, &inbox)?;
    println!("  Message sent to channel {}", channel_id);
    Ok(())
}

pub fn cmd_channel_status(channel_id: &str) -> anyhow::Result<()> {
    let dijiang_dir = require_dijiang_dir()?;
    if channel_id == "all" { return cmd_channel_list(); }
    let channel_dir = dijiang_dir.join("channels").join(channel_id);
    if !channel_dir.exists() { anyhow::bail!("Channel '{}' not found", channel_id); }
    let channel_toml = channel_dir.join("channel.toml");
    if channel_toml.exists() {
        let content = std::fs::read_to_string(&channel_toml)?;
        println!("  Channel {}:", channel_id);
        for line in content.lines() { if !line.trim().is_empty() { println!("    {}", line); } }
    } else { println!("  Channel {}:\n    No metadata found.", channel_id); }
    let inbox_path = channel_dir.join("inbox");
    if inbox_path.exists() { println!("    inbox: {} bytes", std::fs::read_to_string(&inbox_path)?.len()); }
    let output_path = channel_dir.join("output");
    if output_path.exists() { println!("    output: {} bytes", std::fs::read_to_string(&output_path)?.len()); }
    Ok(())
}

pub fn cmd_channel_stop(channel_id: &str) -> anyhow::Result<()> {
    let dijiang_dir = require_dijiang_dir()?;
    let channel_dir = dijiang_dir.join("channels").join(channel_id);
    if !channel_dir.exists() { anyhow::bail!("Channel '{}' not found", channel_id); }
    let channel_toml = channel_dir.join("channel.toml");
    if channel_toml.exists() {
        let content = std::fs::read_to_string(&channel_toml)?;
        std::fs::write(&channel_toml, content.replace("status = \"active\"", "status = \"stopped\""))?;
    }
    println!("  通道 {} 已停止。", channel_id);
    Ok(())
}

fn cmd_channel_execute_single(channel_id: &str, model: Option<&str>, provider: Option<&str>, timeout: u64, cwd: &std::path::Path, dijiang_dir: &std::path::Path) -> anyhow::Result<()> {
    let channel_dir = dijiang_dir.join("channels").join(channel_id);
    if !channel_dir.exists() { anyhow::bail!("Channel '{}' not found", channel_id); }
    let agent_file = channel_dir.join("agent.md");
    if !agent_file.exists() { anyhow::bail!("No agent definition found in channel"); }
    let inbox_file = channel_dir.join("inbox");
    if !inbox_file.exists() { anyhow::bail!("No inbox found in channel"); }
    let mut pi_args = vec!["--print".to_string()];
    if let Some(m) = model { pi_args.push("--model".to_string()); pi_args.push(m.to_string()); }
    if let Some(p) = provider { pi_args.push("--provider".to_string()); pi_args.push(p.to_string()); }
    let agent_def = std::fs::read_to_string(&agent_file)?;
    let inbox_content = std::fs::read_to_string(&inbox_file)?;
    let prompt = format!("{}\n\n---\n\nInbox:\n{}", agent_def, inbox_content);
    let mut child = std::process::Command::new("pi").args(&pi_args)
        .stdin(std::process::Stdio::piped()).stdout(std::process::Stdio::piped()).stderr(std::process::Stdio::piped())
        .current_dir(cwd).spawn()?;
    if let Some(mut stdin) = child.stdin.take() { use std::io::Write; stdin.write_all(prompt.as_bytes())?; }
    let start = std::time::Instant::now();
    let output = loop {
        match child.try_wait() {
            Ok(Some(_status)) => break child.wait_with_output()?,
            Ok(None) => {
                if start.elapsed().as_secs() >= timeout { child.kill()?; child.wait()?; anyhow::bail!("Execution timed out after {}s", timeout); }
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
            Err(e) => anyhow::bail!("Error waiting for process: {}", e),
        }
    };
    let stdout = String::from_utf8_lossy(&output.stdout);
    let output_file = channel_dir.join("output");
    std::fs::write(&output_file, stdout.as_ref())?;
    let channel_toml = channel_dir.join("channel.toml");
    let status = if output.status.success() { "completed" } else { "failed" };
    if channel_toml.exists() {
        let content = std::fs::read_to_string(&channel_toml)?;
        std::fs::write(&channel_toml, content.replace("status = \"active\"", &format!("status = \"{}\"", status)))?;
    }
    Ok(())
}

pub fn cmd_channel_execute(channel_id: &str, model: Option<&str>, provider: Option<&str>, timeout: u64, follow: bool) -> anyhow::Result<()> {
    let dijiang_dir = require_dijiang_dir()?;
    let cwd = std::env::current_dir()?;
    cmd_channel_execute_single(channel_id, model, provider, timeout, &cwd, &dijiang_dir)?;
    Ok(())
}

pub fn cmd_channel_execute_all(model: Option<&str>, provider: Option<&str>, timeout: u64) -> anyhow::Result<()> {
    let dijiang_dir = require_dijiang_dir()?;
    let cwd = std::env::current_dir()?;
    let channels_dir = dijiang_dir.join("channels");
    if !channels_dir.exists() { println!("  No channels found."); return Ok(()); }
    let mut active_channels = Vec::new();
    for entry in std::fs::read_dir(&channels_dir)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            let channel_id = entry.file_name().to_string_lossy().to_string();
            let channel_toml = entry.path().join("channel.toml");
            if channel_toml.exists() {
                let content = std::fs::read_to_string(&channel_toml)?;
                if content.contains("status = \"active\"") { active_channels.push(channel_id); }
            }
        }
    }
    if active_channels.is_empty() { println!("  无活跃通道。"); return Ok(()); }
    println!("  Executing {} 个活跃通道:", active_channels.len());
    let handles: Vec<_> = active_channels.into_iter().map(|channel_id| {
        let cwd = cwd.clone();
        let dijiang_dir = dijiang_dir.clone();
        let model = model.map(|m| m.to_string());
        let provider = provider.map(|p| p.to_string());
        thread::spawn(move || {
            let result = cmd_channel_execute_single(&channel_id, model.as_deref(), provider.as_deref(), timeout, &cwd, &dijiang_dir);
            (channel_id, result.is_ok(), result.err().map(|e| e.to_string()).unwrap_or_default())
        })
    }).collect();
    let mut success_count = 0;
    let mut fail_count = 0;
    for handle in handles {
        let (channel_id, success, error) = handle.join().unwrap();
        if success { println!("  {} 已完成", channel_id); success_count += 1; }
        else { println!("  {} 失败: {}", channel_id, error); fail_count += 1; }
    }
    println!("\n  结果: {} 成功, {} 失败", success_count, fail_count);
    Ok(())
}
