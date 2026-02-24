//! Colored output helpers and filename utilities for ArchDoc CLI

use colored::Colorize;
use archdoc_core::ProjectModel;

/// Sanitize a file path into a safe filename for docs.
/// Removes `./` prefix, replaces `/` with `__`.
pub fn sanitize_filename(filename: &str) -> String {
    let cleaned = filename.strip_prefix("./").unwrap_or(filename);
    cleaned.replace('/', "__")
}

pub fn print_generate_summary(model: &ProjectModel) {
    println!();
    println!("{}", "── Summary ──────────────────────────".dimmed());
    println!("  {} {}", "Files:".bold(), model.files.len());
    println!("  {} {}", "Modules:".bold(), model.modules.len());
    println!("  {} {}", "Symbols:".bold(), model.symbols.len());
    println!("  {} {}",  "Edges:".bold(),
        model.edges.module_import_edges.len() + model.edges.symbol_call_edges.len());

    let integrations: Vec<&str> = {
        let mut v = Vec::new();
        if model.symbols.values().any(|s| s.integrations_flags.http) { v.push("HTTP"); }
        if model.symbols.values().any(|s| s.integrations_flags.db) { v.push("DB"); }
        if model.symbols.values().any(|s| s.integrations_flags.queue) { v.push("Queue"); }
        v
    };
    if !integrations.is_empty() {
        println!("  {} {}", "Integrations:".bold(), integrations.join(", ").yellow());
    }
    println!("{}", "─────────────────────────────────────".dimmed());
}
