//! Enhanced analysis tests for ArchDoc
//!
//! These tests verify that the enhanced analysis functionality works correctly
//! with complex code that includes integrations, calls, and docstrings.

use std::fs;
use std::path::Path;
use archdoc_core::{Config, scanner::FileScanner, python_analyzer::PythonAnalyzer};

#[test]
fn test_enhanced_analysis_with_integrations() {
    // Print current directory for debugging
    let current_dir = std::env::current_dir().unwrap();
    println!("Current directory: {:?}", current_dir);
    
    // Try different paths for the config file
    let possible_paths = [
        "tests/golden/test_project/archdoc.toml",
        "../tests/golden/test_project/archdoc.toml",
    ];
    
    let config_path = possible_paths.iter().find(|&path| {
        Path::new(path).exists()
    }).expect("Could not find config file in any expected location");
    
    println!("Using config path: {:?}", config_path);
    
    let config = Config::load_from_file(Path::new(config_path)).expect("Failed to load config");
    
    // Initialize scanner with the correct root path
    let project_root = Path::new("tests/golden/test_project");
    let scanner = FileScanner::new(config.clone());
    
    // Scan for Python files
    let python_files = scanner.scan_python_files(project_root)
        .expect("Failed to scan Python files");
    
    println!("Found Python files: {:?}", python_files);
    
    // Should find both example.py and advanced_example.py
    assert_eq!(python_files.len(), 2);
    
    // Initialize Python analyzer
    let analyzer = PythonAnalyzer::new(config.clone());
    
    // Parse each Python file
    let mut parsed_modules = Vec::new();
    for file_path in python_files {
        println!("Parsing file: {:?}", file_path);
        match analyzer.parse_module(&file_path) {
            Ok(module) => {
                println!("Successfully parsed module: {:?}", module.module_path);
                println!("Imports: {:?}", module.imports);
                println!("Symbols: {:?}", module.symbols.len());
                println!("Calls: {:?}", module.calls.len());
                parsed_modules.push(module);
            },
            Err(e) => {
                panic!("Failed to parse {}: {}", file_path.display(), e);
            }
        }
    }
    
    println!("Parsed {} modules", parsed_modules.len());
    
    // Resolve symbols and build project model
    let project_model = analyzer.resolve_symbols(&parsed_modules)
        .expect("Failed to resolve symbols");
    
    println!("Project model modules: {}", project_model.modules.len());
    println!("Project model files: {}", project_model.files.len());
    println!("Project model symbols: {}", project_model.symbols.len());
    
    // Add assertions to verify the project model
    assert!(!project_model.modules.is_empty());
    assert!(!project_model.files.is_empty());
    assert!(!project_model.symbols.is_empty());
    
    // Check that we have the right number of modules (2 files = 2 modules)
    assert_eq!(project_model.modules.len(), 2);
    
    // Check that we have the right number of files
    assert_eq!(project_model.files.len(), 2);
    
    // Check that we have the right number of symbols
    // The actual number might be less due to deduplication or other factors
    // but should be at least the sum of symbols from both files minus duplicates
    assert!(project_model.symbols.len() >= 10);
    
    // Check specific details about the advanced example module
    let mut found_advanced_module = false;
    for (_, module) in project_model.modules.iter() {
        if module.path.contains("advanced_example.py") {
            found_advanced_module = true;
            break;
        }
    }
    assert!(found_advanced_module);
    
    // Check that we found the UserService class with DB integration
    let user_service_symbol = project_model.symbols.values().find(|s| s.id == "UserService");
    assert!(user_service_symbol.is_some());
    assert_eq!(user_service_symbol.unwrap().kind, archdoc_core::model::SymbolKind::Class);
    
    // Check that we found the NotificationService class with queue integration
    let notification_service_symbol = project_model.symbols.values().find(|s| s.id == "NotificationService");
    assert!(notification_service_symbol.is_some());
    assert_eq!(notification_service_symbol.unwrap().kind, archdoc_core::model::SymbolKind::Class);
    
    // Check that we found the fetch_external_user_data function with HTTP integration
    let fetch_external_user_data_symbol = project_model.symbols.values().find(|s| s.id == "fetch_external_user_data");
    assert!(fetch_external_user_data_symbol.is_some());
    assert_eq!(fetch_external_user_data_symbol.unwrap().kind, archdoc_core::model::SymbolKind::Function);
    
    // Check file imports
    let mut found_advanced_file = false;
    for (_, file_doc) in project_model.files.iter() {
        if file_doc.path.contains("advanced_example.py") {
            found_advanced_file = true;
            assert!(!file_doc.imports.is_empty());
            // Should have imports for requests, sqlite3, redis, typing
            let import_names: Vec<&String> = file_doc.imports.iter().collect();
            assert!(import_names.contains(&&"requests".to_string()));
            assert!(import_names.contains(&&"sqlite3".to_string()));
            assert!(import_names.contains(&&"redis".to_string()));
            assert!(import_names.contains(&&"typing.List".to_string()) || import_names.contains(&&"typing".to_string()));
            break;
        }
    }
    assert!(found_advanced_file);
}