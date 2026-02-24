use super::Parser;
use crate::error::{Error, Result};
use crate::graph::{DependencyGraph, GraphBuilder};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

pub struct PipParser;

/// Pipfile.lock format
#[derive(Deserialize)]
struct PipfileLock {
    #[serde(default)]
    default: HashMap<String, PipPackage>,
    #[serde(default)]
    develop: HashMap<String, PipPackage>,
}

#[derive(Deserialize)]
struct PipPackage {
    version: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    markers: Option<String>,
}

/// Poetry.lock format
#[derive(Deserialize)]
struct PoetryLock {
    #[serde(default)]
    package: Vec<PoetryPackage>,
}

#[derive(Deserialize)]
struct PoetryPackage {
    name: String,
    version: String,
    #[serde(default)]
    dependencies: HashMap<String, serde_json::Value>,
    #[serde(default)]
    category: Option<String>,
}

impl Parser for PipParser {
    fn parse(&self, path: &Path) -> Result<DependencyGraph> {
        let content = std::fs::read_to_string(path)?;
        let filename = path.file_name()
            .and_then(|f| f.to_str())
            .unwrap_or("");
        
        if filename == "Pipfile.lock" {
            self.parse_pipfile(&content)
        } else if filename == "poetry.lock" {
            self.parse_poetry(&content)
        } else {
            Err(Error::ParseError(format!("unsupported pip lock file: {}", filename)))
        }
    }
}

impl PipParser {
    fn parse_pipfile(&self, content: &str) -> Result<DependencyGraph> {
        let lock: PipfileLock = serde_json::from_str(content)
            .map_err(|e| Error::ParseError(format!("invalid Pipfile.lock: {}", e)))?;
        
        let mut builder = GraphBuilder::new();
        
        // Add a virtual root for the project
        let root = builder.add_root("project", "0.0.0");
        
        // Add default (runtime) dependencies
        for (name, pkg) in &lock.default {
            let version = pkg.version.as_ref()
                .map(|v| v.trim_start_matches("=="))
                .unwrap_or("0.0.0");
            let idx = builder.add_package(name, version);
            builder.add_dep(root, idx);
        }
        
        // Add develop dependencies
        for (name, pkg) in &lock.develop {
            let version = pkg.version.as_ref()
                .map(|v| v.trim_start_matches("=="))
                .unwrap_or("0.0.0");
            let idx = builder.add_package(name, version);
            builder.add_dev_dep(root, idx);
        }
        
        // Note: Pipfile.lock doesn't store transitive dependency relationships
        // It only stores the resolved versions, not the dependency graph
        // For accurate transitive deps, would need to query PyPI or use pipdeptree
        
        Ok(builder.build())
    }
    
    fn parse_poetry(&self, content: &str) -> Result<DependencyGraph> {
        let lock: PoetryLock = toml::from_str(content)
            .map_err(|e| Error::ParseError(format!("invalid poetry.lock: {}", e)))?;
        
        if lock.package.is_empty() {
            return Err(Error::ParseError("empty poetry.lock".into()));
        }
        
        let mut builder = GraphBuilder::new();
        
        // Build package map
        let mut package_map: HashMap<String, petgraph::graph::NodeIndex> = HashMap::new();
        
        // Add root (first package without category is usually the project)
        let root = builder.add_root("project", "0.0.0");
        package_map.insert("project".to_string(), root);
        
        // Add all packages
        for pkg in &lock.package {
            let idx = builder.add_package(&pkg.name, &pkg.version);
            package_map.insert(pkg.name.to_lowercase(), idx);
            
            // Direct dependencies are main category or no category
            let is_dev = pkg.category.as_ref().map(|c| c == "dev").unwrap_or(false);
            if is_dev {
                builder.add_dev_dep(root, idx);
            } else {
                builder.add_dep(root, idx);
            }
        }
        
        // Add dependency relationships
        for pkg in &lock.package {
            let from_idx = match package_map.get(&pkg.name.to_lowercase()) {
                Some(&idx) => idx,
                None => continue,
            };
            
            for dep_name in pkg.dependencies.keys() {
                let dep_name_lower = dep_name.to_lowercase();
                if let Some(&to_idx) = package_map.get(&dep_name_lower) {
                    builder.add_dep(from_idx, to_idx);
                }
            }
        }
        
        Ok(builder.build())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_parse_pipfile_lock() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("Pipfile.lock"), r#"{
            "_meta": {},
            "default": {
                "requests": {
                    "version": "==2.28.0"
                },
                "urllib3": {
                    "version": "==1.26.0"
                }
            },
            "develop": {
                "pytest": {
                    "version": "==7.0.0"
                }
            }
        }"#).unwrap();
        
        let parser = PipParser;
        let graph = parser.parse(&dir.path().join("Pipfile.lock")).unwrap();
        
        // project + requests + urllib3 + pytest = 4
        assert_eq!(graph.package_count(), 4);
        assert!(graph.get_package("requests").is_some());
        assert!(graph.get_package("pytest").is_some());
    }

    #[test]
    fn test_parse_poetry_lock() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("poetry.lock"), r#"
[[package]]
name = "requests"
version = "2.28.0"
dependencies = { urllib3 = ">=1.21.1" }

[[package]]
name = "urllib3"
version = "1.26.0"
"#).unwrap();
        
        let parser = PipParser;
        let graph = parser.parse(&dir.path().join("poetry.lock")).unwrap();
        
        assert!(graph.get_package("requests").is_some());
        assert!(graph.get_package("urllib3").is_some());
    }

    #[test]
    fn test_parse_pipfile_strips_version_prefix() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("Pipfile.lock"), r#"{
            "default": {
                "flask": {
                    "version": "==2.3.0"
                }
            },
            "develop": {}
        }"#).unwrap();
        
        let parser = PipParser;
        let graph = parser.parse(&dir.path().join("Pipfile.lock")).unwrap();
        
        // Check that version is "2.3.0" not "==2.3.0"
        let flask_idx = graph.get_package("flask").unwrap();
        let flask = &graph.graph[flask_idx];
        assert_eq!(flask.version, "2.3.0");
    }

    #[test]
    fn test_poetry_dev_dependencies() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("poetry.lock"), r#"
[[package]]
name = "flask"
version = "2.3.0"

[[package]]
name = "pytest"
version = "7.0.0"
category = "dev"
"#).unwrap();
        
        let parser = PipParser;
        let graph = parser.parse(&dir.path().join("poetry.lock")).unwrap();
        
        assert!(graph.get_package("flask").is_some());
        assert!(graph.get_package("pytest").is_some());
    }

    #[test]
    fn test_unsupported_file() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("requirements.txt"), "flask==2.3.0").unwrap();
        
        let parser = PipParser;
        let result = parser.parse(&dir.path().join("requirements.txt"));
        
        assert!(result.is_err());
    }
}
