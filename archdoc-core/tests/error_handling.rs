//! Error handling tests for ArchDoc
//!
//! These tests verify that ArchDoc properly handles various error conditions
//! and edge cases.

use std::path::Path;
use std::fs;
use tempfile::TempDir;
use archdoc_core::{Config, scanner::FileScanner, python_analyzer::PythonAnalyzer};

#[test]
fn test_scanner_nonexistent_directory() {
    let config = Config::default();
    let scanner = FileScanner::new(config);
    
    // Try to scan a nonexistent directory
    let result = scanner.scan_python_files(Path::new("/nonexistent/directory"));
    assert!(result.is_err());
    
    // Check that we get an IO error
    match result.unwrap_err() {
        archdoc_core::errors::ArchDocError::Io(_) => {},
        _ => panic!("Expected IO error"),
    }
}

#[test]
fn test_scanner_file_instead_of_directory() {
    let config = Config::default();
    let scanner = FileScanner::new(config);
    
    // Create a temporary file
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let temp_file = temp_dir.path().join("test.txt");
    fs::write(&temp_file, "test content").expect("Failed to write test file");
    
    // Try to scan a file instead of a directory
    let result = scanner.scan_python_files(&temp_file);
    assert!(result.is_err());
    
    // Check that we get an IO error
    match result.unwrap_err() {
        archdoc_core::errors::ArchDocError::Io(_) => {},
        _ => panic!("Expected IO error"),
    }
}

#[test]
fn test_analyzer_nonexistent_file() {
    let config = Config::default();
    let analyzer = PythonAnalyzer::new(config);
    
    // Try to parse a nonexistent file
    let result = analyzer.parse_module(Path::new("/nonexistent/file.py"));
    assert!(result.is_err());
    
    // Check that we get an IO error
    match result.unwrap_err() {
        archdoc_core::errors::ArchDocError::Io(_) => {},
        _ => panic!("Expected IO error"),
    }
}

#[test]
fn test_analyzer_invalid_python_syntax() {
    let config = Config::default();
    let analyzer = PythonAnalyzer::new(config);
    
    // Create a temporary file with invalid Python syntax
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let temp_file = temp_dir.path().join("invalid.py");
    fs::write(&temp_file, "invalid python syntax @@#$%").expect("Failed to write test file");
    
    // Try to parse the file
    let result = analyzer.parse_module(&temp_file);
    assert!(result.is_err());
    
    // Check that we get a parse error
    match result.unwrap_err() {
        archdoc_core::errors::ArchDocError::ParseError { .. } => {},
        _ => panic!("Expected parse error"),
    }
}