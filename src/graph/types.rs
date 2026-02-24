use petgraph::graph::{DiGraph, NodeIndex};
use std::collections::HashMap;

/// A package in the dependency graph
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Package {
    pub name: String,
    pub version: String,
}

impl Package {
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
        }
    }
    
    pub fn id(&self) -> String {
        format!("{}@{}", self.name, self.version)
    }
}

/// A dependency edge between packages
#[derive(Debug, Clone)]
pub struct Dependency {
    pub version_constraint: String,
    pub is_dev: bool,
    pub is_optional: bool,
}

impl Default for Dependency {
    fn default() -> Self {
        Self {
            version_constraint: "*".to_string(),
            is_dev: false,
            is_optional: false,
        }
    }
}

/// The dependency graph
pub struct DependencyGraph {
    pub graph: DiGraph<Package, Dependency>,
    pub index_map: HashMap<String, NodeIndex>,
    pub root_packages: Vec<NodeIndex>,
}

impl DependencyGraph {
    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
            index_map: HashMap::new(),
            root_packages: Vec::new(),
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
    
    pub fn package_count(&self) -> usize {
        self.graph.node_count()
    }
    
    pub fn dependency_count(&self) -> usize {
        self.graph.edge_count()
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
}
