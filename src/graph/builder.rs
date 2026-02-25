use super::{DependencyGraph, Package, Dependency, DependencyType};
use petgraph::graph::NodeIndex;

/// Builder for constructing dependency graphs
pub struct GraphBuilder {
    graph: DependencyGraph,
}

impl GraphBuilder {
    pub fn new() -> Self {
        Self {
            graph: DependencyGraph::new(),
        }
    }
    
    pub fn with_name(name: impl Into<String>) -> Self {
        Self {
            graph: DependencyGraph::with_name(name),
        }
    }
    
    pub fn add_root(&mut self, name: impl Into<String>, version: impl Into<String>) -> NodeIndex {
        let pkg = Package::direct(name, version);
        let idx = self.graph.add_package(pkg);
        self.graph.root_packages.push(idx);
        idx
    }
    
    pub fn add_package(&mut self, name: impl Into<String>, version: impl Into<String>) -> NodeIndex {
        let pkg = Package::new(name, version);
        self.graph.add_package(pkg)
    }
    
    pub fn add_direct_package(&mut self, name: impl Into<String>, version: impl Into<String>) -> NodeIndex {
        let mut pkg = Package::new(name, version);
        pkg.is_direct = true;
        self.graph.add_package(pkg)
    }
    
    pub fn add_dep(&mut self, from: NodeIndex, to: NodeIndex) {
        self.graph.add_dependency(from, to, Dependency::runtime());
    }
    
    pub fn add_dep_with_constraint(
        &mut self,
        from: NodeIndex,
        to: NodeIndex,
        constraint: impl Into<String>,
    ) {
        self.graph.add_dependency(from, to, Dependency {
            version_constraint: constraint.into(),
            dep_type: DependencyType::Runtime,
        });
    }
    
    pub fn add_dev_dep(&mut self, from: NodeIndex, to: NodeIndex) {
        self.graph.add_dependency(from, to, Dependency::dev());
    }
    
    pub fn add_optional_dep(&mut self, from: NodeIndex, to: NodeIndex) {
        self.graph.add_dependency(from, to, Dependency::optional());
    }
    
    pub fn build(self) -> DependencyGraph {
        self.graph
    }
}

impl Default for GraphBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_new() {
        let builder = GraphBuilder::new();
        let graph = builder.build();
        assert_eq!(graph.package_count(), 0);
    }

    #[test]
    fn test_builder_with_name() {
        let builder = GraphBuilder::with_name("my-app");
        let graph = builder.build();
        assert_eq!(graph.project_name, "my-app");
    }

    #[test]
    fn test_add_root() {
        let mut builder = GraphBuilder::new();
        builder.add_root("myapp", "1.0.0");
        let graph = builder.build();
        
        assert_eq!(graph.package_count(), 1);
        assert_eq!(graph.root_packages.len(), 1);
        
        let root_idx = graph.root_packages[0];
        assert!(graph.graph[root_idx].is_direct);
    }

    #[test]
    fn test_add_direct_package() {
        let mut builder = GraphBuilder::new();
        let idx = builder.add_direct_package("lodash", "4.17.21");
        let graph = builder.build();
        
        assert!(graph.graph[idx].is_direct);
    }

    #[test]
    fn test_build_simple_graph() {
        let mut builder = GraphBuilder::new();
        let root = builder.add_root("myapp", "1.0.0");
        let lodash = builder.add_package("lodash", "4.17.21");
        builder.add_dep(root, lodash);
        
        let graph = builder.build();
        assert_eq!(graph.package_count(), 2);
        assert_eq!(graph.dependency_count(), 1);
    }

    #[test]
    fn test_build_diamond_graph() {
        let mut builder = GraphBuilder::new();
        let root = builder.add_root("app", "1.0.0");
        let a = builder.add_package("dep-a", "1.0.0");
        let b = builder.add_package("dep-b", "1.0.0");
        let shared = builder.add_package("shared", "1.0.0");
        
        builder.add_dep(root, a);
        builder.add_dep(root, b);
        builder.add_dep(a, shared);
        builder.add_dep(b, shared);
        
        let graph = builder.build();
        assert_eq!(graph.package_count(), 4);
        assert_eq!(graph.dependency_count(), 4);
    }

    #[test]
    fn test_dev_dependency() {
        let mut builder = GraphBuilder::new();
        let root = builder.add_root("app", "1.0.0");
        let test_lib = builder.add_package("jest", "28.0.0");
        builder.add_dev_dep(root, test_lib);
        
        let graph = builder.build();
        assert_eq!(graph.dependency_count(), 1);
        
        // Check the edge is marked as dev
        let edge = graph.graph.edges(root).next().unwrap();
        assert!(edge.weight().is_dev());
    }
}
