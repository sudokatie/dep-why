mod tree;
mod json;
mod mermaid;

pub use tree::TreeOutput;
pub use json::JsonOutput;
pub use mermaid::MermaidOutput;

use crate::graph::{DependencyGraph, QueryResult};
use crate::error::Result;
use crate::security::VulnerabilityInfo;

/// Trait for formatting output
pub trait OutputFormat {
    fn format(&self, graph: &DependencyGraph, result: &QueryResult) -> Result<String>;
    
    /// Format with optional vulnerability information
    fn format_with_security(
        &self,
        graph: &DependencyGraph,
        result: &QueryResult,
        vuln_info: Option<&VulnerabilityInfo>,
    ) -> Result<String> {
        // Default implementation ignores security info
        let _ = vuln_info;
        self.format(graph, result)
    }
}
