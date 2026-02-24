use super::DependencyGraph;
use petgraph::graph::NodeIndex;
use petgraph::visit::EdgeRef;
use std::collections::{HashSet, VecDeque};

/// A path through the dependency graph
#[derive(Debug, Clone)]
pub struct DependencyPath {
    pub nodes: Vec<NodeIndex>,
}

impl DependencyPath {
    pub fn new(nodes: Vec<NodeIndex>) -> Self {
        Self { nodes }
    }
    
    pub fn len(&self) -> usize {
        self.nodes.len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
}

/// Finds paths through the dependency graph
pub struct PathFinder<'a> {
    graph: &'a DependencyGraph,
    max_depth: usize,
}

impl<'a> PathFinder<'a> {
    pub fn new(graph: &'a DependencyGraph, max_depth: usize) -> Self {
        Self { graph, max_depth }
    }
    
    /// Find shortest path from any root to target
    pub fn find_shortest(&self, target: NodeIndex) -> Option<DependencyPath> {
        let paths = self.find_all(target);
        paths.into_iter().min_by_key(|p| p.len())
    }
    
    /// Find all paths from roots to target
    pub fn find_all(&self, target: NodeIndex) -> Vec<DependencyPath> {
        let mut all_paths = Vec::new();
        
        for &root in &self.graph.root_packages {
            let paths = self.find_paths_from(root, target);
            all_paths.extend(paths);
        }
        
        all_paths
    }
    
    /// Find all paths from a specific start node to target using DFS
    fn find_paths_from(&self, start: NodeIndex, target: NodeIndex) -> Vec<DependencyPath> {
        let mut paths = Vec::new();
        let mut current_path = vec![start];
        let mut visited = HashSet::new();
        
        self.dfs_paths(start, target, &mut current_path, &mut visited, &mut paths);
        
        paths
    }
    
    fn dfs_paths(
        &self,
        current: NodeIndex,
        target: NodeIndex,
        path: &mut Vec<NodeIndex>,
        visited: &mut HashSet<NodeIndex>,
        results: &mut Vec<DependencyPath>,
    ) {
        if path.len() > self.max_depth {
            return;
        }
        
        if current == target {
            results.push(DependencyPath::new(path.clone()));
            return;
        }
        
        visited.insert(current);
        
        for edge in self.graph.graph.edges(current) {
            let next = edge.target();
            if !visited.contains(&next) {
                path.push(next);
                self.dfs_paths(next, target, path, visited, results);
                path.pop();
            }
        }
        
        visited.remove(&current);
    }
    
    /// BFS to find if target is reachable (faster than finding all paths)
    pub fn is_reachable(&self, target: NodeIndex) -> bool {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        
        for &root in &self.graph.root_packages {
            queue.push_back(root);
            visited.insert(root);
        }
        
        while let Some(current) = queue.pop_front() {
            if current == target {
                return true;
            }
            
            for edge in self.graph.graph.edges(current) {
                let next = edge.target();
                if !visited.contains(&next) {
                    visited.insert(next);
                    queue.push_back(next);
                }
            }
        }
        
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::GraphBuilder;

    fn simple_chain() -> DependencyGraph {
        // root -> a -> b -> target
        let mut builder = GraphBuilder::new();
        let root = builder.add_root("root", "1.0.0");
        let a = builder.add_package("a", "1.0.0");
        let b = builder.add_package("b", "1.0.0");
        let target = builder.add_package("target", "1.0.0");
        
        builder.add_dep(root, a);
        builder.add_dep(a, b);
        builder.add_dep(b, target);
        
        builder.build()
    }

    fn diamond_graph() -> DependencyGraph {
        // root -> a -> target
        // root -> b -> target
        let mut builder = GraphBuilder::new();
        let root = builder.add_root("root", "1.0.0");
        let a = builder.add_package("a", "1.0.0");
        let b = builder.add_package("b", "1.0.0");
        let target = builder.add_package("target", "1.0.0");
        
        builder.add_dep(root, a);
        builder.add_dep(root, b);
        builder.add_dep(a, target);
        builder.add_dep(b, target);
        
        builder.build()
    }

    #[test]
    fn test_find_shortest_chain() {
        let graph = simple_chain();
        let target = graph.get_package("target").unwrap();
        let finder = PathFinder::new(&graph, 20);
        
        let path = finder.find_shortest(target).unwrap();
        assert_eq!(path.len(), 4); // root -> a -> b -> target
    }

    #[test]
    fn test_find_all_diamond() {
        let graph = diamond_graph();
        let target = graph.get_package("target").unwrap();
        let finder = PathFinder::new(&graph, 20);
        
        let paths = finder.find_all(target);
        assert_eq!(paths.len(), 2); // Two paths: via a and via b
    }

    #[test]
    fn test_is_reachable() {
        let graph = simple_chain();
        let target = graph.get_package("target").unwrap();
        let finder = PathFinder::new(&graph, 20);
        
        assert!(finder.is_reachable(target));
    }

    #[test]
    fn test_not_reachable() {
        let mut builder = GraphBuilder::new();
        builder.add_root("root", "1.0.0");
        let orphan = builder.add_package("orphan", "1.0.0");
        let graph = builder.build();
        
        let finder = PathFinder::new(&graph, 20);
        assert!(!finder.is_reachable(orphan));
    }

    #[test]
    fn test_max_depth_limit() {
        let graph = simple_chain();
        let target = graph.get_package("target").unwrap();
        let finder = PathFinder::new(&graph, 2); // Too shallow
        
        let paths = finder.find_all(target);
        assert!(paths.is_empty());
    }

    #[test]
    fn test_path_length() {
        let path = DependencyPath::new(vec![]);
        assert!(path.is_empty());
        
        let path = DependencyPath::new(vec![NodeIndex::new(0), NodeIndex::new(1)]);
        assert_eq!(path.len(), 2);
        assert!(!path.is_empty());
    }
}
