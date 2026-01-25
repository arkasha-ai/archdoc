//! Python AST analyzer for ArchDoc
//!
//! This module handles parsing Python files using AST and extracting
//! imports, definitions, and calls.

use crate::model::{ParsedModule, ProjectModel, Import, Call, CallType, Symbol, Module, FileDoc};
use crate::config::Config;
use crate::errors::ArchDocError;
use crate::cache::CacheManager;
use std::path::Path;
use std::fs;
use rustpython_parser::{ast, Parse};
use rustpython_ast::{Stmt, StmtClassDef, StmtFunctionDef, Expr, Ranged};

pub struct PythonAnalyzer {
    _config: Config,
    cache_manager: CacheManager,
}

impl PythonAnalyzer {
    pub fn new(config: Config) -> Self {
        let cache_manager = CacheManager::new(config.clone());
        Self { _config: config, cache_manager }
    }
    
    pub fn parse_module(&self, file_path: &Path) -> Result<ParsedModule, ArchDocError> {
        // Try to get from cache first
        if let Some(cached_module) = self.cache_manager.get_cached_module(file_path)? {
            return Ok(cached_module);
        }
        
        // Read the Python file
        let code = fs::read_to_string(file_path)
            .map_err(ArchDocError::Io)?;
        
        // Parse the Python code into an AST
        let ast = ast::Suite::parse(&code, file_path.to_str().unwrap_or("<unknown>"))
            .map_err(|e| ArchDocError::ParseError {
                file: file_path.to_string_lossy().to_string(),
                line: 0, // We don't have line info from the error
                message: format!("Failed to parse: {}", e),
            })?;
        
        // Extract imports, definitions, and calls
        let mut imports = Vec::new();
        let mut symbols = Vec::new();
        let mut calls = Vec::new();
        
        for stmt in ast {
            self.extract_from_statement(&stmt, None, &mut imports, &mut symbols, &mut calls, 0);
        }
        
        let parsed_module = ParsedModule {
            path: file_path.to_path_buf(),
            module_path: file_path.to_string_lossy().to_string(),
            imports,
            symbols,
            calls,
        };
        
        // Store in cache
        self.cache_manager.store_module(file_path, parsed_module.clone())?;
        
        Ok(parsed_module)
    }
    
    fn extract_from_statement(&self, stmt: &Stmt, current_symbol: Option<&str>, imports: &mut Vec<Import>, symbols: &mut Vec<Symbol>, calls: &mut Vec<Call>, depth: usize) {
        match stmt {
            Stmt::Import(import_stmt) => {
                for alias in &import_stmt.names {
                    imports.push(Import {
                        module_name: alias.name.to_string(),
                        alias: alias.asname.as_ref().map(|n| n.to_string()),
                        line_number: alias.range().start().into(),
                    });
                }
            }
            Stmt::ImportFrom(import_from_stmt) => {
                let module_name = import_from_stmt.module.as_ref()
                    .map(|m| m.to_string())
                    .unwrap_or_default();
                for alias in &import_from_stmt.names {
                    let full_name = if module_name.is_empty() {
                        alias.name.to_string()
                    } else {
                        format!("{}.{}", module_name, alias.name)
                    };
                    imports.push(Import {
                        module_name: full_name,
                        alias: alias.asname.as_ref().map(|n| n.to_string()),
                        line_number: alias.range().start().into(),
                    });
                }
            }
            Stmt::FunctionDef(func_def) => {
                // Extract function definition
                // Create a symbol for this function
                let integrations_flags = self.detect_integrations(&func_def.body, &self._config);
                let symbol = Symbol {
                    id: func_def.name.to_string(),
                    kind: crate::model::SymbolKind::Function,
                    module_id: "".to_string(), // Will be filled later
                    file_id: "".to_string(), // Will be filled later
                    qualname: func_def.name.to_string(),
                    signature: format!("def {}(...)", func_def.name),
                    annotations: None,
                    docstring_first_line: self.extract_docstring(&func_def.body), // Extract docstring
                    purpose: "extracted from AST".to_string(),
                    outbound_calls: Vec::new(),
                    inbound_calls: Vec::new(),
                    integrations_flags,
                    metrics: crate::model::SymbolMetrics {
                        fan_in: 0,
                        fan_out: 0,
                        is_critical: false,
                        cycle_participant: false,
                    },
                };
                symbols.push(symbol);
                
                // Recursively process function body for calls
                for body_stmt in &func_def.body {
                    self.extract_from_statement(body_stmt, Some(&func_def.name), imports, symbols, calls, depth + 1);
                }
            }
            Stmt::ClassDef(class_def) => {
                // Extract class definition
                // Create a symbol for this class
                let integrations_flags = self.detect_integrations(&class_def.body, &self._config);
                let symbol = Symbol {
                    id: class_def.name.to_string(),
                    kind: crate::model::SymbolKind::Class,
                    module_id: "".to_string(), // Will be filled later
                    file_id: "".to_string(), // Will be filled later
                    qualname: class_def.name.to_string(),
                    signature: format!("class {}", class_def.name),
                    annotations: None,
                    docstring_first_line: self.extract_docstring(&class_def.body), // Extract docstring
                    purpose: "extracted from AST".to_string(),
                    outbound_calls: Vec::new(),
                    inbound_calls: Vec::new(),
                    integrations_flags,
                    metrics: crate::model::SymbolMetrics {
                        fan_in: 0,
                        fan_out: 0,
                        is_critical: false,
                        cycle_participant: false,
                    },
                };
                symbols.push(symbol);
                
                // Recursively process class body for methods
                for body_stmt in &class_def.body {
                    self.extract_from_statement(body_stmt, Some(&class_def.name), imports, symbols, calls, depth + 1);
                }
            }
            Stmt::Expr(expr_stmt) => {
                self.extract_from_expression(&expr_stmt.value, current_symbol, calls);
            }
            _ => {
                // For other statement types, we might still need to check for calls in expressions
                // This is a simplified approach - a full implementation would need to traverse all expressions
            }
        }
    }
    
    fn extract_docstring(&self, body: &[Stmt]) -> Option<String> {
        // Extract the first statement if it's a string expression (docstring)
        if let Some(first_stmt) = body.first() {
            if let Stmt::Expr(expr_stmt) = first_stmt {
                if let Expr::Constant(constant_expr) = &*expr_stmt.value {
                    if let Some(docstring) = constant_expr.value.as_str() {
                        // Return the first line of the docstring
                        return docstring.lines().next().map(|s| s.to_string());
                    }
                }
            }
        }
        None
    }
    
    fn detect_integrations(&self, body: &[Stmt], config: &Config) -> crate::model::IntegrationFlags {
        let mut flags = crate::model::IntegrationFlags {
            http: false,
            db: false,
            queue: false,
        };
        
        if !config.analysis.detect_integrations {
            return flags;
        }
        
        // Convert body to string for pattern matching
        let body_str = format!("{:?}", body);
        
        // Check for HTTP integrations
        for pattern in &config.analysis.integration_patterns {
            if pattern.type_ == "http" {
                for lib in &pattern.patterns {
                    if body_str.contains(lib) {
                        flags.http = true;
                        break;
                    }
                }
            } else if pattern.type_ == "db" {
                for lib in &pattern.patterns {
                    if body_str.contains(lib) {
                        flags.db = true;
                        break;
                    }
                }
            } else if pattern.type_ == "queue" {
                for lib in &pattern.patterns {
                    if body_str.contains(lib) {
                        flags.queue = true;
                        break;
                    }
                }
            }
        }
        
        flags
    }
    
    #[allow(dead_code)]
    fn extract_function_def(&self, _func_def: &StmtFunctionDef, _symbols: &mut Vec<Symbol>, _calls: &mut Vec<Call>, _depth: usize) {
        // Extract function information
        // This is a simplified implementation - a full implementation would extract more details
    }
    
    #[allow(dead_code)]
    fn extract_class_def(&self, _class_def: &StmtClassDef, _symbols: &mut Vec<Symbol>, _depth: usize) {
        // Extract class information
        // This is a simplified implementation - a full implementation would extract more details
    }
    
    fn extract_from_expression(&self, expr: &Expr, current_symbol: Option<&str>, calls: &mut Vec<Call>) {
        match expr {
            Expr::Call(call_expr) => {
                // Extract call information
                let callee_expr = self.expr_to_string(&call_expr.func);
                calls.push(Call {
                    caller_symbol: current_symbol.unwrap_or("unknown").to_string(), // Use current symbol as caller
                    callee_expr,
                    line_number: call_expr.range().start().into(),
                    call_type: CallType::Unresolved,
                });
                
                // Recursively process arguments
                for arg in &call_expr.args {
                    self.extract_from_expression(arg, current_symbol, calls);
                }
                for keyword in &call_expr.keywords {
                    self.extract_from_expression(&keyword.value, current_symbol, calls);
                }
            }
            Expr::Attribute(attr_expr) => {
                // Recursively process value
                self.extract_from_expression(&attr_expr.value, current_symbol, calls);
            }
            _ => {
                // For other expression types, recursively process child expressions
                // This is a simplified approach - a full implementation would handle all expression variants
            }
        }
    }
    
    fn expr_to_string(&self, expr: &Expr) -> String {
        match expr {
            Expr::Name(name_expr) => name_expr.id.to_string(),
            Expr::Attribute(attr_expr) => {
                format!("{}.{}", self.expr_to_string(&attr_expr.value), attr_expr.attr)
            }
            _ => "<complex_expression>".to_string(),
        }
    }
    
    pub fn resolve_symbols(&self, modules: &[ParsedModule]) -> Result<ProjectModel, ArchDocError> {
        // Build symbol index
        // Resolve cross-module references
        // Build call graph
        
        // This is a simplified implementation that creates a basic project model
        // A full implementation would do much more sophisticated symbol resolution
        
        let mut project_model = ProjectModel::new();
        
        // Add modules to project model
        for parsed_module in modules {
            let module_id = parsed_module.module_path.clone();
            let file_id = parsed_module.path.to_string_lossy().to_string();
            
            // Create file doc
            let file_doc = FileDoc {
                id: file_id.clone(),
                path: parsed_module.path.to_string_lossy().to_string(),
                module_id: module_id.clone(),
                imports: parsed_module.imports.iter().map(|i| i.module_name.clone()).collect(),
                outbound_modules: Vec::new(), // TODO: Resolve outbound modules
                inbound_files: Vec::new(),
                symbols: parsed_module.symbols.iter().map(|s| s.id.clone()).collect(),
            };
            project_model.files.insert(file_id.clone(), file_doc);
            
            // Add symbols to project model
            for mut symbol in parsed_module.symbols.clone() {
                symbol.module_id = module_id.clone();
                symbol.file_id = file_id.clone();
                project_model.symbols.insert(symbol.id.clone(), symbol);
            }
            
            // Create module
            let module = Module {
                id: module_id.clone(),
                path: parsed_module.path.to_string_lossy().to_string(),
                files: vec![file_id.clone()],
                doc_summary: None,
                outbound_modules: Vec::new(), // TODO: Resolve outbound modules
                inbound_modules: Vec::new(),
                symbols: parsed_module.symbols.iter().map(|s| s.id.clone()).collect(),
            };
            project_model.modules.insert(module_id, module);
        }
        
        // Build dependency graphs and compute metrics
        self.build_dependency_graphs(&mut project_model, modules)?;
        self.compute_metrics(&mut project_model)?;
        
        Ok(project_model)
    }
    
    fn build_dependency_graphs(&self, project_model: &mut ProjectModel, parsed_modules: &[ParsedModule]) -> Result<(), ArchDocError> {
        // Build module import edges
        for parsed_module in parsed_modules {
            let from_module_id = parsed_module.module_path.clone();
            
            for import in &parsed_module.imports {
                // Try to resolve the imported module
                let to_module_id = import.module_name.clone();
                
                // Create module import edge
                let edge = crate::model::Edge {
                    from_id: from_module_id.clone(),
                    to_id: to_module_id,
                    edge_type: crate::model::EdgeType::ModuleImport,
                    meta: None,
                };
                project_model.edges.module_import_edges.push(edge);
            }
        }
        
        // Build symbol call edges
        for parsed_module in parsed_modules {
            let _module_id = parsed_module.module_path.clone();
            
            for call in &parsed_module.calls {
                // Try to resolve the called symbol
                let callee_expr = call.callee_expr.clone();
                
                // Create symbol call edge
                let edge = crate::model::Edge {
                    from_id: call.caller_symbol.clone(),
                    to_id: callee_expr,
                    edge_type: crate::model::EdgeType::SymbolCall, // TODO: Map CallType to EdgeType properly
                    meta: None,
                };
                project_model.edges.symbol_call_edges.push(edge);
            }
        }
        
        Ok(())
    }
    
    fn compute_metrics(&self, project_model: &mut ProjectModel) -> Result<(), ArchDocError> {
        // Compute fan-in and fan-out metrics for symbols
        for symbol in project_model.symbols.values_mut() {
            // Fan-out: count of outgoing calls
            let fan_out = project_model.edges.symbol_call_edges
                .iter()
                .filter(|edge| edge.from_id == symbol.id)
                .count();
                
            // Fan-in: count of incoming calls
            let fan_in = project_model.edges.symbol_call_edges
                .iter()
                .filter(|edge| edge.to_id == symbol.id)
                .count();
                
            symbol.metrics.fan_in = fan_in;
            symbol.metrics.fan_out = fan_out;
            symbol.metrics.is_critical = fan_in > 10 || fan_out > 10; // Simple threshold
            symbol.metrics.cycle_participant = false; // TODO: Detect cycles
        }
        
        Ok(())
    }
}