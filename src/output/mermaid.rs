use super::OutputFormat;
use crate::error::Result;
use crate::graph::{DependencyGraph, DependencyPath};
use std::collections::HashSet;

pub struct MermaidOutput;

impl OutputFormat for MermaidOutput {
    fn format(&self, graph: &DependencyGraph, paths: &[DependencyPath], show_versions: bool) -> Result<String> {
        let mut output = String::new();
        output.push_str("graph TD\n");
        
        if paths.is_empty() {
            output.push_str("    empty[No paths found]\n");
            return Ok(output);
        }
        
        // Collect unique nodes and edges
        let mut nodes = HashSet::new();
        let mut edges = HashSet::new();
        
        for path in paths {
            for &node in &path.nodes {
                nodes.insert(node);
            }
            for window in path.nodes.windows(2) {
                edges.insert((window[0], window[1]));
            }
        }
        
        // Output nodes
        for node in &nodes {
            let pkg = &graph.graph[*node];
            let id = sanitize_mermaid_id(&pkg.name);
            let label = if show_versions {
                format!("{}@{}", pkg.name, pkg.version)
            } else {
                pkg.name.clone()
            };
            output.push_str(&format!("    {}[\"{}\"]\n", id, label));
        }
        
        // Output edges
        for (from, to) in &edges {
            let from_pkg = &graph.graph[*from];
            let to_pkg = &graph.graph[*to];
            let from_id = sanitize_mermaid_id(&from_pkg.name);
            let to_id = sanitize_mermaid_id(&to_pkg.name);
            output.push_str(&format!("    {} --> {}\n", from_id, to_id));
        }
        
        Ok(output)
    }
}

#[allow(clippy::collapsible_str_replace)]
fn sanitize_mermaid_id(name: &str) -> String {
    name.replace('-', "_")
        .replace('@', "_at_")
        .replace('/', "_")
        .replace('.', "_")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::GraphBuilder;

    #[test]
    fn test_mermaid_output() {
        let mut builder = GraphBuilder::new();
        let root = builder.add_root("myapp", "1.0.0");
        let dep = builder.add_package("my-dep", "2.0.0");
        builder.add_dep(root, dep);
        
        let graph = builder.build();
        let paths = vec![DependencyPath::new(vec![root, dep])];
        
        let formatter = MermaidOutput;
        let output = formatter.format(&graph, &paths, false).unwrap();
        
        assert!(output.starts_with("graph TD\n"));
        assert!(output.contains("myapp"));
        assert!(output.contains("my_dep")); // sanitized
        assert!(output.contains("-->"));
    }

    #[test]
    fn test_mermaid_output_empty() {
        let graph = DependencyGraph::new();
        let paths: Vec<DependencyPath> = vec![];
        
        let formatter = MermaidOutput;
        let output = formatter.format(&graph, &paths, false).unwrap();
        
        assert!(output.contains("No paths found"));
    }

    #[test]
    fn test_sanitize_mermaid_id() {
        assert_eq!(sanitize_mermaid_id("lodash"), "lodash");
        assert_eq!(sanitize_mermaid_id("my-dep"), "my_dep");
        assert_eq!(sanitize_mermaid_id("@scope/pkg"), "_at_scope_pkg");
    }
}
