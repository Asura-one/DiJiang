use crate::util::require_dijiang_dir;

pub fn cmd_spec_sync_check() -> anyhow::Result<()> {
    let dijiang_dir = require_dijiang_dir()?;
    let diff = dijiang_task::spec_sync::check_spec_changes(&dijiang_dir)?;
    if !diff.has_changes() {
        println!("  所有 spec 文件与已记录 checksum 一致，无变化。");
        return Ok(());
    }
    if !diff.new.is_empty() {
        println!(" 新增 specs:");
        for p in &diff.new { println!("    + {p}"); }
    }
    if !diff.changed.is_empty() {
        println!(" 已更改 specs:");
        for p in &diff.changed { println!("    ~ {p}"); }
    }
    if !diff.deleted.is_empty() {
        println!(" 已删除 specs:");
        for p in &diff.deleted { println!("    - {p}"); }
    }
    println!();
    println!("  提示: 运行 `dijiang spec-sync record` 更新 checksum 记录。");
    Ok(())
}

pub fn cmd_spec_sync_record() -> anyhow::Result<()> {
    let dijiang_dir = require_dijiang_dir()?;
    let checksums = dijiang_task::spec_sync::compute_spec_checksums(&dijiang_dir);
    let count = checksums.len();
    dijiang_task::spec_sync::write_stored_checksums(&dijiang_dir, &checksums)?;
    println!("  已记录 {count} 个 spec 文件的 checksums。");
    Ok(())
}
