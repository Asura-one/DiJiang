pub fn cmd_update(force: bool, from_github: bool) -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    let _dijiang_dir = crate::util::resolve_dijiang_dir(&cwd)
        .ok_or_else(|| anyhow::anyhow!("未找到 .dijiang/ 目录。请先运行 `dijiang init`。"))?;

    if from_github {
        println!("  正在从 GitHub 下载最新版本...");
        let temp_dir = std::env::temp_dir().join("dijiang-update");
        if temp_dir.exists() {
            std::fs::remove_dir_all(&temp_dir)?;
        }

        let output = std::process::Command::new("git")
            .args(["clone", "--depth", "1", "https://github.com/Asura-one/DiJiang.git", temp_dir.to_str().unwrap()])
            .output()?;

        if !output.status.success() {
            anyhow::bail!("从 GitHub 下载失败: {}", String::from_utf8_lossy(&output.stderr));
        }

        println!("  下载完成，正在更新全局技能...");
        let global_dir = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("无法获取用户主目录"))?
            .join(".dijiang")
            .join("skills");
        std::fs::create_dir_all(&global_dir)?;

        let src_skills = temp_dir
            .join("crates").join("configurator").join("templates").join("skills");
        if src_skills.exists() {
            for entry in std::fs::read_dir(&src_skills)? {
                let entry = entry?;
                if entry.file_type()?.is_dir() {
                    let name = entry.file_name();
                    let src = entry.path().join("SKILL.md");
                    let dst = global_dir.join(&name).join("SKILL.md");
                    if src.exists() {
                        std::fs::create_dir_all(dst.parent().unwrap())?;
                        std::fs::copy(&src, &dst)?;
                        println!("  已更新全局技能: {}", name.to_string_lossy());
                    }
                }
            }
        }

        let _ = std::fs::remove_dir_all(&temp_dir);
        println!("  GitHub 更新完成。\n");
    }

    let report = dijiang_configurator::update_project(&cwd, dijiang_configurator::UpdateOptions { force })?;

    let old_version = report.old_version.as_deref().unwrap_or("unknown");
    let version_changed = report.old_version.as_deref() != Some(&report.new_version);

    if version_changed {
        println!("  DiJiang 版本: {old_version} → {}", report.new_version);
    } else {
        println!("  DiJiang 版本: {} · 当前已是最新", report.new_version);
    }
    println!();

    // 变更日志
    if version_changed {
        let changelog = dijiang_configurator::changelog_between(old_version, &report.new_version);
        if !changelog.is_empty() {
            println!("  ── 变更日志 {} ──", report.new_version);
            for line in changelog.lines() {
                println!("  {line}");
            }
            println!();
        }
    }

    // 只显示有变动的文件，略过未变更的
    if !report.updated.is_empty() {
        println!("  ── 更新 ({} 文件) ──", report.updated.len());
        for path in &report.updated {
            println!("    {path}");
        }
        println!();
    }

    if !report.removed.is_empty() {
        println!("  ── 删除 ({} 文件) ──", report.removed.len());
        for path in &report.removed {
            println!("    {path}");
        }
        println!();
    }

    if !report.conflicts.is_empty() {
        println!("  ── 冲突 ({} 文件) ──", report.conflicts.len());
        for path in &report.conflicts {
            println!("    {path}");
        }
        println!();
    }

    if !report.warnings.is_empty() {
        println!("  ── 警告 ({} 条) ──", report.warnings.len());
        for warning in &report.warnings {
            println!("    {warning}");
        }
        println!();
    }

    // 概要
    let has_changes = !report.updated.is_empty()
        || !report.removed.is_empty()
        || !report.conflicts.is_empty()
        || !report.warnings.is_empty();

    if has_changes {
        let mut summary = Vec::new();
        if !report.updated.is_empty() {
            summary.push(format!("{} 更新", report.updated.len()));
        }
        if !report.removed.is_empty() {
            summary.push(format!("{} 删除", report.removed.len()));
        }
        if !report.conflicts.is_empty() {
            summary.push(format!("{} 冲突", report.conflicts.len()));
        }
        if !report.warnings.is_empty() {
            summary.push(format!("{} 警告", report.warnings.len()));
        }
        println!("  更新完成: {} ({} 个已是最新)",
            summary.join(", "),
            report.unchanged.len());
    } else {
        println!("  所有文件已是最新 ({} 个文件)", report.unchanged.len());
    }

    if !report.is_clean() {
        anyhow::bail!(
            "update blocked: {} 个受管文件可能包含用户修改，未覆盖。确认后可使用 `dijiang update --force` 覆盖并建立后续升级 hash。",
            report.conflicts.len()
        );
    }

    Ok(())
}
