pub fn cmd_template_list() -> anyhow::Result<()> {
    let registry = dijiang_configurator::TemplateRegistry::new();
    let builtins = registry.list_builtin();
    println!("\n  ── Available Templates ──\n");
    println!("  Built-in:");
    if builtins.is_empty() { println!("    (none)"); }
    else { for name in &builtins { println!("    • {name}"); } }
    let cached = registry.list_local().unwrap_or_default();
    println!("\n  Cached ({}):", cached.len());
    if cached.is_empty() {
        println!("    (none — use `dijiang template pull <source>` to add templates)");
    } else {
        for pkg in &cached {
            println!("    • {} v{} — {}",
                pkg.manifest.template.name, pkg.manifest.template.version,
                pkg.manifest.template.description);
        }
    }
    println!();
    Ok(())
}

pub fn cmd_template_pull(source: &str) -> anyhow::Result<()> {
    let registry = dijiang_configurator::TemplateRegistry::new();
    match registry.resolve(source) {
        Ok(pkg) => {
            println!("✓ Pulled template '{}' v{} to cache", pkg.manifest.template.name, pkg.manifest.template.version);
            println!("  Location: {}", pkg.root.display());
            println!("  Files: {}", pkg.manifest.files.len());
            Ok(())
        }
        Err(e) => {
            eprintln!("Error pulling template: {e}");
            std::process::exit(1);
        }
    }
}

pub fn cmd_template_validate(path: &str) -> anyhow::Result<()> {
    let template_path = std::path::Path::new(path);
    match dijiang_configurator::TemplateRegistry::validate(template_path) {
        Ok(manifest) => {
            println!("✓ Template '{}' v{} is valid", manifest.template.name, manifest.template.version);
            println!("  Description: {}", manifest.template.description);
            println!("  Files: {}", manifest.files.len());
            if let Some(meta) = &manifest.metadata {
                if let Some(author) = &meta.author {
                    println!("  Author: {author}");
                }
            }
            Ok(())
        }
        Err(errors) => {
            for err in &errors { eprintln!("  ✗ {err}"); }
            std::process::exit(1);
        }
    }
}
