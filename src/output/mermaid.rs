use super::OutputFormat;
use crate::error::Result;
use crate::graph::{DependencyGraph, QueryResult};
use std::collections::HashSet;

pub struct MermaidOutput;

impl OutputFormat for MermaidOutput {
    fn format(&self, graph: &DependencyGraph, result: &QueryResult) -> Result<String> {
        let mut output = String::new();
        output.push_str("graph TD\n");
        
        if result.paths.is_empty() {
            output.push_str("    empty[No paths found]\n");
            return Ok(output);
        }
        
        // Collect unique nodes and edges
        let mut nodes = HashSet::new();
        let mut edges = HashSet::new();
        
        for path in &result.paths {
            for &node in &path.nodes {
                nodes.insert(node);
            }
            for window in path.nodes.windows(2) {
                edges.insert((window[0], window[1]));
            }
        }
        
        // Add ROOT node representing the project
        let root_id = "ROOT";
        output.push_str(&format!("    {}[{}]\n", root_id, sanitize_label(&graph.project_name)));
        
        // Connect ROOT to all direct dependents
        for &direct in &result.direct_dependents {
            let pkg = &graph.graph[direct];
            let direct_id = sanitize_mermaid_id(&pkg.name);
            output.push_str(&format!("    {} --> {}\n", root_id, direct_id));
        }
        
        // Output nodes (excluding target, we'll do it special)
        for &node in &nodes {
            if node == result.target {
                continue;
            }
            let pkg = &graph.graph[node];
            let id = sanitize_mermaid_id(&pkg.name);
            let label = format!("{}@{}", pkg.name, pkg.version);
            output.push_str(&format!("    {}[\"{}\"]\n", id, sanitize_label(&label)));
        }
        
        // Output target node with special marker
        let target_pkg = &graph.graph[result.target];
        let target_id = sanitize_mermaid_id(&target_pkg.name);
        let target_label = format!("{}@{} - TARGET", target_pkg.name, target_pkg.version);
        output.push_str(&format!("    {}[\"{}\"]\n", target_id, sanitize_label(&target_label)));
        
        // Output edges (skip edges from ROOT, we did those above)
        for &(from, to) in &edges {
            // Skip if 'from' is a direct dependent (we connect from ROOT instead)
            if result.direct_dependents.contains(&from) {
                // Only output if going to a different node than target through dep chain
                let from_pkg = &graph.graph[from];
                let to_pkg = &graph.graph[to];
                let from_id = sanitize_mermaid_id(&from_pkg.name);
                let to_id = sanitize_mermaid_id(&to_pkg.name);
                output.push_str(&format!("    {} --> {}\n", from_id, to_id));
            } else {
                let from_pkg = &graph.graph[from];
                let to_pkg = &graph.graph[to];
                let from_id = sanitize_mermaid_id(&from_pkg.name);
                let to_id = sanitize_mermaid_id(&to_pkg.name);
                output.push_str(&format!("    {} --> {}\n", from_id, to_id));
            }
        }
        
        // Style target node with highlight color
        output.push_str(&format!("    style {} fill:#f96\n", target_id));
        
        Ok(output)
    }
}

/// Sanitize a string for use as Mermaid node ID
fn sanitize_mermaid_id(name: &str) -> String {
    name.replace('-', "_")
        .replace('@', "_at_")
        .replace(['/', '.'], "_")
}

/// Sanitize a string for use in Mermaid label
fn sanitize_label(label: &str) -> String {
    label.replace('"', "'")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::{GraphBuilder, PathFinder};

    #[test]
    fn test_mermaid_output_header() {
        let mut builder = GraphBuilder::with_name("my-project");
        let root = builder.add_root("myapp", "1.0.0");
        let dep = builder.add_package("my-dep", "2.0.0");
        builder.add_dep(root, dep);
        
        let graph = builder.build();
        let target_idx = graph.get_package("my-dep").unwrap();
        let finder = PathFinder::new(&graph, 20);
        let result = finder.query(target_idx);
        
        let formatter = MermaidOutput;
        let output = formatter.format(&graph, &result).unwrap();
        
        assert!(output.starts_with("graph TD\n"));
    }

    #[test]
    fn test_mermaid_output_root_node() {
        let mut builder = GraphBuilder::with_name("my-project");
        let root = builder.add_root("myapp", "1.0.0");
        let dep = builder.add_package("target", "2.0.0");
        builder.add_dep(root, dep);
        
        let graph = builder.build();
        let target_idx = graph.get_package("target").unwrap();
        let finder = PathFinder::new(&graph, 20);
        let result = finder.query(target_idx);
        
        let formatter = MermaidOutput;
        let output = formatter.format(&graph, &result).unwrap();
        
        assert!(output.contains("ROOT[my-project]"));
    }

    #[test]
    fn test_mermaid_output_target_highlight() {
        let mut builder = GraphBuilder::new();
        let root = builder.add_root("myapp", "1.0.0");
        let dep = builder.add_package("target", "2.0.0");
        builder.add_dep(root, dep);
        
        let graph = builder.build();
        let target_idx = graph.get_package("target").unwrap();
        let finder = PathFinder::new(&graph, 20);
        let result = finder.query(target_idx);
        
        let formatter = MermaidOutput;
        let output = formatter.format(&graph, &result).unwrap();
        
        assert!(output.contains("- TARGET"));
        assert!(output.contains("style target fill:#f96"));
    }

    #[test]
    fn test_mermaid_output_edges() {
        let mut builder = GraphBuilder::new();
        let root = builder.add_root("myapp", "1.0.0");
        let dep = builder.add_package("my-dep", "2.0.0");
        builder.add_dep(root, dep);
        
        let graph = builder.build();
        let target_idx = graph.get_package("my-dep").unwrap();
        let finder = PathFinder::new(&graph, 20);
        let result = finder.query(target_idx);
        
        let formatter = MermaidOutput;
        let output = formatter.format(&graph, &result).unwrap();
        
        assert!(output.contains("-->"));
    }

    #[test]
    fn test_mermaid_output_empty() {
        let graph = DependencyGraph::new();
        let result = QueryResult {
            target: petgraph::graph::NodeIndex::new(0),
            target_name: "missing".to_string(),
            target_version: "1.0.0".to_string(),
            paths: vec![],
            shortest_depth: 0,
            longest_depth: 0,
            direct_dependents: vec![],
        };
        
        let formatter = MermaidOutput;
        let output = formatter.format(&graph, &result).unwrap();
        
        assert!(output.contains("No paths found"));
    }

    #[test]
    fn test_sanitize_mermaid_id() {
        assert_eq!(sanitize_mermaid_id("lodash"), "lodash");
        assert_eq!(sanitize_mermaid_id("my-dep"), "my_dep");
        assert_eq!(sanitize_mermaid_id("@scope/pkg"), "_at_scope_pkg");
    }
}
