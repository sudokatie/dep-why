use super::OutputFormat;
use crate::error::Result;
use crate::graph::{DependencyGraph, DependencyPath};
use serde::Serialize;

pub struct JsonOutput;

#[derive(Serialize)]
struct JsonPath {
    packages: Vec<JsonPackage>,
    length: usize,
}

#[derive(Serialize)]
struct JsonPackage {
    name: String,
    version: String,
}

#[derive(Serialize)]
struct JsonResult {
    paths: Vec<JsonPath>,
    total_paths: usize,
}

impl OutputFormat for JsonOutput {
    fn format(&self, graph: &DependencyGraph, paths: &[DependencyPath], _show_versions: bool) -> Result<String> {
        let json_paths: Vec<JsonPath> = paths
            .iter()
            .map(|path| {
                let packages: Vec<JsonPackage> = path
                    .nodes
                    .iter()
                    .map(|&node| {
                        let pkg = &graph.graph[node];
                        JsonPackage {
                            name: pkg.name.clone(),
                            version: pkg.version.clone(),
                        }
                    })
                    .collect();
                
                JsonPath {
                    length: packages.len(),
                    packages,
                }
            })
            .collect();
        
        let result = JsonResult {
            total_paths: json_paths.len(),
            paths: json_paths,
        };
        
        Ok(serde_json::to_string_pretty(&result)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::GraphBuilder;

    #[test]
    fn test_json_output() {
        let mut builder = GraphBuilder::new();
        let root = builder.add_root("myapp", "1.0.0");
        let target = builder.add_package("target", "2.0.0");
        builder.add_dep(root, target);
        
        let graph = builder.build();
        let paths = vec![DependencyPath::new(vec![root, target])];
        
        let formatter = JsonOutput;
        let output = formatter.format(&graph, &paths, false).unwrap();
        
        assert!(output.contains("\"total_paths\": 1"));
        assert!(output.contains("\"name\": \"myapp\""));
        assert!(output.contains("\"name\": \"target\""));
    }

    #[test]
    fn test_json_output_empty() {
        let graph = DependencyGraph::new();
        let paths: Vec<DependencyPath> = vec![];
        
        let formatter = JsonOutput;
        let output = formatter.format(&graph, &paths, false).unwrap();
        
        assert!(output.contains("\"total_paths\": 0"));
    }
}
