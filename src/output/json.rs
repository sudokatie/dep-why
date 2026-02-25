use super::OutputFormat;
use crate::error::Result;
use crate::graph::{DependencyGraph, QueryResult};
use serde::Serialize;

pub struct JsonOutput;

/// JSON output structure per spec Section 5.2
#[derive(Serialize)]
struct JsonResult {
    target: JsonTarget,
    paths: Vec<JsonPath>,
    summary: JsonSummary,
}

#[derive(Serialize)]
struct JsonTarget {
    name: String,
    version: String,
}

#[derive(Serialize)]
struct JsonPath {
    /// Chain of packages as "name@version" strings
    chain: Vec<String>,
    depth: usize,
    is_dev: bool,
}

#[derive(Serialize)]
struct JsonSummary {
    total_paths: usize,
    shortest_depth: usize,
    longest_depth: usize,
    direct_dependents: Vec<String>,
}

impl OutputFormat for JsonOutput {
    fn format(&self, graph: &DependencyGraph, result: &QueryResult) -> Result<String> {
        let json_paths: Vec<JsonPath> = result.paths
            .iter()
            .map(|path| {
                // Per spec Section 5.2: chain starts from direct dependency, not root
                // Skip the first node (root project) in the chain
                let chain: Vec<String> = path.nodes
                    .iter()
                    .skip(1) // Skip root node
                    .map(|&node| {
                        let pkg = &graph.graph[node];
                        format!("{}@{}", pkg.name, pkg.version)
                    })
                    .collect();
                
                JsonPath {
                    // Depth is chain length (excluding root)
                    depth: chain.len(),
                    is_dev: path.is_dev(),
                    chain,
                }
            })
            .collect();
        
        let direct_dependents: Vec<String> = result.direct_dependents
            .iter()
            .map(|&idx| graph.graph[idx].name.clone())
            .collect();
        
        let output = JsonResult {
            target: JsonTarget {
                name: result.target_name.clone(),
                version: result.target_version.clone(),
            },
            paths: json_paths,
            summary: JsonSummary {
                total_paths: result.total_paths(),
                shortest_depth: result.shortest_depth,
                longest_depth: result.longest_depth,
                direct_dependents,
            },
        };
        
        Ok(serde_json::to_string_pretty(&output)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::{GraphBuilder, PathFinder};

    #[test]
    fn test_json_output_structure() {
        let mut builder = GraphBuilder::new();
        let root = builder.add_root("myapp", "1.0.0");
        let target = builder.add_package("target", "2.0.0");
        builder.add_dep(root, target);
        
        let graph = builder.build();
        let target_idx = graph.get_package("target").unwrap();
        let finder = PathFinder::new(&graph, 20);
        let result = finder.query(target_idx);
        
        let formatter = JsonOutput;
        let output = formatter.format(&graph, &result).unwrap();
        
        // Verify structure
        assert!(output.contains("\"target\":"));
        assert!(output.contains("\"name\": \"target\""));
        assert!(output.contains("\"version\": \"2.0.0\""));
        assert!(output.contains("\"paths\":"));
        assert!(output.contains("\"chain\":"));
        assert!(output.contains("\"is_dev\":"));
        assert!(output.contains("\"summary\":"));
        assert!(output.contains("\"total_paths\":"));
        assert!(output.contains("\"shortest_depth\":"));
        assert!(output.contains("\"longest_depth\":"));
        assert!(output.contains("\"direct_dependents\":"));
    }

    #[test]
    fn test_json_output_chain_format() {
        let mut builder = GraphBuilder::new();
        let root = builder.add_root("myapp", "1.0.0");
        let target = builder.add_package("lodash", "4.17.21");
        builder.add_dep(root, target);
        
        let graph = builder.build();
        let target_idx = graph.get_package("lodash").unwrap();
        let finder = PathFinder::new(&graph, 20);
        let result = finder.query(target_idx);
        
        let formatter = JsonOutput;
        let output = formatter.format(&graph, &result).unwrap();
        
        // Per spec: chain starts from direct dep, not root project
        // Should NOT include myapp (root), only lodash
        assert!(!output.contains("\"myapp@1.0.0\""));
        assert!(output.contains("\"lodash@4.17.21\""));
    }

    #[test]
    fn test_json_output_empty() {
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
        
        let formatter = JsonOutput;
        let output = formatter.format(&graph, &result).unwrap();
        
        assert!(output.contains("\"total_paths\": 0"));
        assert!(output.contains("\"paths\": []"));
    }

    #[test]
    fn test_json_is_valid() {
        let mut builder = GraphBuilder::new();
        let root = builder.add_root("myapp", "1.0.0");
        let target = builder.add_package("target", "2.0.0");
        builder.add_dep(root, target);
        
        let graph = builder.build();
        let target_idx = graph.get_package("target").unwrap();
        let finder = PathFinder::new(&graph, 20);
        let result = finder.query(target_idx);
        
        let formatter = JsonOutput;
        let output = formatter.format(&graph, &result).unwrap();
        
        // Should parse as valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert!(parsed.is_object());
    }
}
