mod tree;
mod json;
mod mermaid;

pub use tree::TreeOutput;
pub use json::JsonOutput;
pub use mermaid::MermaidOutput;

use crate::graph::{DependencyGraph, QueryResult};
use crate::error::Result;
use crate::license::LicenseSummary;
use crate::security::VulnerabilityInfo;

/// Options for output formatting
#[derive(Debug, Default)]
pub struct OutputOptions<'a> {
    /// Vulnerability information for the target package
    pub vuln_info: Option<&'a VulnerabilityInfo>,
    /// Whether to show license information
    pub show_licenses: bool,
    /// License summary for the project
    pub license_summary: Option<&'a LicenseSummary>,
}

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
    
    /// Format with full output options (security + licenses)
    fn format_with_options(
        &self,
        graph: &DependencyGraph,
        result: &QueryResult,
        options: &OutputOptions,
    ) -> Result<String> {
        // Default implementation falls back to format_with_security
        self.format_with_security(graph, result, options.vuln_info)
    }
}
