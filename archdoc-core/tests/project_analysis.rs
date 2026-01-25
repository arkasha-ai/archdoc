//! Tests for analyzing the test project

use archdoc_core::{
    config::Config,
    python_analyzer::PythonAnalyzer,
};
use std::path::Path;

#[test]
fn test_project_analysis() {
    // Load config from test project
    let config = Config::load_from_file(Path::new("../test-project/archdoc.toml")).unwrap();
    
    // Initialize analyzer
    let analyzer = PythonAnalyzer::new(config);
    
    // Parse core module
    let core_module = analyzer.parse_module(Path::new("../test-project/src/core.py")).unwrap();
    
    println!("Core module symbols: {}", core_module.symbols.len());
    for symbol in &core_module.symbols {
        println!("  Symbol: {} ({:?}), DB: {}, HTTP: {}", symbol.id, symbol.kind, symbol.integrations_flags.db, symbol.integrations_flags.http);
    }
    
    println!("Core module calls: {}", core_module.calls.len());
    for call in &core_module.calls {
        println!("  Call: {} -> {}", call.caller_symbol, call.callee_expr);
    }
    
    // Check that we found symbols
    assert!(!core_module.symbols.is_empty()); // Should find at least the main symbols
    
    // Check that we found calls
    assert!(!core_module.calls.is_empty());
    
    // Check that integrations are detected
    let db_integration_found = core_module.symbols.iter().any(|s| s.integrations_flags.db);
    let http_integration_found = core_module.symbols.iter().any(|s| s.integrations_flags.http);
    
    assert!(db_integration_found, "Database integration should be detected");
    assert!(http_integration_found, "HTTP integration should be detected");
    
    // Parse utils module
    let utils_module = analyzer.parse_module(Path::new("../test-project/src/utils.py")).unwrap();
    
    println!("Utils module symbols: {}", utils_module.symbols.len());
    for symbol in &utils_module.symbols {
        println!("  Symbol: {} ({:?}), DB: {}, HTTP: {}", symbol.id, symbol.kind, symbol.integrations_flags.db, symbol.integrations_flags.http);
    }
    
    // Check that we found symbols
    assert!(!utils_module.symbols.is_empty());
}

#[test]
fn test_full_project_resolution() {
    // Load config from test project
    let config = Config::load_from_file(Path::new("../test-project/archdoc.toml")).unwrap();
    
    // Initialize analyzer
    let analyzer = PythonAnalyzer::new(config);
    
    // Parse all modules
    let core_module = analyzer.parse_module(Path::new("../test-project/src/core.py")).unwrap();
    let utils_module = analyzer.parse_module(Path::new("../test-project/src/utils.py")).unwrap();
    
    let modules = vec![core_module, utils_module];
    
    // Resolve symbols
    let project_model = analyzer.resolve_symbols(&modules).unwrap();
    
    // Check project model
    assert!(!project_model.modules.is_empty());
    assert!(!project_model.symbols.is_empty());
    assert!(!project_model.files.is_empty());
    
    // Check that integrations are preserved in the project model
    let db_integration_found = project_model.symbols.values().any(|s| s.integrations_flags.db);
    let http_integration_found = project_model.symbols.values().any(|s| s.integrations_flags.http);
    
    assert!(db_integration_found, "Database integration should be preserved in project model");
    assert!(http_integration_found, "HTTP integration should be preserved in project model");
    
    println!("Project modules: {:?}", project_model.modules.keys().collect::<Vec<_>>());
    println!("Project symbols: {}", project_model.symbols.len());
    
    // Print integration information
    for (id, symbol) in &project_model.symbols {
        if symbol.integrations_flags.db || symbol.integrations_flags.http {
            println!("Symbol {} has DB: {}, HTTP: {}", id, symbol.integrations_flags.db, symbol.integrations_flags.http);
        }
    }
}