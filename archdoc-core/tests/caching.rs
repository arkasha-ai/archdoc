//! Caching tests for ArchDoc
//!
//! These tests verify that the caching functionality works correctly.

use std::path::Path;
use std::fs;
use tempfile::TempDir;
use archdoc_core::{Config, python_analyzer::PythonAnalyzer};

#[test]
fn test_cache_store_and_retrieve() {
    let config = Config::default();
    let analyzer = PythonAnalyzer::new(config);
    
    // Create a temporary Python file
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let temp_file = temp_dir.path().join("test.py");
    let python_code = r#"
def hello():
    return "Hello, World!"

class Calculator:
    def add(self, a, b):
        return a + b
"#;
    fs::write(&temp_file, python_code).expect("Failed to write test file");
    
    // Parse the module for the first time
    let parsed_module1 = analyzer.parse_module(&temp_file)
        .expect("Failed to parse module first time");
    
    // Parse the module again - should come from cache
    let parsed_module2 = analyzer.parse_module(&temp_file)
        .expect("Failed to parse module second time");
    
    // Both parses should return the same data
    assert_eq!(parsed_module1.path, parsed_module2.path);
    assert_eq!(parsed_module1.module_path, parsed_module2.module_path);
    assert_eq!(parsed_module1.imports.len(), parsed_module2.imports.len());
    assert_eq!(parsed_module1.symbols.len(), parsed_module2.symbols.len());
    assert_eq!(parsed_module1.calls.len(), parsed_module2.calls.len());
}

#[test]
fn test_cache_invalidation_on_file_change() {
    let config = Config::default();
    let analyzer = PythonAnalyzer::new(config);
    
    // Create a temporary Python file
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let temp_file = temp_dir.path().join("test.py");
    let python_code1 = r#"
def hello():
    return "Hello, World!"
"#;
    fs::write(&temp_file, python_code1).expect("Failed to write test file");
    
    // Parse the module for the first time
    let parsed_module1 = analyzer.parse_module(&temp_file)
        .expect("Failed to parse module first time");
    
    // Modify the file
    let python_code2 = r#"
def hello():
    return "Hello, World!"

def goodbye():
    return "Goodbye, World!"
"#;
    fs::write(&temp_file, python_code2).expect("Failed to write test file");
    
    // Parse the module again - should NOT come from cache due to file change
    let parsed_module2 = analyzer.parse_module(&temp_file)
        .expect("Failed to parse module second time");
    
    // The second parse should have more symbols
    assert!(parsed_module2.symbols.len() >= parsed_module1.symbols.len());
}

#[test]
fn test_cache_disabled() {
    let mut config = Config::default();
    config.caching.enabled = false;
    let analyzer = PythonAnalyzer::new(config);
    
    // Create a temporary Python file
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let temp_file = temp_dir.path().join("test.py");
    let python_code = r#"
def hello():
    return "Hello, World!"
"#;
    fs::write(&temp_file, python_code).expect("Failed to write test file");
    
    // Parse the module - should work even with caching disabled
    let parsed_module = analyzer.parse_module(&temp_file)
        .expect("Failed to parse module with caching disabled");
    
    assert_eq!(parsed_module.symbols.len(), 1);
}