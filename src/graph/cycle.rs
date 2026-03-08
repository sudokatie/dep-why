use petgraph::algo::kosaraju_scc;
use petgraph::graph::NodeIndex;
use petgraph::visit::EdgeRef;
use serde::Serialize;
use std::collections::HashSet;

use crate::graph::{DependencyGraph, Package};

/// A cycle detected in the dependency graph
#[derive(Debug, Clone, Serialize)]
pub struct DependencyCycle {
    /// Packages involved in the cycle, in order
    pub packages: Vec<CyclePackage>,
    /// Length of the cycle
    pub length: usize,
    /// Suggested break point (package that might be easiest to refactor)
    pub suggested_break: Option<String>,
}

/// A package in a cycle
#[derive(Debug, Clone, Serialize)]
pub struct CyclePackage {
    pub name: String,
    pub version: String,
    /// Number of dependencies this package has in the cycle
    pub cycle_deps: usize,
}

/// Result of cycle detection
#[derive(Debug, Clone, Serialize)]
pub struct CycleResult {
    /// Whether any cycles were detected
    pub has_cycles: bool,
    /// All cycles found
    pub cycles: Vec<DependencyCycle>,
    /// Total number of packages involved in cycles
    pub packages_in_cycles: usize,
}

impl CycleResult {
    pub fn none() -> Self {
        Self {
            has_cycles: false,
            cycles: Vec::new(),
            packages_in_cycles: 0,
        }
    }
}

/// Detect cycles in the dependency graph
pub fn detect_cycles(graph: &DependencyGraph) -> CycleResult {
    // Use Kosaraju's algorithm to find strongly connected components
    let sccs = kosaraju_scc(&graph.graph);
    
    let mut cycles = Vec::new();
    let mut all_cycle_nodes = HashSet::new();
    
    for scc in sccs {
        // A SCC with more than one node indicates a cycle
        if scc.len() > 1 {
            let cycle = build_cycle(graph, &scc);
            for pkg in &cycle.packages {
                all_cycle_nodes.insert(pkg.name.clone());
            }
            cycles.push(cycle);
        }
        // A SCC with one node that has a self-loop is also a cycle
        else if scc.len() == 1 {
            let node = scc[0];
            if has_self_loop(&graph.graph, node) {
                let pkg = &graph.graph[node];
                cycles.push(DependencyCycle {
                    packages: vec![CyclePackage {
                        name: pkg.name.clone(),
                        version: pkg.version.clone(),
                        cycle_deps: 1,
                    }],
                    length: 1,
                    suggested_break: Some(pkg.name.clone()),
                });
                all_cycle_nodes.insert(pkg.name.clone());
            }
        }
    }
    
    if cycles.is_empty() {
        return CycleResult::none();
    }
    
    CycleResult {
        has_cycles: true,
        packages_in_cycles: all_cycle_nodes.len(),
        cycles,
    }
}

/// Build a DependencyCycle from a strongly connected component
fn build_cycle(graph: &DependencyGraph, scc: &[NodeIndex]) -> DependencyCycle {
    let scc_set: HashSet<NodeIndex> = scc.iter().copied().collect();
    
    let mut packages = Vec::new();
    
    for &node in scc {
        let pkg = &graph.graph[node];
        
        // Count dependencies within the cycle
        let cycle_deps = graph.graph
            .edges(node)
            .filter(|e| scc_set.contains(&e.target()))
            .count();
        
        packages.push(CyclePackage {
            name: pkg.name.clone(),
            version: pkg.version.clone(),
            cycle_deps,
        });
    }
    
    // Sort by name for consistent output
    packages.sort_by(|a, b| a.name.cmp(&b.name));
    
    // Suggest breaking at package with fewest in-cycle dependencies
    let suggested_break = packages
        .iter()
        .min_by_key(|p| p.cycle_deps)
        .map(|p| p.name.clone());
    
    DependencyCycle {
        length: packages.len(),
        packages,
        suggested_break,
    }
}

/// Check if a node has an edge to itself
fn has_self_loop(graph: &petgraph::graph::DiGraph<Package, crate::graph::Dependency>, node: NodeIndex) -> bool {
    graph.edges(node).any(|e| e.target() == node)
}

/// Format cycles for terminal display
pub fn format_cycles_terminal(result: &CycleResult) -> String {
    if !result.has_cycles {
        return "No circular dependencies detected.".to_string();
    }
    
    let mut output = String::new();
    output.push_str(&format!(
        "Found {} circular dependency chain(s) involving {} packages:\n\n",
        result.cycles.len(),
        result.packages_in_cycles
    ));
    
    for (i, cycle) in result.cycles.iter().enumerate() {
        output.push_str(&format!("Cycle {}: {} packages\n", i + 1, cycle.length));
        output.push_str("  ");
        
        // Draw the cycle
        for (j, pkg) in cycle.packages.iter().enumerate() {
            if j > 0 {
                output.push_str(" -> ");
            }
            output.push_str(&pkg.name);
        }
        output.push_str(&format!(" -> {} (cycle)\n", cycle.packages[0].name));
        
        // Show suggested break point
        if let Some(ref break_pkg) = cycle.suggested_break {
            output.push_str(&format!(
                "  Suggested break point: {} (fewest in-cycle dependencies)\n",
                break_pkg
            ));
        }
        output.push('\n');
    }
    
    output
}

/// Format cycles as JSON
pub fn format_cycles_json(result: &CycleResult) -> serde_json::Value {
    serde_json::to_value(result).unwrap_or_default()
}

/// Format cycles as Mermaid diagram
pub fn format_cycles_mermaid(result: &CycleResult) -> String {
    if !result.has_cycles {
        return "graph TD\n  NoCircularDeps[No circular dependencies]".to_string();
    }
    
    let mut output = String::from("graph TD\n");
    
    for (i, cycle) in result.cycles.iter().enumerate() {
        // Add class for cycle highlighting
        for pkg in &cycle.packages {
            let node_id = pkg.name.replace('-', "_").replace('.', "_");
            output.push_str(&format!("  {}[{}]\n", node_id, pkg.name));
        }
        
        // Add edges
        for j in 0..cycle.packages.len() {
            let from = &cycle.packages[j];
            let to = &cycle.packages[(j + 1) % cycle.packages.len()];
            let from_id = from.name.replace('-', "_").replace('.', "_");
            let to_id = to.name.replace('-', "_").replace('.', "_");
            output.push_str(&format!("  {} --> {}\n", from_id, to_id));
        }
        
        // Add styling for cycle
        output.push_str(&format!("  %% Cycle {}\n", i + 1));
    }
    
    // Add style classes
    output.push_str("  classDef cycle fill:#ff6b6b,stroke:#c92a2a\n");
    for cycle in &result.cycles {
        for pkg in &cycle.packages {
            let node_id = pkg.name.replace('-', "_").replace('.', "_");
            output.push_str(&format!("  class {} cycle\n", node_id));
        }
    }
    
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::{Package, Dependency, DependencyGraph};

    fn make_graph_with_cycle() -> DependencyGraph {
        let mut graph = DependencyGraph::new();
        
        let a = graph.add_package(Package::new("a", "1.0.0"));
        let b = graph.add_package(Package::new("b", "1.0.0"));
        let c = graph.add_package(Package::new("c", "1.0.0"));
        
        // Create cycle: a -> b -> c -> a
        graph.add_dependency(a, b, Dependency::runtime());
        graph.add_dependency(b, c, Dependency::runtime());
        graph.add_dependency(c, a, Dependency::runtime());
        
        graph
    }

    fn make_graph_no_cycle() -> DependencyGraph {
        let mut graph = DependencyGraph::new();
        
        let a = graph.add_package(Package::new("a", "1.0.0"));
        let b = graph.add_package(Package::new("b", "1.0.0"));
        let c = graph.add_package(Package::new("c", "1.0.0"));
        
        // No cycle: a -> b -> c
        graph.add_dependency(a, b, Dependency::runtime());
        graph.add_dependency(b, c, Dependency::runtime());
        
        graph
    }

    #[test]
    fn test_detect_cycle() {
        let graph = make_graph_with_cycle();
        let result = detect_cycles(&graph);
        
        assert!(result.has_cycles);
        assert_eq!(result.cycles.len(), 1);
        assert_eq!(result.cycles[0].length, 3);
    }

    #[test]
    fn test_no_cycle() {
        let graph = make_graph_no_cycle();
        let result = detect_cycles(&graph);
        
        assert!(!result.has_cycles);
        assert!(result.cycles.is_empty());
    }

    #[test]
    fn test_suggested_break_point() {
        let graph = make_graph_with_cycle();
        let result = detect_cycles(&graph);
        
        assert!(result.cycles[0].suggested_break.is_some());
    }

    #[test]
    fn test_self_loop() {
        let mut graph = DependencyGraph::new();
        let a = graph.add_package(Package::new("a", "1.0.0"));
        graph.add_dependency(a, a, Dependency::runtime());
        
        let result = detect_cycles(&graph);
        
        assert!(result.has_cycles);
        assert_eq!(result.cycles.len(), 1);
        assert_eq!(result.cycles[0].length, 1);
    }

    #[test]
    fn test_format_terminal() {
        let graph = make_graph_with_cycle();
        let result = detect_cycles(&graph);
        let output = format_cycles_terminal(&result);
        
        assert!(output.contains("circular dependency"));
        assert!(output.contains("3 packages"));
    }

    #[test]
    fn test_format_no_cycles() {
        let result = CycleResult::none();
        let output = format_cycles_terminal(&result);
        
        assert!(output.contains("No circular dependencies"));
    }

    #[test]
    fn test_format_mermaid() {
        let graph = make_graph_with_cycle();
        let result = detect_cycles(&graph);
        let output = format_cycles_mermaid(&result);
        
        assert!(output.contains("graph TD"));
        assert!(output.contains("-->"));
    }

    #[test]
    fn test_multiple_cycles() {
        let mut graph = DependencyGraph::new();
        
        // First cycle: a -> b -> a
        let a = graph.add_package(Package::new("a", "1.0.0"));
        let b = graph.add_package(Package::new("b", "1.0.0"));
        graph.add_dependency(a, b, Dependency::runtime());
        graph.add_dependency(b, a, Dependency::runtime());
        
        // Second independent cycle: x -> y -> x
        let x = graph.add_package(Package::new("x", "1.0.0"));
        let y = graph.add_package(Package::new("y", "1.0.0"));
        graph.add_dependency(x, y, Dependency::runtime());
        graph.add_dependency(y, x, Dependency::runtime());
        
        let result = detect_cycles(&graph);
        
        assert!(result.has_cycles);
        assert_eq!(result.cycles.len(), 2);
    }
}
