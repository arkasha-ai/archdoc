use anyhow::Result;
use archdoc_core::{Config, ProjectModel, scanner::FileScanner, python_analyzer::PythonAnalyzer};
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::Path;

use crate::output::sanitize_filename;

pub fn load_config(config_path: &str) -> Result<Config> {
    Config::load_from_file(Path::new(config_path))
        .map_err(|e| anyhow::anyhow!("Failed to load config: {}", e))
}

pub fn analyze_project(root: &str, config: &Config) -> Result<ProjectModel> {
    println!("{}", "Scanning project...".cyan());

    let scanner = FileScanner::new(config.clone());
    let python_files = scanner.scan_python_files(std::path::Path::new(root))?;

    println!("  Found {} Python files", python_files.len().to_string().yellow());

    let analyzer = PythonAnalyzer::new(config.clone());

    let pb = ProgressBar::new(python_files.len() as u64);
    pb.set_style(ProgressStyle::default_bar()
        .template("  {spinner:.green} [{bar:30.cyan/dim}] {pos}/{len} {msg}")
        .unwrap_or_else(|_| ProgressStyle::default_bar())
        .progress_chars("█▓░"));

    let mut parsed_modules = Vec::new();
    let mut parse_errors = 0;
    for file_path in &python_files {
        pb.set_message(file_path.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default());
        match analyzer.parse_module(file_path) {
            Ok(module) => parsed_modules.push(module),
            Err(e) => {
                parse_errors += 1;
                pb.println(format!("  {} Failed to parse {}: {}", "⚠".yellow(), file_path.display(), e));
            }
        }
        pb.inc(1);
    }
    pb.finish_and_clear();

    if parse_errors > 0 {
        println!("  {} {} file(s) had parse errors", "⚠".yellow(), parse_errors);
    }

    println!("{}", "Resolving symbols...".cyan());
    let model = analyzer.resolve_symbols(&parsed_modules)
        .map_err(|e| anyhow::anyhow!("Failed to resolve symbols: {}", e))?;

    Ok(model)
}

pub fn generate_docs(model: &ProjectModel, out: &str, verbose: bool) -> Result<()> {
    println!("{}", "Generating documentation...".cyan());

    let out_path = std::path::Path::new(out);
    std::fs::create_dir_all(out_path)?;

    let modules_path = out_path.join("modules");
    let files_path = out_path.join("files");
    std::fs::create_dir_all(&modules_path)?;
    std::fs::create_dir_all(&files_path)?;

    let renderer = archdoc_core::renderer::Renderer::new();
    let writer = archdoc_core::writer::DiffAwareWriter::new();

    let output_path = std::path::Path::new(".").join("ARCHITECTURE.md");

    // Generate module docs
    for module_id in model.modules.keys() {
        let module_doc_path = modules_path.join(format!("{}.md", sanitize_filename(module_id)));
        match renderer.render_module_md(model, module_id) {
            Ok(module_content) => {
                std::fs::write(&module_doc_path, module_content)?;
            }
            Err(e) => {
                eprintln!("  {} Module {}: {}", "⚠".yellow(), module_id, e);
                let fallback = format!("# Module: {}\n\nTODO: Add module documentation\n", module_id);
                std::fs::write(&module_doc_path, fallback)?;
            }
        }
    }

    // Generate file docs
    for file_doc in model.files.values() {
        let file_doc_path = files_path.join(format!("{}.md", sanitize_filename(&file_doc.path)));

        let mut file_content = format!("# File: {}\n\n", file_doc.path);
        file_content.push_str(&format!("- **Module:** {}\n", file_doc.module_id));
        file_content.push_str(&format!("- **Defined symbols:** {}\n", file_doc.symbols.len()));
        file_content.push_str(&format!("- **Imports:** {}\n\n", file_doc.imports.len()));

        file_content.push_str("<!-- MANUAL:BEGIN -->\n## File intent (manual)\n<FILL_MANUALLY>\n<!-- MANUAL:END -->\n\n---\n\n");

        file_content.push_str("## Imports & file-level dependencies\n<!-- ARCHDOC:BEGIN section=file_imports -->\n> Generated. Do not edit inside this block.\n");
        for import in &file_doc.imports {
            file_content.push_str(&format!("- {}\n", import));
        }
        file_content.push_str("<!-- ARCHDOC:END section=file_imports -->\n\n---\n\n");

        file_content.push_str("## Symbols index\n<!-- ARCHDOC:BEGIN section=symbols_index -->\n> Generated. Do not edit inside this block.\n");
        for symbol_id in &file_doc.symbols {
            if let Some(symbol) = model.symbols.get(symbol_id) {
                file_content.push_str(&format!("- `{}` ({:?})\n", symbol.qualname, symbol.kind));
            }
        }
        file_content.push_str("<!-- ARCHDOC:END section=symbols_index -->\n\n---\n\n");

        file_content.push_str("## Symbol details\n");

        for symbol_id in &file_doc.symbols {
            if model.symbols.contains_key(symbol_id) {
                file_content.push_str(&format!("\n<!-- ARCHDOC:BEGIN symbol id={} -->\n", symbol_id));
                file_content.push_str("<!-- AUTOGENERATED SYMBOL CONTENT WILL BE INSERTED HERE -->\n");
                file_content.push_str(&format!("<!-- ARCHDOC:END symbol id={} -->\n", symbol_id));
            }
        }

        std::fs::write(&file_doc_path, &file_content)?;

        for symbol_id in &file_doc.symbols {
            if model.symbols.contains_key(symbol_id) {
                match renderer.render_symbol_details(model, symbol_id) {
                    Ok(content) => {
                        if verbose {
                            println!("  Updating symbol section for {}", symbol_id);
                        }
                        if let Err(e) = writer.update_symbol_section(&file_doc_path, symbol_id, &content) {
                            eprintln!("  {} Symbol {}: {}", "⚠".yellow(), symbol_id, e);
                        }
                    }
                    Err(e) => {
                        eprintln!("  {} Symbol {}: {}", "⚠".yellow(), symbol_id, e);
                    }
                }
            }
        }
    }

    // Update ARCHITECTURE.md sections
    let sections = [
        ("integrations", renderer.render_integrations_section(model)),
        ("rails", renderer.render_rails_section(model)),
        ("layout", renderer.render_layout_section(model)),
        ("modules_index", renderer.render_modules_index_section(model)),
        ("critical_points", renderer.render_critical_points_section(model)),
    ];

    for (name, result) in sections {
        match result {
            Ok(content) => {
                if let Err(e) = writer.update_file_with_markers(&output_path, &content, name)
                    && verbose {
                        eprintln!("  {} Section {}: {}", "⚠".yellow(), name, e);
                    }
            }
            Err(e) => {
                if verbose {
                    eprintln!("  {} Section {}: {}", "⚠".yellow(), name, e);
                }
            }
        }
    }

    // Update layout.md
    let layout_md_path = out_path.join("layout.md");
    if let Ok(content) = renderer.render_layout_md(model) {
        let _ = std::fs::write(&layout_md_path, &content);
    }

    println!("{} Documentation generated in {}", "✓".green().bold(), out);

    Ok(())
}
