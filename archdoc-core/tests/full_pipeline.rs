//! Full pipeline integration tests for ArchDoc
//!
//! Tests the complete scan → analyze → render pipeline using test-project/.

use archdoc_core::config::Config;
use archdoc_core::cycle_detector;
use archdoc_core::model::{Module, ProjectModel};
use archdoc_core::renderer::Renderer;
use archdoc_core::scanner::FileScanner;
use std::path::Path;

#[test]
fn test_config_load_and_validate() {
    let config_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("test-project/archdoc.toml");

    let config = Config::load_from_file(&config_path).expect("Failed to load config");
    assert_eq!(config.project.language, "python");
    assert!(!config.scan.include.is_empty());
}

#[test]
fn test_config_validate_on_test_project() {
    let config_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("test-project/archdoc.toml");

    let mut config = Config::load_from_file(&config_path).expect("Failed to load config");
    // Set root to actual test-project path so validation passes
    config.project.root = config_path.parent().unwrap().to_string_lossy().to_string();
    assert!(config.validate().is_ok());
}

#[test]
fn test_config_validate_rejects_bad_language() {
    let mut config = Config::default();
    config.project.language = "java".to_string();
    assert!(config.validate().is_err());
}

#[test]
fn test_scan_test_project() {
    let test_project = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("test-project");

    let config_path = test_project.join("archdoc.toml");
    let mut config = Config::load_from_file(&config_path).expect("Failed to load config");
    config.project.root = test_project.to_string_lossy().to_string();

    let scanner = FileScanner::new(config);
    let files = scanner.scan_python_files(&test_project).expect("Scan should succeed");
    assert!(!files.is_empty(), "Should find Python files in test-project");
}

#[test]
fn test_cycle_detection_with_known_cycles() {
    let mut model = ProjectModel::new();

    // Create a known cycle: a → b → c → a
    model.modules.insert(
        "mod_a".into(),
        Module {
            id: "mod_a".into(),
            path: "a.py".into(),
            files: vec![],
            doc_summary: None,
            outbound_modules: vec!["mod_b".into()],
            inbound_modules: vec!["mod_c".into()],
            symbols: vec![],
        },
    );
    model.modules.insert(
        "mod_b".into(),
        Module {
            id: "mod_b".into(),
            path: "b.py".into(),
            files: vec![],
            doc_summary: None,
            outbound_modules: vec!["mod_c".into()],
            inbound_modules: vec!["mod_a".into()],
            symbols: vec![],
        },
    );
    model.modules.insert(
        "mod_c".into(),
        Module {
            id: "mod_c".into(),
            path: "c.py".into(),
            files: vec![],
            doc_summary: None,
            outbound_modules: vec!["mod_a".into()],
            inbound_modules: vec!["mod_b".into()],
            symbols: vec![],
        },
    );

    let cycles = cycle_detector::detect_cycles(&model);
    assert_eq!(cycles.len(), 1, "Should detect exactly one cycle");
    assert_eq!(cycles[0].len(), 3, "Cycle should have 3 modules");
}

#[test]
fn test_cycle_detection_no_cycles() {
    let mut model = ProjectModel::new();

    model.modules.insert(
        "mod_a".into(),
        Module {
            id: "mod_a".into(),
            path: "a.py".into(),
            files: vec![],
            doc_summary: None,
            outbound_modules: vec!["mod_b".into()],
            inbound_modules: vec![],
            symbols: vec![],
        },
    );
    model.modules.insert(
        "mod_b".into(),
        Module {
            id: "mod_b".into(),
            path: "b.py".into(),
            files: vec![],
            doc_summary: None,
            outbound_modules: vec![],
            inbound_modules: vec!["mod_a".into()],
            symbols: vec![],
        },
    );

    let cycles = cycle_detector::detect_cycles(&model);
    assert!(cycles.is_empty(), "Should detect no cycles in DAG");
}

#[test]
fn test_renderer_produces_output() {
    let config = Config::default();
    let model = ProjectModel::new();
    let renderer = Renderer::new();
    let result = renderer.render_architecture_md(&model);
    assert!(result.is_ok(), "Renderer should produce output for empty model");
}

#[test]
fn test_parse_duration_values() {
    use archdoc_core::config::{parse_duration, parse_file_size};

    assert_eq!(parse_duration("24h").unwrap(), 86400);
    assert_eq!(parse_duration("7d").unwrap(), 604800);
    assert_eq!(parse_file_size("10MB").unwrap(), 10 * 1024 * 1024);
    assert_eq!(parse_file_size("1GB").unwrap(), 1024 * 1024 * 1024);
}
