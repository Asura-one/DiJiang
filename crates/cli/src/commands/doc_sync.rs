use crate::util::require_dijiang_dir;

pub fn cmd_doc_sync_check(base: String) -> anyhow::Result<()> {
    let project_root = std::env::current_dir()?;

    let report = dijiang_task::doc_sync::analyzer::DiffAnalyzer::analyze(&project_root, &base)
        .map_err(|e| anyhow::anyhow!(e))?;
    let impacts = dijiang_task::doc_sync::mapper::map_changes_to_docs(&report);
    let output = dijiang_task::doc_sync::format_report(&report, &impacts);
    print!("{output}");

    Ok(())
}
