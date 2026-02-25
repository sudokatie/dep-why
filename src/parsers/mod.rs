mod npm;
mod cargo;
mod pip;
mod detect;

pub use npm::NpmParser;
pub use cargo::CargoParser;
pub use pip::PipParser;
pub use detect::{detect_ecosystem, detect_from_path, LockFile};

use crate::cli::Ecosystem;
use crate::error::Result;
use crate::graph::DependencyGraph;
use std::path::Path;

/// Trait for parsing lock files into dependency graphs
pub trait Parser {
    fn parse(&self, path: &Path) -> Result<DependencyGraph>;
}

/// Parse a lock file using the appropriate parser for the ecosystem
pub fn parse_lock_file(path: &Path, ecosystem: Ecosystem) -> Result<DependencyGraph> {
    match ecosystem {
        Ecosystem::Npm => NpmParser.parse(path),
        Ecosystem::Cargo => CargoParser.parse(path),
        Ecosystem::Pip => PipParser.parse(path),
    }
}
