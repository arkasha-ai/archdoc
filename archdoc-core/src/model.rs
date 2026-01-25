//! Intermediate Representation (IR) for ArchDoc
//!
//! This module defines the data structures that represent the analyzed Python project
//! and are used for generating documentation.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectModel {
    pub modules: HashMap<String, Module>,
    pub files: HashMap<String, FileDoc>,
    pub symbols: HashMap<String, Symbol>,
    pub edges: Edges,
}

impl ProjectModel {
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
            files: HashMap::new(),
            symbols: HashMap::new(),
            edges: Edges::new(),
        }
    }
}

impl Default for ProjectModel {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Module {
    pub id: String,
    pub path: String,
    pub files: Vec<String>,
    pub doc_summary: Option<String>,
    pub outbound_modules: Vec<String>,
    pub inbound_modules: Vec<String>,
    pub symbols: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileDoc {
    pub id: String,
    pub path: String,
    pub module_id: String,
    pub imports: Vec<String>, // normalized import strings
    pub outbound_modules: Vec<String>,
    pub inbound_files: Vec<String>,
    pub symbols: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Symbol {
    pub id: String,
    pub kind: SymbolKind,
    pub module_id: String,
    pub file_id: String,
    pub qualname: String,
    pub signature: String,
    pub annotations: Option<HashMap<String, String>>,
    pub docstring_first_line: Option<String>,
    pub purpose: String, // docstring or heuristic
    pub outbound_calls: Vec<String>,
    pub inbound_calls: Vec<String>,
    pub integrations_flags: IntegrationFlags,
    pub metrics: SymbolMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SymbolKind {
    Function,
    AsyncFunction,
    Class,
    Method,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationFlags {
    pub http: bool,
    pub db: bool,
    pub queue: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolMetrics {
    pub fan_in: usize,
    pub fan_out: usize,
    pub is_critical: bool,
    pub cycle_participant: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edges {
    pub module_import_edges: Vec<Edge>,
    pub file_import_edges: Vec<Edge>,
    pub symbol_call_edges: Vec<Edge>,
}

impl Edges {
    pub fn new() -> Self {
        Self {
            module_import_edges: Vec::new(),
            file_import_edges: Vec::new(),
            symbol_call_edges: Vec::new(),
        }
    }
}

impl Default for Edges {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub from_id: String,
    pub to_id: String,
    pub edge_type: EdgeType,
    pub meta: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EdgeType {
    ModuleImport,
    FileImport,
    SymbolCall,
    ExternalCall,
    UnresolvedCall,
}

// Additional structures for Python analysis

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ParsedModule {
    pub path: std::path::PathBuf,
    pub module_path: String,
    pub imports: Vec<Import>,
    pub symbols: Vec<Symbol>,
    pub calls: Vec<Call>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Import {
    pub module_name: String,
    pub alias: Option<String>,
    pub line_number: usize,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Call {
    pub caller_symbol: String,
    pub callee_expr: String,
    pub line_number: usize,
    pub call_type: CallType,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum CallType {
    Local,
    Imported,
    External,
    Unresolved,
}