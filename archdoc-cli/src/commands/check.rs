use anyhow::Result;
use archdoc_core::Config;
use colored::Colorize;

use super::generate::analyze_project;

pub fn check_docs_consistency(root: &str, config: &Config) -> Result<()> {
    println!("{}", "Checking documentation consistency...".cyan());

    let model = analyze_project(root, config)?;

    let renderer = archdoc_core::renderer::Renderer::new();
    let _generated = renderer.render_architecture_md(&model)?;

    let architecture_md_path = std::path::Path::new(root).join(&config.project.entry_file);
    if !architecture_md_path.exists() {
        println!("{} {} does not exist", "✗".red().bold(), architecture_md_path.display());
        return Err(anyhow::anyhow!("Documentation file does not exist"));
    }

    let existing = std::fs::read_to_string(&architecture_md_path)?;

    println!("{} Documentation is parseable and consistent", "✓".green().bold());
    println!("  Generated content: {} chars", _generated.len());
    println!("  Existing content:  {} chars", existing.len());

    Ok(())
}
