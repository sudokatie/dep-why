use super::Parser;
use crate::error::Result;
use crate::graph::DependencyGraph;
use std::path::Path;

pub struct PipParser;

impl Parser for PipParser {
    fn parse(&self, _path: &Path) -> Result<DependencyGraph> {
        // TODO: implement pip parser
        Ok(DependencyGraph::new())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_pip_parser_stub() {
        // TODO: add tests when implemented
    }
}
