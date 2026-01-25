//! Tests for the renderer functionality

use archdoc_core::{
    model::{ProjectModel, Symbol, SymbolKind, IntegrationFlags, SymbolMetrics},
    renderer::Renderer,
};
use std::collections::HashMap;

#[test]
fn test_render_with_integrations() {
    // Create a mock project model with integration information
    let mut project_model = ProjectModel::new();
    
    // Add a symbol with database integration
    let db_symbol = Symbol {
        id: "DatabaseManager".to_string(),
        kind: SymbolKind::Class,
        module_id: "test_module".to_string(),
        file_id: "test_file.py".to_string(),
        qualname: "DatabaseManager".to_string(),
        signature: "class DatabaseManager".to_string(),
        annotations: None,
        docstring_first_line: None,
        purpose: "test".to_string(),
        outbound_calls: vec![],
        inbound_calls: vec![],
        integrations_flags: IntegrationFlags {
            db: true,
            http: false,
            queue: false,
        },
        metrics: SymbolMetrics {
            fan_in: 0,
            fan_out: 0,
            is_critical: false,
            cycle_participant: false,
        },
    };
    
    // Add a symbol with HTTP integration
    let http_symbol = Symbol {
        id: "fetch_data".to_string(),
        kind: SymbolKind::Function,
        module_id: "test_module".to_string(),
        file_id: "test_file.py".to_string(),
        qualname: "fetch_data".to_string(),
        signature: "def fetch_data()".to_string(),
        annotations: None,
        docstring_first_line: None,
        purpose: "test".to_string(),
        outbound_calls: vec![],
        inbound_calls: vec![],
        integrations_flags: IntegrationFlags {
            db: false,
            http: true,
            queue: false,
        },
        metrics: SymbolMetrics {
            fan_in: 0,
            fan_out: 0,
            is_critical: false,
            cycle_participant: false,
        },
    };
    
    project_model.symbols.insert("DatabaseManager".to_string(), db_symbol);
    project_model.symbols.insert("fetch_data".to_string(), http_symbol);
    
    // Initialize renderer
    let renderer = Renderer::new();
    
    // Render architecture documentation
    let result = renderer.render_architecture_md(&project_model);
    assert!(result.is_ok());
    
    let rendered_content = result.unwrap();
    println!("Rendered content:\n{}", rendered_content);
    
    // Check that integration sections are present
    assert!(rendered_content.contains("## Integrations"));
    assert!(rendered_content.contains("### Database Integrations"));
    assert!(rendered_content.contains("### HTTP/API Integrations"));
    assert!(rendered_content.contains("DatabaseManager in test_file.py"));
    assert!(rendered_content.contains("fetch_data in test_file.py"));
}