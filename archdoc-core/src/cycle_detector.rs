//! Dependency cycle detection for module graphs.
//!
//! Uses DFS-based cycle detection to find circular dependencies
//! in the module dependency graph.

use crate::model::ProjectModel;
use std::collections::{HashMap, HashSet};

/// Detect cycles in the module dependency graph.
///
/// Returns a list of cycles, where each cycle is a list of module IDs
/// forming a circular dependency chain.
pub fn detect_cycles(model: &ProjectModel) -> Vec<Vec<String>> {
    let mut visited = HashSet::new();
    let mut rec_stack = HashSet::new();
    let mut path = Vec::new();
    let mut cycles = Vec::new();

    // Build adjacency list from model
    let adj = build_adjacency_list(model);

    for module_id in model.modules.keys() {
        if !visited.contains(module_id.as_str()) {
            dfs(
                module_id,
                &adj,
                &mut visited,
                &mut rec_stack,
                &mut path,
                &mut cycles,
            );
        }
    }

    // Deduplicate cycles (normalize by rotating to smallest element first)
    deduplicate_cycles(cycles)
}

fn build_adjacency_list(model: &ProjectModel) -> HashMap<String, Vec<String>> {
    let mut adj: HashMap<String, Vec<String>> = HashMap::new();

    for (module_id, module) in &model.modules {
        let neighbors: Vec<String> = module
            .outbound_modules
            .iter()
            .filter(|target| model.modules.contains_key(*target))
            .cloned()
            .collect();
        adj.insert(module_id.clone(), neighbors);
    }

    adj
}

fn dfs(
    node: &str,
    adj: &HashMap<String, Vec<String>>,
    visited: &mut HashSet<String>,
    rec_stack: &mut HashSet<String>,
    path: &mut Vec<String>,
    cycles: &mut Vec<Vec<String>>,
) {
    visited.insert(node.to_string());
    rec_stack.insert(node.to_string());
    path.push(node.to_string());

    if let Some(neighbors) = adj.get(node) {
        for neighbor in neighbors {
            if !visited.contains(neighbor.as_str()) {
                dfs(neighbor, adj, visited, rec_stack, path, cycles);
            } else if rec_stack.contains(neighbor.as_str()) {
                // Found a cycle: extract it from path
                if let Some(start_idx) = path.iter().position(|n| n == neighbor) {
                    let cycle: Vec<String> = path[start_idx..].to_vec();
                    cycles.push(cycle);
                }
            }
        }
    }

    path.pop();
    rec_stack.remove(node);
}

fn deduplicate_cycles(cycles: Vec<Vec<String>>) -> Vec<Vec<String>> {
    let mut seen: HashSet<Vec<String>> = HashSet::new();
    let mut unique = Vec::new();

    for cycle in cycles {
        if cycle.is_empty() {
            continue;
        }
        // Normalize: rotate so the lexicographically smallest element is first
        let min_idx = cycle
            .iter()
            .enumerate()
            .min_by_key(|(_, v)| v.as_str())
            .map(|(i, _)| i)
            .unwrap_or(0);

        let mut normalized = Vec::with_capacity(cycle.len());
        for i in 0..cycle.len() {
            normalized.push(cycle[(min_idx + i) % cycle.len()].clone());
        }

        if seen.insert(normalized.clone()) {
            unique.push(normalized);
        }
    }

    unique
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Edges, Module, ProjectModel};
    use std::collections::HashMap;

    fn make_module(id: &str, outbound: Vec<&str>) -> Module {
        Module {
            id: id.to_string(),
            path: format!("{}.py", id),
            files: vec![],
            doc_summary: None,
            outbound_modules: outbound.into_iter().map(String::from).collect(),
            inbound_modules: vec![],
            symbols: vec![],
        }
    }

    #[test]
    fn test_no_cycles() {
        let mut model = ProjectModel::new();
        model.modules.insert("a".into(), make_module("a", vec!["b"]));
        model.modules.insert("b".into(), make_module("b", vec!["c"]));
        model.modules.insert("c".into(), make_module("c", vec![]));

        let cycles = detect_cycles(&model);
        assert!(cycles.is_empty());
    }

    #[test]
    fn test_simple_cycle() {
        let mut model = ProjectModel::new();
        model.modules.insert("a".into(), make_module("a", vec!["b"]));
        model.modules.insert("b".into(), make_module("b", vec!["a"]));

        let cycles = detect_cycles(&model);
        assert_eq!(cycles.len(), 1);
        assert!(cycles[0].contains(&"a".to_string()));
        assert!(cycles[0].contains(&"b".to_string()));
    }

    #[test]
    fn test_three_node_cycle() {
        let mut model = ProjectModel::new();
        model.modules.insert("a".into(), make_module("a", vec!["b"]));
        model.modules.insert("b".into(), make_module("b", vec!["c"]));
        model.modules.insert("c".into(), make_module("c", vec!["a"]));

        let cycles = detect_cycles(&model);
        assert_eq!(cycles.len(), 1);
        assert_eq!(cycles[0].len(), 3);
    }

    #[test]
    fn test_empty_graph() {
        let model = ProjectModel::new();
        let cycles = detect_cycles(&model);
        assert!(cycles.is_empty());
    }

    #[test]
    fn test_self_cycle() {
        let mut model = ProjectModel::new();
        model.modules.insert("a".into(), make_module("a", vec!["a"]));

        let cycles = detect_cycles(&model);
        assert_eq!(cycles.len(), 1);
        assert_eq!(cycles[0], vec!["a".to_string()]);
    }
}
