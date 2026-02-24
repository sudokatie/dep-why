use super::Parser;
use crate::error::Result;
use crate::graph::DependencyGraph;
use std::path::Path;

pub struct NpmParser;

impl Parser for NpmParser {
    fn parse(&self, _path: &Path) -> Result<DependencyGraph> {
        // TODO: implement npm parser
        Ok(DependencyGraph::new())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_npm_parser_stub() {
        // TODO: add tests when implemented
    }
}
