use super::{DependencyGraph, DependencyType};
use petgraph::graph::NodeIndex;
use petgraph::visit::EdgeRef;
use std::collections::{HashSet, VecDeque};

/// A path through the dependency graph
#[derive(Debug, Clone)]
pub struct DependencyPath {
    pub nodes: Vec<NodeIndex>,
    pub dep_types: Vec<DependencyType>,
}

impl DependencyPath {
    pub fn new(nodes: Vec<NodeIndex>) -> Self {
        Self {
            dep_types: vec![DependencyType::Runtime; nodes.len().saturating_sub(1)],
            nodes,
        }
    }
    
    pub fn with_types(nodes: Vec<NodeIndex>, dep_types: Vec<DependencyType>) -> Self {
        Self { nodes, dep_types }
    }
    
    pub fn len(&self) -> usize {
        self.nodes.len()
    }
    
    pub fn depth(&self) -> usize {
        self.nodes.len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
    
    /// Check if any edge in the path is a dev dependency
    pub fn is_dev(&self) -> bool {
        self.dep_types.iter().any(|t| matches!(t, DependencyType::Dev))
    }
    
    /// Get the direct dependency that starts this path (second node, after root)
    pub fn direct_dependent(&self) -> Option<NodeIndex> {
        // Path is [root, direct_dep, ..., target]
        // We want the direct_dep (index 1), not root (index 0)
        self.nodes.get(1).copied()
    }
}

/// Result of a dependency query (per spec Section 3.2)
#[derive(Debug)]
pub struct QueryResult {
    pub target: NodeIndex,
    pub target_name: String,
    pub target_version: String,
    pub paths: Vec<DependencyPath>,
    pub shortest_depth: usize,
    pub longest_depth: usize,
    pub direct_dependents: Vec<NodeIndex>,
}

impl QueryResult {
    pub fn new(graph: &DependencyGraph, target: NodeIndex, paths: Vec<DependencyPath>) -> Self {
        let pkg = &graph.graph[target];
        // Per spec: depth counts from direct dependency, not root
        // So depth = path.len() - 1 (excluding root node)
        let shortest = paths.iter().map(|p| p.depth().saturating_sub(1)).min().unwrap_or(0);
        let longest = paths.iter().map(|p| p.depth().saturating_sub(1)).max().unwrap_or(0);
        
        // Collect unique direct dependents (first node of each path)
        let mut direct_set: HashSet<NodeIndex> = HashSet::new();
        for path in &paths {
            if let Some(first) = path.direct_dependent() {
                direct_set.insert(first);
            }
        }
        let direct_dependents: Vec<NodeIndex> = direct_set.into_iter().collect();
        
        Self {
            target,
            target_name: pkg.name.clone(),
            target_version: pkg.version.clone(),
            paths,
            shortest_depth: shortest,
            longest_depth: longest,
            direct_dependents,
        }
    }
    
    pub fn total_paths(&self) -> usize {
        self.paths.len()
    }
}

/// Options for path finding
#[derive(Debug, Clone)]
pub struct SearchOptions {
    pub max_depth: usize,
    pub max_paths: usize,
    pub include_dev: bool,
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            max_depth: 20,
            max_paths: 5,
            include_dev: false,
        }
    }
}

/// Finds paths through the dependency graph
pub struct PathFinder<'a> {
    graph: &'a DependencyGraph,
    options: SearchOptions,
}

impl<'a> PathFinder<'a> {
    pub fn new(graph: &'a DependencyGraph, max_depth: usize) -> Self {
        Self {
            graph,
            options: SearchOptions {
                max_depth,
                ..Default::default()
            },
        }
    }
    
    pub fn with_options(graph: &'a DependencyGraph, options: SearchOptions) -> Self {
        Self { graph, options }
    }
    
    /// Find shortest path from any root to target
    pub fn find_shortest(&self, target: NodeIndex) -> Option<DependencyPath> {
        let paths = self.find_all(target);
        paths.into_iter().min_by_key(|p| p.len())
    }
    
    /// Find all paths from roots to target (up to max_paths)
    pub fn find_all(&self, target: NodeIndex) -> Vec<DependencyPath> {
        let mut all_paths = Vec::new();
        
        for &root in &self.graph.root_packages {
            let paths = self.find_paths_from(root, target);
            all_paths.extend(paths);
            
            if all_paths.len() >= self.options.max_paths && self.options.max_paths > 0 {
                all_paths.truncate(self.options.max_paths);
                break;
            }
        }
        
        all_paths
    }
    
    /// Find all paths, ignoring max_paths limit
    pub fn find_all_unlimited(&self, target: NodeIndex) -> Vec<DependencyPath> {
        let mut all_paths = Vec::new();
        
        for &root in &self.graph.root_packages {
            let paths = self.find_paths_from(root, target);
            all_paths.extend(paths);
        }
        
        all_paths
    }
    
    /// Build a QueryResult for the target
    pub fn query(&self, target: NodeIndex) -> QueryResult {
        let paths = self.find_all_unlimited(target);
        QueryResult::new(self.graph, target, paths)
    }
    
    /// Find all paths from a specific start node to target using DFS
    fn find_paths_from(&self, start: NodeIndex, target: NodeIndex) -> Vec<DependencyPath> {
        let mut paths = Vec::new();
        let mut current_path = vec![start];
        let mut current_types = Vec::new();
        let mut visited = HashSet::new();
        
        self.dfs_paths(start, target, &mut current_path, &mut current_types, &mut visited, &mut paths);
        
        paths
    }
    
    fn dfs_paths(
        &self,
        current: NodeIndex,
        target: NodeIndex,
        path: &mut Vec<NodeIndex>,
        types: &mut Vec<DependencyType>,
        visited: &mut HashSet<NodeIndex>,
        results: &mut Vec<DependencyPath>,
    ) {
        if path.len() > self.options.max_depth {
            return;
        }
        
        if current == target {
            results.push(DependencyPath::with_types(path.clone(), types.clone()));
            return;
        }
        
        visited.insert(current);
        
        for edge in self.graph.graph.edges(current) {
            let dep = edge.weight();
            
            // Skip dev dependencies unless include_dev is set
            if dep.is_dev() && !self.options.include_dev {
                continue;
            }
            
            let next = edge.target();
            if !visited.contains(&next) {
                path.push(next);
                types.push(dep.dep_type);
                self.dfs_paths(next, target, path, types, visited, results);
                path.pop();
                types.pop();
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
                let dep = edge.weight();
                
                // Skip dev dependencies unless include_dev
                if dep.is_dev() && !self.options.include_dev {
                    continue;
                }
                
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
    
    fn dev_dep_graph() -> DependencyGraph {
        // root -> a (runtime) -> target
        // root -> b (dev) -> target
        let mut builder = GraphBuilder::new();
        let root = builder.add_root("root", "1.0.0");
        let a = builder.add_package("a", "1.0.0");
        let b = builder.add_package("b", "1.0.0");
        let target = builder.add_package("target", "1.0.0");
        
        builder.add_dep(root, a);
        builder.add_dev_dep(root, b);
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
        
        let paths = finder.find_all_unlimited(target);
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
    
    #[test]
    fn test_dev_deps_excluded_by_default() {
        let graph = dev_dep_graph();
        let target = graph.get_package("target").unwrap();
        let finder = PathFinder::new(&graph, 20);
        
        // Should only find path via 'a' (runtime), not via 'b' (dev)
        let paths = finder.find_all_unlimited(target);
        assert_eq!(paths.len(), 1);
    }
    
    #[test]
    fn test_dev_deps_included_with_option() {
        let graph = dev_dep_graph();
        let target = graph.get_package("target").unwrap();
        let options = SearchOptions {
            max_depth: 20,
            max_paths: 0,
            include_dev: true,
        };
        let finder = PathFinder::with_options(&graph, options);
        
        // Should find both paths
        let paths = finder.find_all_unlimited(target);
        assert_eq!(paths.len(), 2);
    }
    
    #[test]
    fn test_query_result() {
        let graph = diamond_graph();
        let target = graph.get_package("target").unwrap();
        let finder = PathFinder::new(&graph, 20);
        
        let result = finder.query(target);
        
        assert_eq!(result.target_name, "target");
        assert_eq!(result.target_version, "1.0.0");
        assert_eq!(result.total_paths(), 2);
        // Paths are [root, a, target] and [root, b, target] - 3 nodes each
        // Per spec: depth excludes root, so depth is 2 (a->target or b->target)
        assert_eq!(result.shortest_depth, 2);
        assert_eq!(result.longest_depth, 2);
        assert_eq!(result.direct_dependents.len(), 2); // a and b
    }
    
    #[test]
    fn test_path_is_dev() {
        let path = DependencyPath::with_types(
            vec![NodeIndex::new(0), NodeIndex::new(1)],
            vec![DependencyType::Dev],
        );
        assert!(path.is_dev());
        
        let path = DependencyPath::with_types(
            vec![NodeIndex::new(0), NodeIndex::new(1)],
            vec![DependencyType::Runtime],
        );
        assert!(!path.is_dev());
    }
}
