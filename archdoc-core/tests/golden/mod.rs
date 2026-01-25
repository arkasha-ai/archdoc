//! Golden tests for ArchDoc
//!
//! These tests generate documentation for test projects and compare the output
//! with expected "golden" files to ensure consistency.

mod test_utils;

use std::fs;
use std::path::Path;
use archdoc_core::{Config, scanner::FileScanner, python_analyzer::PythonAnalyzer};

#[test]
fn test_simple_project_generation() {
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
    
    // Check specific details about the parsed modules
    // Now we have 2 modules (example.py and advanced_example.py)
    assert_eq!(project_model.modules.len(), 2);
    
    // Find the example.py module
    let mut found_example_module = false;
    for (_, module) in project_model.modules.iter() {
        if module.path.contains("example.py") {
            found_example_module = true;
            break;
        }
    }
    assert!(found_example_module);
    
    // Check that we found the Calculator class
    let calculator_symbol = project_model.symbols.values().find(|s| s.id == "Calculator");
    assert!(calculator_symbol.is_some());
    assert_eq!(calculator_symbol.unwrap().kind, archdoc_core::model::SymbolKind::Class);
    
    // Check that we found the process_numbers function
    let process_numbers_symbol = project_model.symbols.values().find(|s| s.id == "process_numbers");
    assert!(process_numbers_symbol.is_some());
    assert_eq!(process_numbers_symbol.unwrap().kind, archdoc_core::model::SymbolKind::Function);
    
    // Check file imports
    assert!(!project_model.files.is_empty());
    let file_entry = project_model.files.iter().next().unwrap();
    let file_doc = file_entry.1;
    assert!(!file_doc.imports.is_empty());
}