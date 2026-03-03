use super::Parser;
use crate::error::{Error, Result};
use crate::graph::{DependencyGraph, GraphBuilder};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

pub struct NpmParser;

#[derive(Deserialize)]
struct PackageLock {
    name: Option<String>,
    #[serde(default)]
    packages: HashMap<String, NpmPackage>,
    #[serde(rename = "lockfileVersion")]
    lockfile_version: Option<u32>,
}

#[derive(Deserialize, Default)]
struct NpmPackage {
    version: Option<String>,
    license: Option<NpmLicense>,
    #[serde(default)]
    dependencies: HashMap<String, String>,
    #[serde(rename = "devDependencies", default)]
    dev_dependencies: HashMap<String, String>,
    #[serde(default)]
    #[allow(dead_code)]
    optional: bool,
    #[serde(default)]
    #[allow(dead_code)]
    dev: bool,
}

/// License can be a string or an object with type field
#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
enum NpmLicense {
    Simple(String),
    Object { r#type: String },
}

impl std::fmt::Display for NpmLicense {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NpmLicense::Simple(s) => write!(f, "{}", s),
            NpmLicense::Object { r#type } => write!(f, "{}", r#type),
        }
    }
}

impl Parser for NpmParser {
    fn parse(&self, path: &Path) -> Result<DependencyGraph> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| Error::io_error(path, e))?;
        let lock: PackageLock = serde_json::from_str(&content)
            .map_err(|e| Error::parse_error(path, format!("invalid package-lock.json: {}", e)))?;
        
        // Check lockfile version
        let version = lock.lockfile_version.unwrap_or(1);
        if version < 2 {
            return Err(Error::parse_error(
                path,
                "package-lock.json v1 not supported, please upgrade to npm 7+"
            ));
        }
        
        let project_name = lock.name.clone().unwrap_or_else(|| "project".to_string());
        let mut builder = GraphBuilder::with_name(&project_name);
        
        // First pass: add all packages
        let mut package_map: HashMap<String, petgraph::graph::NodeIndex> = HashMap::new();
        
        for (key, pkg) in &lock.packages {
            if key.is_empty() {
                // Root package - add as root
                let name = lock.name.clone().unwrap_or_else(|| "root".to_string());
                let version = pkg.version.clone().unwrap_or_else(|| "0.0.0".to_string());
                let idx = builder.add_root(&name, &version);
                package_map.insert(key.clone(), idx);
            } else {
                // Extract package name from path (e.g., "node_modules/lodash" -> "lodash")
                let name = extract_package_name(key);
                let version = pkg.version.clone().unwrap_or_else(|| "0.0.0".to_string());
                let license = pkg.license.as_ref().map(|l| l.to_string());
                let idx = builder.add_package_with_license(&name, &version, license);
                package_map.insert(key.clone(), idx);
            }
        }
        
        // Second pass: add dependencies
        for (key, pkg) in &lock.packages {
            let from_idx = match package_map.get(key) {
                Some(&idx) => idx,
                None => continue,
            };
            
            // Add runtime dependencies
            for dep_name in pkg.dependencies.keys() {
                if let Some(&to_idx) = find_package(&package_map, key, dep_name) {
                    builder.add_dep(from_idx, to_idx);
                }
            }
            
            // Add dev dependencies (mark as dev)
            for dep_name in pkg.dev_dependencies.keys() {
                if let Some(&to_idx) = find_package(&package_map, key, dep_name) {
                    builder.add_dev_dep(from_idx, to_idx);
                }
            }
        }
        
        Ok(builder.build())
    }
}

/// Extract package name from node_modules path
/// "node_modules/lodash" -> "lodash"
/// "node_modules/@scope/pkg" -> "@scope/pkg"
/// "node_modules/a/node_modules/b" -> "b"
fn extract_package_name(path: &str) -> String {
    // Find the last "node_modules/" segment
    let parts: Vec<&str> = path.split("node_modules/").collect();
    let last = parts.last().unwrap_or(&path);
    last.to_string()
}

/// Find a package in the map, handling nested node_modules
fn find_package<'a>(
    map: &'a HashMap<String, petgraph::graph::NodeIndex>,
    from_path: &str,
    dep_name: &str,
) -> Option<&'a petgraph::graph::NodeIndex> {
    // Try nested path first (node_modules/a/node_modules/b)
    if !from_path.is_empty() {
        let nested_path = format!("{}/node_modules/{}", from_path, dep_name);
        if let Some(idx) = map.get(&nested_path) {
            return Some(idx);
        }
    }
    
    // Fall back to top-level
    let top_path = format!("node_modules/{}", dep_name);
    map.get(&top_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    fn create_package_lock(dir: &Path, content: &str) {
        fs::write(dir.join("package-lock.json"), content).unwrap();
    }

    #[test]
    fn test_parse_simple_lockfile() {
        let dir = TempDir::new().unwrap();
        create_package_lock(dir.path(), r#"{
            "name": "test-project",
            "lockfileVersion": 3,
            "packages": {
                "": {
                    "name": "test-project",
                    "version": "1.0.0",
                    "dependencies": {
                        "lodash": "^4.17.21"
                    }
                },
                "node_modules/lodash": {
                    "version": "4.17.21"
                }
            }
        }"#);
        
        let parser = NpmParser;
        let graph = parser.parse(&dir.path().join("package-lock.json")).unwrap();
        
        assert_eq!(graph.package_count(), 2);
        assert!(graph.get_package("test-project").is_some());
        assert!(graph.get_package("lodash").is_some());
        assert_eq!(graph.project_name, "test-project");
    }

    #[test]
    fn test_parse_nested_deps() {
        let dir = TempDir::new().unwrap();
        create_package_lock(dir.path(), r#"{
            "name": "app",
            "lockfileVersion": 3,
            "packages": {
                "": {
                    "name": "app",
                    "version": "1.0.0",
                    "dependencies": {
                        "express": "^4.18.0"
                    }
                },
                "node_modules/express": {
                    "version": "4.18.2",
                    "dependencies": {
                        "accepts": "~1.3.8"
                    }
                },
                "node_modules/accepts": {
                    "version": "1.3.8"
                }
            }
        }"#);
        
        let parser = NpmParser;
        let graph = parser.parse(&dir.path().join("package-lock.json")).unwrap();
        
        assert_eq!(graph.package_count(), 3);
        assert!(graph.get_package("express").is_some());
        assert!(graph.get_package("accepts").is_some());
    }

    #[test]
    fn test_parse_scoped_package() {
        let dir = TempDir::new().unwrap();
        create_package_lock(dir.path(), r#"{
            "name": "app",
            "lockfileVersion": 3,
            "packages": {
                "": {
                    "version": "1.0.0",
                    "dependencies": {
                        "@types/node": "^18.0.0"
                    }
                },
                "node_modules/@types/node": {
                    "version": "18.19.0"
                }
            }
        }"#);
        
        let parser = NpmParser;
        let graph = parser.parse(&dir.path().join("package-lock.json")).unwrap();
        
        assert!(graph.get_package("@types/node").is_some());
    }

    #[test]
    fn test_reject_v1_lockfile() {
        let dir = TempDir::new().unwrap();
        create_package_lock(dir.path(), r#"{
            "name": "old-project",
            "lockfileVersion": 1,
            "dependencies": {}
        }"#);
        
        let parser = NpmParser;
        let result = parser.parse(&dir.path().join("package-lock.json"));
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("v1 not supported"));
    }

    #[test]
    fn test_extract_package_name() {
        assert_eq!(extract_package_name("node_modules/lodash"), "lodash");
        assert_eq!(extract_package_name("node_modules/@scope/pkg"), "@scope/pkg");
        assert_eq!(extract_package_name("node_modules/a/node_modules/b"), "b");
    }

    #[test]
    fn test_parse_with_dev_deps() {
        let dir = TempDir::new().unwrap();
        create_package_lock(dir.path(), r#"{
            "name": "app",
            "lockfileVersion": 3,
            "packages": {
                "": {
                    "version": "1.0.0",
                    "devDependencies": {
                        "jest": "^29.0.0"
                    }
                },
                "node_modules/jest": {
                    "version": "29.7.0",
                    "dev": true
                }
            }
        }"#);
        
        let parser = NpmParser;
        let graph = parser.parse(&dir.path().join("package-lock.json")).unwrap();
        
        assert!(graph.get_package("jest").is_some());
    }

    #[test]
    fn test_parse_license_string() {
        let dir = TempDir::new().unwrap();
        create_package_lock(dir.path(), r#"{
            "name": "app",
            "lockfileVersion": 3,
            "packages": {
                "": {
                    "version": "1.0.0",
                    "dependencies": {
                        "lodash": "^4.17.21"
                    }
                },
                "node_modules/lodash": {
                    "version": "4.17.21",
                    "license": "MIT"
                }
            }
        }"#);
        
        let parser = NpmParser;
        let graph = parser.parse(&dir.path().join("package-lock.json")).unwrap();
        
        let idx = graph.get_package("lodash").unwrap();
        let pkg = &graph.graph[idx];
        assert!(pkg.license.is_some());
        let license = pkg.license.as_ref().unwrap();
        assert_eq!(license.spdx, "MIT");
        assert!(!license.is_copyleft);
    }

    #[test]
    fn test_parse_license_object() {
        let dir = TempDir::new().unwrap();
        create_package_lock(dir.path(), r#"{
            "name": "app",
            "lockfileVersion": 3,
            "packages": {
                "": {
                    "version": "1.0.0",
                    "dependencies": {
                        "some-pkg": "^1.0.0"
                    }
                },
                "node_modules/some-pkg": {
                    "version": "1.0.0",
                    "license": { "type": "GPL-3.0" }
                }
            }
        }"#);
        
        let parser = NpmParser;
        let graph = parser.parse(&dir.path().join("package-lock.json")).unwrap();
        
        let idx = graph.get_package("some-pkg").unwrap();
        let pkg = &graph.graph[idx];
        assert!(pkg.license.is_some());
        let license = pkg.license.as_ref().unwrap();
        assert_eq!(license.spdx, "GPL-3.0");
        assert!(license.is_copyleft);
    }
}
