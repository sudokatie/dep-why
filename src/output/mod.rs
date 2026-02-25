mod tree;
mod json;
mod mermaid;

pub use tree::TreeOutput;
pub use json::JsonOutput;
pub use mermaid::MermaidOutput;

use crate::graph::{DependencyGraph, QueryResult};
use crate::error::Result;

/// Trait for formatting output
pub trait OutputFormat {
    fn format(&self, graph: &DependencyGraph, result: &QueryResult) -> Result<String>;
}
