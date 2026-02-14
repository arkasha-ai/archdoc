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
use rustpython_ast::{Stmt, Expr, Ranged};

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
        
        let code = fs::read_to_string(file_path)
            .map_err(ArchDocError::Io)?;
        
        let ast = ast::Suite::parse(&code, file_path.to_str().unwrap_or("<unknown>"))
            .map_err(|e| ArchDocError::ParseError {
                file: file_path.to_string_lossy().to_string(),
                line: 0,
                message: format!("Failed to parse: {}", e),
            })?;
        
        let mut imports = Vec::new();
        let mut symbols = Vec::new();
        let mut calls = Vec::new();
        
        for stmt in &ast {
            self.extract_from_statement(stmt, None, &mut imports, &mut symbols, &mut calls, 0);
        }
        
        let parsed_module = ParsedModule {
            path: file_path.to_path_buf(),
            module_path: file_path.to_string_lossy().to_string(),
            imports,
            symbols,
            calls,
        };
        
        self.cache_manager.store_module(file_path, parsed_module.clone())?;
        
        Ok(parsed_module)
    }
    
    fn extract_from_statement(
        &self,
        stmt: &Stmt,
        parent_class: Option<&str>,
        imports: &mut Vec<Import>,
        symbols: &mut Vec<Symbol>,
        calls: &mut Vec<Call>,
        depth: usize,
    ) {
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
                let (kind, qualname) = if let Some(class_name) = parent_class {
                    (crate::model::SymbolKind::Method, format!("{}.{}", class_name, func_def.name))
                } else {
                    (crate::model::SymbolKind::Function, func_def.name.to_string())
                };
                
                let signature = self.build_function_signature(&func_def.name, &func_def.args);
                let integrations_flags = self.detect_integrations(&func_def.body, &self._config);
                let docstring = self.extract_docstring(&func_def.body);
                
                let symbol = Symbol {
                    id: qualname.clone(),
                    kind,
                    module_id: String::new(),
                    file_id: String::new(),
                    qualname: qualname.clone(),
                    signature,
                    annotations: None,
                    docstring_first_line: docstring,
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
                
                for body_stmt in &func_def.body {
                    self.extract_from_statement(body_stmt, parent_class, imports, symbols, calls, depth + 1);
                }
                // Extract calls from body expressions recursively
                self.extract_calls_from_body(&func_def.body, Some(&qualname), calls);
            }
            Stmt::AsyncFunctionDef(func_def) => {
                let (kind, qualname) = if let Some(class_name) = parent_class {
                    (crate::model::SymbolKind::Method, format!("{}.{}", class_name, func_def.name))
                } else {
                    (crate::model::SymbolKind::AsyncFunction, func_def.name.to_string())
                };
                
                let signature = format!("async {}", self.build_function_signature(&func_def.name, &func_def.args));
                let integrations_flags = self.detect_integrations(&func_def.body, &self._config);
                let docstring = self.extract_docstring(&func_def.body);
                
                let symbol = Symbol {
                    id: qualname.clone(),
                    kind,
                    module_id: String::new(),
                    file_id: String::new(),
                    qualname: qualname.clone(),
                    signature,
                    annotations: None,
                    docstring_first_line: docstring,
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
                
                for body_stmt in &func_def.body {
                    self.extract_from_statement(body_stmt, parent_class, imports, symbols, calls, depth + 1);
                }
                self.extract_calls_from_body(&func_def.body, Some(&qualname), calls);
            }
            Stmt::ClassDef(class_def) => {
                let integrations_flags = self.detect_integrations(&class_def.body, &self._config);
                let docstring = self.extract_docstring(&class_def.body);
                
                let symbol = Symbol {
                    id: class_def.name.to_string(),
                    kind: crate::model::SymbolKind::Class,
                    module_id: String::new(),
                    file_id: String::new(),
                    qualname: class_def.name.to_string(),
                    signature: format!("class {}", class_def.name),
                    annotations: None,
                    docstring_first_line: docstring,
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
                
                // Process class body with class name as parent
                for body_stmt in &class_def.body {
                    self.extract_from_statement(body_stmt, Some(&class_def.name), imports, symbols, calls, depth + 1);
                }
            }
            Stmt::Expr(expr_stmt) => {
                let caller = parent_class.map(|c| c.to_string()).unwrap_or_else(|| "unknown".to_string());
                self.extract_from_expression(&expr_stmt.value, Some(&caller), calls);
            }
            // Recurse into compound statements to find calls
            Stmt::If(if_stmt) => {
                let caller = parent_class.map(|c| c.to_string());
                self.extract_from_expression(&if_stmt.test, caller.as_deref(), calls);
                self.extract_calls_from_body(&if_stmt.body, caller.as_deref(), calls);
                self.extract_calls_from_body(&if_stmt.orelse, caller.as_deref(), calls);
            }
            Stmt::For(for_stmt) => {
                let caller = parent_class.map(|c| c.to_string());
                self.extract_from_expression(&for_stmt.iter, caller.as_deref(), calls);
                self.extract_calls_from_body(&for_stmt.body, caller.as_deref(), calls);
                self.extract_calls_from_body(&for_stmt.orelse, caller.as_deref(), calls);
            }
            Stmt::While(while_stmt) => {
                let caller = parent_class.map(|c| c.to_string());
                self.extract_from_expression(&while_stmt.test, caller.as_deref(), calls);
                self.extract_calls_from_body(&while_stmt.body, caller.as_deref(), calls);
                self.extract_calls_from_body(&while_stmt.orelse, caller.as_deref(), calls);
            }
            Stmt::With(with_stmt) => {
                let caller = parent_class.map(|c| c.to_string());
                for item in &with_stmt.items {
                    self.extract_from_expression(&item.context_expr, caller.as_deref(), calls);
                }
                self.extract_calls_from_body(&with_stmt.body, caller.as_deref(), calls);
            }
            Stmt::Return(return_stmt) => {
                if let Some(value) = &return_stmt.value {
                    let caller = parent_class.map(|c| c.to_string());
                    self.extract_from_expression(value, caller.as_deref(), calls);
                }
            }
            Stmt::Assign(assign_stmt) => {
                let caller = parent_class.map(|c| c.to_string());
                self.extract_from_expression(&assign_stmt.value, caller.as_deref(), calls);
            }
            Stmt::Try(try_stmt) => {
                let caller = parent_class.map(|c| c.to_string());
                self.extract_calls_from_body(&try_stmt.body, caller.as_deref(), calls);
                for handler in &try_stmt.handlers {
                    let rustpython_ast::ExceptHandler::ExceptHandler(h) = handler; {
                        self.extract_calls_from_body(&h.body, caller.as_deref(), calls);
                    }
                }
                self.extract_calls_from_body(&try_stmt.orelse, caller.as_deref(), calls);
                self.extract_calls_from_body(&try_stmt.finalbody, caller.as_deref(), calls);
            }
            _ => {}
        }
    }
    
    /// Extract calls from a body (list of statements)
    fn extract_calls_from_body(&self, body: &[Stmt], caller: Option<&str>, calls: &mut Vec<Call>) {
        for stmt in body {
            match stmt {
                Stmt::Expr(expr_stmt) => {
                    self.extract_from_expression(&expr_stmt.value, caller, calls);
                }
                Stmt::Return(return_stmt) => {
                    if let Some(value) = &return_stmt.value {
                        self.extract_from_expression(value, caller, calls);
                    }
                }
                Stmt::Assign(assign_stmt) => {
                    self.extract_from_expression(&assign_stmt.value, caller, calls);
                }
                Stmt::If(if_stmt) => {
                    self.extract_from_expression(&if_stmt.test, caller, calls);
                    self.extract_calls_from_body(&if_stmt.body, caller, calls);
                    self.extract_calls_from_body(&if_stmt.orelse, caller, calls);
                }
                Stmt::For(for_stmt) => {
                    self.extract_from_expression(&for_stmt.iter, caller, calls);
                    self.extract_calls_from_body(&for_stmt.body, caller, calls);
                    self.extract_calls_from_body(&for_stmt.orelse, caller, calls);
                }
                Stmt::While(while_stmt) => {
                    self.extract_from_expression(&while_stmt.test, caller, calls);
                    self.extract_calls_from_body(&while_stmt.body, caller, calls);
                    self.extract_calls_from_body(&while_stmt.orelse, caller, calls);
                }
                Stmt::With(with_stmt) => {
                    for item in &with_stmt.items {
                        self.extract_from_expression(&item.context_expr, caller, calls);
                    }
                    self.extract_calls_from_body(&with_stmt.body, caller, calls);
                }
                Stmt::Try(try_stmt) => {
                    self.extract_calls_from_body(&try_stmt.body, caller, calls);
                    for handler in &try_stmt.handlers {
                        let rustpython_ast::ExceptHandler::ExceptHandler(h) = handler; {
                            self.extract_calls_from_body(&h.body, caller, calls);
                        }
                    }
                    self.extract_calls_from_body(&try_stmt.orelse, caller, calls);
                    self.extract_calls_from_body(&try_stmt.finalbody, caller, calls);
                }
                _ => {}
            }
        }
    }
    
    fn build_function_signature(&self, name: &str, args: &rustpython_ast::Arguments) -> String {
        let mut params = Vec::new();
        
        for arg in &args.args {
            let param_name = arg.def.arg.to_string();
            let annotation = arg.def.annotation.as_ref()
                .map(|a| format!(": {}", self.expr_to_string(a)))
                .unwrap_or_default();
            
            if let Some(default) = &arg.default {
                params.push(format!("{}{} = {}", param_name, annotation, self.expr_to_string(default)));
            } else {
                params.push(format!("{}{}", param_name, annotation));
            }
        }
        
        // Add *args
        if let Some(vararg) = &args.vararg {
            let annotation = vararg.annotation.as_ref()
                .map(|a| format!(": {}", self.expr_to_string(a)))
                .unwrap_or_default();
            params.push(format!("*{}{}", vararg.arg, annotation));
        }
        
        // Add **kwargs
        if let Some(kwarg) = &args.kwarg {
            let annotation = kwarg.annotation.as_ref()
                .map(|a| format!(": {}", self.expr_to_string(a)))
                .unwrap_or_default();
            params.push(format!("**{}{}", kwarg.arg, annotation));
        }
        
        format!("def {}({})", name, params.join(", "))
    }
    
    fn extract_docstring(&self, body: &[Stmt]) -> Option<String> {
        if let Some(first_stmt) = body.first() {
            if let Stmt::Expr(expr_stmt) = first_stmt {
                if let Expr::Constant(constant_expr) = &*expr_stmt.value {
                    if let Some(docstring) = constant_expr.value.as_str() {
                        // Return full docstring, trimmed
                        let trimmed = docstring.trim();
                        if trimmed.is_empty() {
                            return None;
                        }
                        return Some(trimmed.to_string());
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
        
        let body_str = format!("{:?}", body);
        
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
    
    fn extract_from_expression(&self, expr: &Expr, current_symbol: Option<&str>, calls: &mut Vec<Call>) {
        match expr {
            Expr::Call(call_expr) => {
                let callee_expr = self.expr_to_string(&call_expr.func);
                calls.push(Call {
                    caller_symbol: current_symbol.unwrap_or("unknown").to_string(),
                    callee_expr,
                    line_number: call_expr.range().start().into(),
                    call_type: CallType::Unresolved,
                });
                
                // Recursively process the function expression itself
                self.extract_from_expression(&call_expr.func, current_symbol, calls);
                
                for arg in &call_expr.args {
                    self.extract_from_expression(arg, current_symbol, calls);
                }
                for keyword in &call_expr.keywords {
                    self.extract_from_expression(&keyword.value, current_symbol, calls);
                }
            }
            Expr::Attribute(attr_expr) => {
                self.extract_from_expression(&attr_expr.value, current_symbol, calls);
            }
            Expr::BoolOp(bool_op) => {
                for value in &bool_op.values {
                    self.extract_from_expression(value, current_symbol, calls);
                }
            }
            Expr::BinOp(bin_op) => {
                self.extract_from_expression(&bin_op.left, current_symbol, calls);
                self.extract_from_expression(&bin_op.right, current_symbol, calls);
            }
            Expr::UnaryOp(unary_op) => {
                self.extract_from_expression(&unary_op.operand, current_symbol, calls);
            }
            Expr::IfExp(if_exp) => {
                self.extract_from_expression(&if_exp.test, current_symbol, calls);
                self.extract_from_expression(&if_exp.body, current_symbol, calls);
                self.extract_from_expression(&if_exp.orelse, current_symbol, calls);
            }
            Expr::Dict(dict_expr) => {
                for key in &dict_expr.keys {
                    if let Some(k) = key {
                        self.extract_from_expression(k, current_symbol, calls);
                    }
                }
                for value in &dict_expr.values {
                    self.extract_from_expression(value, current_symbol, calls);
                }
            }
            Expr::List(list_expr) => {
                for elt in &list_expr.elts {
                    self.extract_from_expression(elt, current_symbol, calls);
                }
            }
            Expr::Tuple(tuple_expr) => {
                for elt in &tuple_expr.elts {
                    self.extract_from_expression(elt, current_symbol, calls);
                }
            }
            Expr::ListComp(comp) => {
                self.extract_from_expression(&comp.elt, current_symbol, calls);
                for generator in &comp.generators {
                    self.extract_from_expression(&generator.iter, current_symbol, calls);
                    for if_clause in &generator.ifs {
                        self.extract_from_expression(if_clause, current_symbol, calls);
                    }
                }
            }
            Expr::Compare(compare) => {
                self.extract_from_expression(&compare.left, current_symbol, calls);
                for comp in &compare.comparators {
                    self.extract_from_expression(comp, current_symbol, calls);
                }
            }
            Expr::JoinedStr(joined) => {
                for value in &joined.values {
                    self.extract_from_expression(value, current_symbol, calls);
                }
            }
            Expr::FormattedValue(fv) => {
                self.extract_from_expression(&fv.value, current_symbol, calls);
            }
            Expr::Subscript(sub) => {
                self.extract_from_expression(&sub.value, current_symbol, calls);
                self.extract_from_expression(&sub.slice, current_symbol, calls);
            }
            Expr::Starred(starred) => {
                self.extract_from_expression(&starred.value, current_symbol, calls);
            }
            Expr::Await(await_expr) => {
                self.extract_from_expression(&await_expr.value, current_symbol, calls);
            }
            _ => {}
        }
    }
    
    fn expr_to_string(&self, expr: &Expr) -> String {
        match expr {
            Expr::Name(name_expr) => name_expr.id.to_string(),
            Expr::Attribute(attr_expr) => {
                format!("{}.{}", self.expr_to_string(&attr_expr.value), attr_expr.attr)
            }
            Expr::Constant(c) => {
                if let Some(s) = c.value.as_str() {
                    format!("\"{}\"", s)
                } else {
                    format!("{:?}", c.value)
                }
            }
            Expr::Subscript(sub) => {
                format!("{}[{}]", self.expr_to_string(&sub.value), self.expr_to_string(&sub.slice))
            }
            _ => "<complex_expression>".to_string(),
        }
    }
    
    pub fn resolve_symbols(&self, modules: &[ParsedModule]) -> Result<ProjectModel, ArchDocError> {
        let mut project_model = ProjectModel::new();
        
        // Build import alias map for call resolution
        // alias_name -> original_module_name
        let mut import_aliases: std::collections::HashMap<String, String> = std::collections::HashMap::new();
        for parsed_module in modules {
            for import in &parsed_module.imports {
                if let Some(alias) = &import.alias {
                    import_aliases.insert(alias.clone(), import.module_name.clone());
                }
            }
        }
        
        for parsed_module in modules {
            let module_id = parsed_module.module_path.clone();
            let file_id = parsed_module.path.to_string_lossy().to_string();
            
            let file_doc = FileDoc {
                id: file_id.clone(),
                path: parsed_module.path.to_string_lossy().to_string(),
                module_id: module_id.clone(),
                imports: parsed_module.imports.iter().map(|i| i.module_name.clone()).collect(),
                outbound_modules: Vec::new(),
                inbound_files: Vec::new(),
                symbols: parsed_module.symbols.iter().map(|s| s.id.clone()).collect(),
            };
            project_model.files.insert(file_id.clone(), file_doc);
            
            for mut symbol in parsed_module.symbols.clone() {
                symbol.module_id = module_id.clone();
                symbol.file_id = file_id.clone();
                project_model.symbols.insert(symbol.id.clone(), symbol);
            }
            
            let module = Module {
                id: module_id.clone(),
                path: parsed_module.path.to_string_lossy().to_string(),
                files: vec![file_id.clone()],
                doc_summary: None,
                outbound_modules: Vec::new(),
                inbound_modules: Vec::new(),
                symbols: parsed_module.symbols.iter().map(|s| s.id.clone()).collect(),
            };
            project_model.modules.insert(module_id, module);
        }
        
        self.build_dependency_graphs(&mut project_model, modules)?;
        self.resolve_call_types(&mut project_model, modules, &import_aliases);
        self.compute_metrics(&mut project_model)?;
        
        Ok(project_model)
    }
    
    /// Resolve call types using import information
    fn resolve_call_types(
        &self,
        project_model: &mut ProjectModel,
        parsed_modules: &[ParsedModule],
        import_aliases: &std::collections::HashMap<String, String>,
    ) {
        // Collect all known symbol names
        let known_symbols: std::collections::HashSet<String> = project_model.symbols.keys().cloned().collect();
        
        for parsed_module in parsed_modules {
            let import_map: std::collections::HashMap<String, String> = parsed_module.imports.iter()
                .filter_map(|i| {
                    i.alias.as_ref().map(|alias| (alias.clone(), i.module_name.clone()))
                })
                .collect();
            
            // Also map plain imported names
            let mut name_map: std::collections::HashMap<String, String> = import_map;
            for import in &parsed_module.imports {
                // For "from foo.bar import baz", map "baz" -> "foo.bar.baz"
                let parts: Vec<&str> = import.module_name.split('.').collect();
                if let Some(last) = parts.last() {
                    name_map.insert(last.to_string(), import.module_name.clone());
                }
            }
            
            // Update edge call types
            for edge in &mut project_model.edges.symbol_call_edges {
                let callee = &edge.to_id;
                
                // Check if callee is a known local symbol
                if known_symbols.contains(callee) {
                    edge.edge_type = crate::model::EdgeType::SymbolCall;
                } else {
                    // Check if it matches an import alias
                    let root_name = callee.split('.').next().unwrap_or(callee);
                    if name_map.contains_key(root_name) || import_aliases.contains_key(root_name) {
                        edge.edge_type = crate::model::EdgeType::ExternalCall;
                    } else {
                        edge.edge_type = crate::model::EdgeType::UnresolvedCall;
                    }
                }
            }
        }
    }
    
    fn build_dependency_graphs(&self, project_model: &mut ProjectModel, parsed_modules: &[ParsedModule]) -> Result<(), ArchDocError> {
        for parsed_module in parsed_modules {
            let from_module_id = parsed_module.module_path.clone();
            
            for import in &parsed_module.imports {
                let to_module_id = import.module_name.clone();
                let edge = crate::model::Edge {
                    from_id: from_module_id.clone(),
                    to_id: to_module_id,
                    edge_type: crate::model::EdgeType::ModuleImport,
                    meta: None,
                };
                project_model.edges.module_import_edges.push(edge);
            }
        }
        
        for parsed_module in parsed_modules {
            for call in &parsed_module.calls {
                let callee_expr = call.callee_expr.clone();
                let edge = crate::model::Edge {
                    from_id: call.caller_symbol.clone(),
                    to_id: callee_expr,
                    edge_type: crate::model::EdgeType::SymbolCall,
                    meta: None,
                };
                project_model.edges.symbol_call_edges.push(edge);
            }
        }
        
        Ok(())
    }
    
    fn compute_metrics(&self, project_model: &mut ProjectModel) -> Result<(), ArchDocError> {
        // Collect fan-in/fan-out first to avoid borrow issues
        let mut metrics: std::collections::HashMap<String, (usize, usize)> = std::collections::HashMap::new();
        
        for symbol_id in project_model.symbols.keys() {
            let fan_out = project_model.edges.symbol_call_edges
                .iter()
                .filter(|edge| edge.from_id == *symbol_id)
                .count();
            let fan_in = project_model.edges.symbol_call_edges
                .iter()
                .filter(|edge| edge.to_id == *symbol_id)
                .count();
            metrics.insert(symbol_id.clone(), (fan_in, fan_out));
        }
        
        for (symbol_id, (fan_in, fan_out)) in &metrics {
            if let Some(symbol) = project_model.symbols.get_mut(symbol_id) {
                symbol.metrics.fan_in = *fan_in;
                symbol.metrics.fan_out = *fan_out;
                symbol.metrics.is_critical = *fan_in > 10 || *fan_out > 10;
            }
        }
        
        Ok(())
    }
}
