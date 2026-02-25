use super::Parser;
use crate::error::{Error, Result};
use crate::graph::{DependencyGraph, GraphBuilder};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

pub struct CargoParser;

#[derive(Deserialize)]
struct CargoLock {
    #[serde(default)]
    package: Vec<CargoPackage>,
}

#[derive(Deserialize)]
struct CargoPackage {
    name: String,
    version: String,
    #[serde(default)]
    dependencies: Vec<String>,
    #[serde(default)]
    source: Option<String>,
}

impl Parser for CargoParser {
    fn parse(&self, path: &Path) -> Result<DependencyGraph> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| Error::io_error(path, e))?;
        let lock: CargoLock = toml::from_str(&content)
            .map_err(|e| Error::parse_error(path, format!("invalid Cargo.lock: {}", e)))?;
        
        if lock.package.is_empty() {
            return Err(Error::parse_error(path, "empty Cargo.lock"));
        }
        
        // First package is typically the root (workspace member)
        let project_name = lock.package.first().map(|p| p.name.clone()).unwrap_or_else(|| "project".to_string());
        let mut builder = GraphBuilder::with_name(&project_name);
        
        // Build a map of name@version -> NodeIndex
        let mut package_map: HashMap<String, petgraph::graph::NodeIndex> = HashMap::new();
        
        let mut is_first = true;
        
        for pkg in &lock.package {
            let id = format!("{}@{}", pkg.name, pkg.version);
            
            let idx = if is_first && pkg.source.is_none() {
                // Root package (no source = local)
                is_first = false;
                builder.add_root(&pkg.name, &pkg.version)
            } else {
                builder.add_package(&pkg.name, &pkg.version)
            };
            
            package_map.insert(id, idx);
        }
        
        // Second pass: add dependencies
        for pkg in &lock.package {
            let from_id = format!("{}@{}", pkg.name, pkg.version);
            let from_idx = match package_map.get(&from_id) {
                Some(&idx) => idx,
                None => continue,
            };
            
            for dep_str in &pkg.dependencies {
                // Parse dependency string: "name version" or "name version (source)"
                let dep_info = parse_cargo_dep(dep_str);
                let to_id = format!("{}@{}", dep_info.name, dep_info.version);
                
                if let Some(&to_idx) = package_map.get(&to_id) {
                    builder.add_dep(from_idx, to_idx);
                }
            }
        }
        
        Ok(builder.build())
    }
}

struct DepInfo {
    name: String,
    version: String,
}

/// Parse Cargo.lock dependency string
/// Format: "name version" or "name version (source)"
fn parse_cargo_dep(s: &str) -> DepInfo {
    let s = s.trim();
    
    // Remove source info if present
    let s = if let Some(idx) = s.find(" (") {
        &s[..idx]
    } else {
        s
    };
    
    // Split into name and version
    let parts: Vec<&str> = s.splitn(2, ' ').collect();
    
    DepInfo {
        name: parts.first().unwrap_or(&"").to_string(),
        version: parts.get(1).unwrap_or(&"0.0.0").to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    fn create_cargo_lock(dir: &Path, content: &str) {
        fs::write(dir.join("Cargo.lock"), content).unwrap();
    }

    #[test]
    fn test_parse_simple_lockfile() {
        let dir = TempDir::new().unwrap();
        create_cargo_lock(dir.path(), r#"
[[package]]
name = "myapp"
version = "0.1.0"
dependencies = [
    "serde 1.0.0",
]

[[package]]
name = "serde"
version = "1.0.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
"#);
        
        let parser = CargoParser;
        let graph = parser.parse(&dir.path().join("Cargo.lock")).unwrap();
        
        assert_eq!(graph.package_count(), 2);
        assert!(graph.get_package("myapp").is_some());
        assert!(graph.get_package("serde").is_some());
        assert_eq!(graph.project_name, "myapp");
    }

    #[test]
    fn test_parse_nested_deps() {
        let dir = TempDir::new().unwrap();
        create_cargo_lock(dir.path(), r#"
[[package]]
name = "myapp"
version = "0.1.0"
dependencies = [
    "tokio 1.0.0",
]

[[package]]
name = "tokio"
version = "1.0.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
dependencies = [
    "mio 0.8.0",
]

[[package]]
name = "mio"
version = "0.8.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
"#);
        
        let parser = CargoParser;
        let graph = parser.parse(&dir.path().join("Cargo.lock")).unwrap();
        
        assert_eq!(graph.package_count(), 3);
        assert!(graph.get_package("tokio").is_some());
        assert!(graph.get_package("mio").is_some());
    }

    #[test]
    fn test_parse_with_source_info() {
        let dir = TempDir::new().unwrap();
        create_cargo_lock(dir.path(), r#"
[[package]]
name = "myapp"
version = "0.1.0"
dependencies = [
    "serde 1.0.0 (registry+https://github.com/rust-lang/crates.io-index)",
]

[[package]]
name = "serde"
version = "1.0.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
"#);
        
        let parser = CargoParser;
        let graph = parser.parse(&dir.path().join("Cargo.lock")).unwrap();
        
        assert_eq!(graph.dependency_count(), 1);
    }

    #[test]
    fn test_parse_multiple_versions() {
        let dir = TempDir::new().unwrap();
        create_cargo_lock(dir.path(), r#"
[[package]]
name = "myapp"
version = "0.1.0"
dependencies = [
    "rand 0.8.0",
    "old-lib 1.0.0",
]

[[package]]
name = "rand"
version = "0.8.0"
source = "registry+https://github.com/rust-lang/crates.io-index"

[[package]]
name = "rand"
version = "0.7.0"
source = "registry+https://github.com/rust-lang/crates.io-index"

[[package]]
name = "old-lib"
version = "1.0.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
dependencies = [
    "rand 0.7.0",
]
"#);
        
        let parser = CargoParser;
        let graph = parser.parse(&dir.path().join("Cargo.lock")).unwrap();
        
        // 4 packages: myapp, rand 0.8, rand 0.7, old-lib
        assert_eq!(graph.package_count(), 4);
    }

    #[test]
    fn test_parse_cargo_dep() {
        let info = parse_cargo_dep("serde 1.0.0");
        assert_eq!(info.name, "serde");
        assert_eq!(info.version, "1.0.0");
        
        let info = parse_cargo_dep("tokio 1.0.0 (registry+https://github.com/rust-lang/crates.io-index)");
        assert_eq!(info.name, "tokio");
        assert_eq!(info.version, "1.0.0");
    }

    #[test]
    fn test_reject_empty_lockfile() {
        let dir = TempDir::new().unwrap();
        create_cargo_lock(dir.path(), "# empty");
        
        let parser = CargoParser;
        let result = parser.parse(&dir.path().join("Cargo.lock"));
        
        assert!(result.is_err());
    }
}
