use super::OutputFormat;
use crate::error::Result;
use crate::graph::{DependencyGraph, DependencyPath};
use colored::Colorize;

pub struct TreeOutput;

impl OutputFormat for TreeOutput {
    fn format(&self, graph: &DependencyGraph, paths: &[DependencyPath], show_versions: bool) -> Result<String> {
        let mut output = String::new();
        
        if paths.is_empty() {
            output.push_str("No paths found.\n");
            return Ok(output);
        }
        
        output.push_str(&format!("Found {} path(s):\n\n", paths.len()));
        
        for (i, path) in paths.iter().enumerate() {
            output.push_str(&format!("Path {}:\n", i + 1));
            
            for (j, &node) in path.nodes.iter().enumerate() {
                let pkg = &graph.graph[node];
                let indent = "  ".repeat(j);
                let connector = if j == 0 { "" } else { "└─ " };
                
                let name = if j == 0 {
                    pkg.name.green().to_string()
                } else if j == path.nodes.len() - 1 {
                    pkg.name.yellow().bold().to_string()
                } else {
                    pkg.name.clone()
                };
                
                if show_versions {
                    output.push_str(&format!("{}{}{} @ {}\n", indent, connector, name, pkg.version.dimmed()));
                } else {
                    output.push_str(&format!("{}{}{}\n", indent, connector, name));
                }
            }
            output.push('\n');
        }
        
        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::GraphBuilder;

    fn sample_graph_and_paths() -> (DependencyGraph, Vec<DependencyPath>) {
        let mut builder = GraphBuilder::new();
        let root = builder.add_root("myapp", "1.0.0");
        let dep_a = builder.add_package("dep-a", "2.0.0");
        let target = builder.add_package("target", "3.0.0");
        
        builder.add_dep(root, dep_a);
        builder.add_dep(dep_a, target);
        
        let graph = builder.build();
        let paths = vec![DependencyPath::new(vec![root, dep_a, target])];
        
        (graph, paths)
    }

    #[test]
    fn test_tree_output_format() {
        let (graph, paths) = sample_graph_and_paths();
        let formatter = TreeOutput;
        
        let output = formatter.format(&graph, &paths, false).unwrap();
        assert!(output.contains("myapp"));
        assert!(output.contains("dep-a"));
        assert!(output.contains("target"));
    }

    #[test]
    fn test_tree_output_with_versions() {
        let (graph, paths) = sample_graph_and_paths();
        let formatter = TreeOutput;
        
        let output = formatter.format(&graph, &paths, true).unwrap();
        assert!(output.contains("1.0.0"));
        assert!(output.contains("2.0.0"));
        assert!(output.contains("3.0.0"));
    }

    #[test]
    fn test_tree_output_empty() {
        let graph = DependencyGraph::new();
        let paths: Vec<DependencyPath> = vec![];
        let formatter = TreeOutput;
        
        let output = formatter.format(&graph, &paths, false).unwrap();
        assert!(output.contains("No paths found"));
    }
}
