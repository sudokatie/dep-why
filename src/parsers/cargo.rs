use super::Parser;
use crate::error::Result;
use crate::graph::DependencyGraph;
use std::path::Path;

pub struct CargoParser;

impl Parser for CargoParser {
    fn parse(&self, _path: &Path) -> Result<DependencyGraph> {
        // TODO: implement cargo parser
        Ok(DependencyGraph::new())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_cargo_parser_stub() {
        // TODO: add tests when implemented
    }
}
