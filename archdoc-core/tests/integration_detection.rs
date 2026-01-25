//! Integration detection tests for ArchDoc
//!
//! These tests verify that the integration detection functionality works correctly.

use std::fs;
use tempfile::TempDir;
use archdoc_core::{Config, python_analyzer::PythonAnalyzer};

#[test]
fn test_http_integration_detection() {
    let config = Config::default();
    let analyzer = PythonAnalyzer::new(config);
    
    // Create a temporary Python file with HTTP integration
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let temp_file = temp_dir.path().join("test.py");
    let python_code = r#"
import requests

def fetch_data():
    response = requests.get("https://api.example.com/data")
    return response.json()
"#;
    fs::write(&temp_file, python_code).expect("Failed to write test file");
    
    // Parse the module
    let parsed_module = analyzer.parse_module(&temp_file)
        .expect("Failed to parse module");
    
    // Check that we found the function
    assert_eq!(parsed_module.symbols.len(), 1);
    let symbol = &parsed_module.symbols[0];
    assert_eq!(symbol.id, "fetch_data");
    
    // Check that HTTP integration is detected
    assert!(symbol.integrations_flags.http);
    assert!(!symbol.integrations_flags.db);
    assert!(!symbol.integrations_flags.queue);
}

#[test]
fn test_db_integration_detection() {
    let config = Config::default();
    let analyzer = PythonAnalyzer::new(config);
    
    // Create a temporary Python file with DB integration
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let temp_file = temp_dir.path().join("test.py");
    let python_code = r#"
import sqlite3

def get_user(user_id):
    conn = sqlite3.connect("database.db")
    cursor = conn.cursor()
    cursor.execute("SELECT * FROM users WHERE id = ?", (user_id,))
    return cursor.fetchone()
"#;
    fs::write(&temp_file, python_code).expect("Failed to write test file");
    
    // Parse the module
    let parsed_module = analyzer.parse_module(&temp_file)
        .expect("Failed to parse module");
    
    // Check that we found the function
    assert_eq!(parsed_module.symbols.len(), 1);
    let symbol = &parsed_module.symbols[0];
    assert_eq!(symbol.id, "get_user");
    
    // Check that DB integration is detected
    assert!(!symbol.integrations_flags.http);
    assert!(symbol.integrations_flags.db);
    assert!(!symbol.integrations_flags.queue);
}

#[test]
fn test_queue_integration_detection() {
    let config = Config::default();
    let analyzer = PythonAnalyzer::new(config);
    
    // Create a temporary Python file with queue integration
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let temp_file = temp_dir.path().join("test.py");
    let python_code = r#"
import redis

def process_job(job_data):
    client = redis.Redis()
    client.lpush("job_queue", job_data)
"#;
    fs::write(&temp_file, python_code).expect("Failed to write test file");
    
    // Parse the module
    let parsed_module = analyzer.parse_module(&temp_file)
        .expect("Failed to parse module");
    
    // Check that we found the function
    assert_eq!(parsed_module.symbols.len(), 1);
    let symbol = &parsed_module.symbols[0];
    assert_eq!(symbol.id, "process_job");
    
    // Check that queue integration is detected
    assert!(!symbol.integrations_flags.http);
    assert!(!symbol.integrations_flags.db);
    assert!(symbol.integrations_flags.queue);
}

#[test]
fn test_no_integration_detection() {
    let config = Config::default();
    let analyzer = PythonAnalyzer::new(config);
    
    // Create a temporary Python file with no integrations
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let temp_file = temp_dir.path().join("test.py");
    let python_code = r#"
def calculate_sum(a, b):
    return a + b
"#;
    fs::write(&temp_file, python_code).expect("Failed to write test file");
    
    // Parse the module
    let parsed_module = analyzer.parse_module(&temp_file)
        .expect("Failed to parse module");
    
    // Check that we found the function
    assert_eq!(parsed_module.symbols.len(), 1);
    let symbol = &parsed_module.symbols[0];
    assert_eq!(symbol.id, "calculate_sum");
    
    // Check that no integrations are detected
    assert!(!symbol.integrations_flags.http);
    assert!(!symbol.integrations_flags.db);
    assert!(!symbol.integrations_flags.queue);
}