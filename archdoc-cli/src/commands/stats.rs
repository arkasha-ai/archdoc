use archdoc_core::ProjectModel;
use colored::Colorize;

pub fn print_stats(model: &ProjectModel) {
    println!();
    println!("{}", "╔══════════════════════════════════════╗".cyan());
    println!("{}", "║       archdoc project statistics     ║".cyan().bold());
    println!("{}", "╚══════════════════════════════════════╝".cyan());
    println!();

    // Basic counts
    println!("{}", "Overview".bold().underline());
    println!("  Files:   {}", model.files.len().to_string().yellow());
    println!("  Modules: {}", model.modules.len().to_string().yellow());
    println!("  Symbols: {}", model.symbols.len().to_string().yellow());
    println!("  Import edges:  {}", model.edges.module_import_edges.len());
    println!("  Call edges:    {}", model.edges.symbol_call_edges.len());
    println!();

    // Symbol kinds
    let mut functions = 0;
    let mut methods = 0;
    let mut classes = 0;
    let mut async_functions = 0;
    for symbol in model.symbols.values() {
        match symbol.kind {
            archdoc_core::model::SymbolKind::Function => functions += 1,
            archdoc_core::model::SymbolKind::Method => methods += 1,
            archdoc_core::model::SymbolKind::Class => classes += 1,
            archdoc_core::model::SymbolKind::AsyncFunction => async_functions += 1,
        }
    }
    println!("{}", "Symbol breakdown".bold().underline());
    println!("  Classes:         {}", classes);
    println!("  Functions:       {}", functions);
    println!("  Async functions: {}", async_functions);
    println!("  Methods:         {}", methods);
    println!();

    // Top fan-in
    let mut symbols_by_fan_in: Vec<_> = model.symbols.values().collect();
    symbols_by_fan_in.sort_by(|a, b| b.metrics.fan_in.cmp(&a.metrics.fan_in));

    println!("{}", "Top-10 by fan-in (most called)".bold().underline());
    for (i, sym) in symbols_by_fan_in.iter().take(10).enumerate() {
        if sym.metrics.fan_in == 0 { break; }
        let critical = if sym.metrics.is_critical { " ⚠ CRITICAL".red().to_string() } else { String::new() };
        println!("  {}. {} (fan-in: {}){}", i + 1, sym.qualname.green(), sym.metrics.fan_in, critical);
    }
    println!();

    // Top fan-out
    let mut symbols_by_fan_out: Vec<_> = model.symbols.values().collect();
    symbols_by_fan_out.sort_by(|a, b| b.metrics.fan_out.cmp(&a.metrics.fan_out));

    println!("{}", "Top-10 by fan-out (calls many)".bold().underline());
    for (i, sym) in symbols_by_fan_out.iter().take(10).enumerate() {
        if sym.metrics.fan_out == 0 { break; }
        let critical = if sym.metrics.is_critical { " ⚠ CRITICAL".red().to_string() } else { String::new() };
        println!("  {}. {} (fan-out: {}){}", i + 1, sym.qualname.green(), sym.metrics.fan_out, critical);
    }
    println!();

    // Integrations
    let http_symbols: Vec<_> = model.symbols.values().filter(|s| s.integrations_flags.http).collect();
    let db_symbols: Vec<_> = model.symbols.values().filter(|s| s.integrations_flags.db).collect();
    let queue_symbols: Vec<_> = model.symbols.values().filter(|s| s.integrations_flags.queue).collect();

    if !http_symbols.is_empty() || !db_symbols.is_empty() || !queue_symbols.is_empty() {
        println!("{}", "Detected integrations".bold().underline());
        if !http_symbols.is_empty() {
            println!("  {} HTTP: {}", "●".yellow(), http_symbols.iter().map(|s| s.qualname.as_str()).collect::<Vec<_>>().join(", "));
        }
        if !db_symbols.is_empty() {
            println!("  {} DB:   {}", "●".blue(), db_symbols.iter().map(|s| s.qualname.as_str()).collect::<Vec<_>>().join(", "));
        }
        if !queue_symbols.is_empty() {
            println!("  {} Queue: {}", "●".magenta(), queue_symbols.iter().map(|s| s.qualname.as_str()).collect::<Vec<_>>().join(", "));
        }
        println!();
    }

    // Cycles
    println!("{}", "Cycle detection".bold().underline());
    let mut found_cycles = false;
    for edge in &model.edges.module_import_edges {
        let has_reverse = model.edges.module_import_edges.iter()
            .any(|e| e.from_id == edge.to_id && e.to_id == edge.from_id);
        if has_reverse && edge.from_id < edge.to_id {
            println!("  {} {} ↔ {}", "⚠".red(), edge.from_id, edge.to_id);
            found_cycles = true;
        }
    }
    if !found_cycles {
        println!("  {} No cycles detected", "✓".green());
    }
}
