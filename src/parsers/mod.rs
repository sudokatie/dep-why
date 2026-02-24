mod npm;
mod cargo;
mod pip;
mod detect;

pub use npm::NpmParser;
pub use cargo::CargoParser;
pub use pip::PipParser;
pub use detect::{detect_manager, LockFile};

use crate::error::Result;
use crate::graph::DependencyGraph;
use std::path::Path;

/// Trait for parsing lock files into dependency graphs
pub trait Parser {
    fn parse(&self, path: &Path) -> Result<DependencyGraph>;
}
