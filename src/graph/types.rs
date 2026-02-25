use petgraph::graph::{DiGraph, NodeIndex};
use serde::Serialize;
use std::collections::HashMap;

/// A package in the dependency graph
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub is_direct: bool,
    pub is_dev: bool,
}

impl Package {
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            is_direct: false,
            is_dev: false,
        }
    }
    
    pub fn direct(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            is_direct: true,
            is_dev: false,
        }
    }
    
    pub fn id(&self) -> String {
        format!("{}@{}", self.name, self.version)
    }
}

/// Type of dependency relationship
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum DependencyType {
    Runtime,
    Dev,
    Build,
    Optional,
    Peer,
}

impl Default for DependencyType {
    fn default() -> Self {
        Self::Runtime
    }
}

/// A dependency edge between packages
#[derive(Debug, Clone)]
pub struct Dependency {
    pub version_constraint: String,
    pub dep_type: DependencyType,
}

impl Default for Dependency {
    fn default() -> Self {
        Self {
            version_constraint: "*".to_string(),
            dep_type: DependencyType::Runtime,
        }
    }
}

impl Dependency {
    pub fn runtime() -> Self {
        Self::default()
    }
    
    pub fn dev() -> Self {
        Self {
            version_constraint: "*".to_string(),
            dep_type: DependencyType::Dev,
        }
    }
    
    pub fn optional() -> Self {
        Self {
            version_constraint: "*".to_string(),
            dep_type: DependencyType::Optional,
        }
    }
    
    pub fn is_dev(&self) -> bool {
        matches!(self.dep_type, DependencyType::Dev)
    }
}

/// The dependency graph
pub struct DependencyGraph {
    pub graph: DiGraph<Package, Dependency>,
    pub index_map: HashMap<String, NodeIndex>,
    pub root_packages: Vec<NodeIndex>,
    pub project_name: String,
}

impl std::fmt::Debug for DependencyGraph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DependencyGraph")
            .field("project_name", &self.project_name)
            .field("package_count", &self.graph.node_count())
            .field("dependency_count", &self.graph.edge_count())
            .field("root_count", &self.root_packages.len())
            .finish()
    }
}

impl DependencyGraph {
    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
            index_map: HashMap::new(),
            root_packages: Vec::new(),
            project_name: "project".to_string(),
        }
    }
    
    pub fn with_name(name: impl Into<String>) -> Self {
        Self {
            graph: DiGraph::new(),
            index_map: HashMap::new(),
            root_packages: Vec::new(),
            project_name: name.into(),
        }
    }
    
    pub fn add_package(&mut self, pkg: Package) -> NodeIndex {
        let id = pkg.id();
        if let Some(&idx) = self.index_map.get(&id) {
            return idx;
        }
        let idx = self.graph.add_node(pkg);
        self.index_map.insert(id, idx);
        idx
    }
    
    pub fn add_dependency(&mut self, from: NodeIndex, to: NodeIndex, dep: Dependency) {
        self.graph.add_edge(from, to, dep);
    }
    
    pub fn get_package(&self, name: &str) -> Option<NodeIndex> {
        // First try exact match with version
        if let Some(&idx) = self.index_map.get(name) {
            return Some(idx);
        }
        // Try matching by name only (first match)
        for (id, &idx) in &self.index_map {
            if id.starts_with(&format!("{}@", name)) {
                return Some(idx);
            }
        }
        None
    }
    
    /// Get package by name with specific version
    pub fn get_package_version(&self, name: &str, version: &str) -> Option<NodeIndex> {
        let id = format!("{}@{}", name, version);
        self.index_map.get(&id).copied()
    }
    
    /// Get all versions of a package
    pub fn get_package_versions(&self, name: &str) -> Vec<(NodeIndex, &Package)> {
        let prefix = format!("{}@", name);
        self.index_map
            .iter()
            .filter(|(id, _)| id.starts_with(&prefix))
            .map(|(_, &idx)| (idx, &self.graph[idx]))
            .collect()
    }
    
    pub fn package_count(&self) -> usize {
        self.graph.node_count()
    }
    
    pub fn dependency_count(&self) -> usize {
        self.graph.edge_count()
    }
    
    /// Get direct dependents of a package (packages that directly depend on it)
    pub fn direct_dependents(&self, target: NodeIndex) -> Vec<NodeIndex> {
        use petgraph::Direction;
        self.graph
            .neighbors_directed(target, Direction::Incoming)
            .filter(|&idx| self.graph[idx].is_direct)
            .collect()
    }
}

impl Default for DependencyGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_package_id() {
        let pkg = Package::new("lodash", "4.17.21");
        assert_eq!(pkg.id(), "lodash@4.17.21");
        assert!(!pkg.is_direct);
        assert!(!pkg.is_dev);
    }

    #[test]
    fn test_package_direct() {
        let pkg = Package::direct("lodash", "4.17.21");
        assert!(pkg.is_direct);
    }

    #[test]
    fn test_add_package() {
        let mut graph = DependencyGraph::new();
        let pkg = Package::new("lodash", "4.17.21");
        let idx = graph.add_package(pkg);
        assert_eq!(graph.package_count(), 1);
        
        // Adding same package returns same index
        let pkg2 = Package::new("lodash", "4.17.21");
        let idx2 = graph.add_package(pkg2);
        assert_eq!(idx, idx2);
        assert_eq!(graph.package_count(), 1);
    }

    #[test]
    fn test_add_dependency() {
        let mut graph = DependencyGraph::new();
        let pkg1 = Package::new("myapp", "1.0.0");
        let pkg2 = Package::new("lodash", "4.17.21");
        let idx1 = graph.add_package(pkg1);
        let idx2 = graph.add_package(pkg2);
        
        graph.add_dependency(idx1, idx2, Dependency::default());
        assert_eq!(graph.dependency_count(), 1);
    }

    #[test]
    fn test_get_package_by_name() {
        let mut graph = DependencyGraph::new();
        let pkg = Package::new("lodash", "4.17.21");
        let idx = graph.add_package(pkg);
        
        assert_eq!(graph.get_package("lodash"), Some(idx));
        assert_eq!(graph.get_package("lodash@4.17.21"), Some(idx));
        assert_eq!(graph.get_package("nonexistent"), None);
    }

    #[test]
    fn test_multiple_versions() {
        let mut graph = DependencyGraph::new();
        let pkg1 = Package::new("lodash", "4.17.21");
        let pkg2 = Package::new("lodash", "3.10.0");
        
        let idx1 = graph.add_package(pkg1);
        let idx2 = graph.add_package(pkg2);
        
        assert_ne!(idx1, idx2);
        assert_eq!(graph.package_count(), 2);
    }

    #[test]
    fn test_get_package_versions() {
        let mut graph = DependencyGraph::new();
        graph.add_package(Package::new("lodash", "4.17.21"));
        graph.add_package(Package::new("lodash", "3.10.0"));
        graph.add_package(Package::new("other", "1.0.0"));
        
        let versions = graph.get_package_versions("lodash");
        assert_eq!(versions.len(), 2);
    }

    #[test]
    fn test_dependency_type() {
        let dep = Dependency::dev();
        assert!(dep.is_dev());
        assert!(matches!(dep.dep_type, DependencyType::Dev));
        
        let dep = Dependency::runtime();
        assert!(!dep.is_dev());
    }

    #[test]
    fn test_graph_with_name() {
        let graph = DependencyGraph::with_name("my-project");
        assert_eq!(graph.project_name, "my-project");
    }
}
