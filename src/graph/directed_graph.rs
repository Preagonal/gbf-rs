use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Error type for Graph operations
/// 
/// # Variants
/// - `NodeNotFound`: The requested node was not found.
/// - `EdgeAlreadyExists`: An edge already exists between the two nodes.
/// - `SelfLoop`: A self-loop was detected.
#[derive(Error, Debug)]
pub enum GraphError {
    #[error("Node not found: {0}")]
    NodeNotFound(String),
    #[error("Self-loop detected for NodeId({0})")]
    SelfLoop(usize),
    #[error("Edge already exists between NodeId({0}) and NodeId({1})")]
    EdgeAlreadyExists(usize, usize),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Ord)]
pub struct NodeId(usize);

// Implement the `PartialOrd` trait for `NodeId` to allow sorting for test cases
impl PartialOrd for NodeId {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Represents a node in the directed graph.
#[derive(Debug, Serialize, Deserialize)]
pub struct Node<T> {
    value: T,
    successors: HashSet<NodeId>,   // Indices of successor nodes
    predecessors: HashSet<NodeId>, // Indices of predecessor nodes
}

impl<T> Node<T> {
    /// Creates a new node with the given value.
    /// 
    /// # Arguments
    /// - `value`: The value to store in the node.
    /// 
    /// # Returns
    /// A new `Node` instance.
    /// 
    /// # Example
    /// ```
    /// use gbf_rs::graph::directed_graph::Node;
    /// 
    /// let node = Node::new(42);
    /// ```
    pub fn new(value: T) -> Self {
        Self {
            value,
            successors: HashSet::new(),
            predecessors: HashSet::new(),
        }
    }
}

/// A trait that defines how a node and its edges are rendered.
pub trait RenderableNode {
    /// Renders the node as a Graphviz label.
    fn render_node(&self, padding: usize) -> String;
}

/// Trait for resolving NodeId to renderable metadata.
pub trait NodeResolver {
    type NodeData: RenderableNode;

    /// Resolves a NodeId to its associated metadata.
    fn resolve(&self, node_id: NodeId) -> Option<&Self::NodeData>;
}

/// Represents a directed graph.
#[derive(Debug)]
pub struct DirectedGraph<T> {
    nodes: HashMap<NodeId, Node<T>>, // Maps NodeId to Node
    node_map: HashMap<T, NodeId>,   // Maps node values to NodeId
    next_id: usize,             // Tracks the next available NodeId
}

impl<T: Eq + std::hash::Hash + Clone + Serialize + for<'de> Deserialize<'de>> DirectedGraph<T> {
    /// Creates a new, empty directed graph.
    /// 
    /// # Returns
    /// A new `DirectedGraph` instance.
    /// 
    /// # Example
    /// ```
    /// use gbf_rs::graph::directed_graph::DirectedGraph;
    /// 
    /// let graph: DirectedGraph<i32> = DirectedGraph::new();
    /// ```
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            node_map: HashMap::new(),
            next_id: 0,
        }
    }

    /// Get the total number of nodes in the graph
    /// 
    /// # Returns
    /// The number of nodes in the graph.
    /// 
    /// # Example
    /// ```
    /// use gbf_rs::graph::directed_graph::DirectedGraph;
    /// 
    /// let mut graph: DirectedGraph<i32> = DirectedGraph::new();
    /// let a = graph.add_node(42);
    /// let b = graph.add_node(43);
    /// let count = graph.node_count();
    /// ```
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Adds a node to the graph. Returns the index of the node.
    /// 
    /// # Arguments
    /// - `value`: The value to store in the node.
    /// 
    /// # Returns
    /// The index of the newly added node.
    /// 
    /// # Example
    /// ```
    /// use gbf_rs::graph::directed_graph::DirectedGraph;
    /// 
    /// let mut graph: DirectedGraph<i32> = DirectedGraph::new();
    /// let node_id = graph.add_node(42);
    /// ```
    pub fn add_node(&mut self, value: T) -> NodeId {
        if let Some(&id) = self.node_map.get(&value) {
            return id;
        }
        let id = NodeId(self.next_id);
        self.next_id += 1;

        self.nodes.insert(id, Node::new(value.clone()));
        self.node_map.insert(value, id);

        id
    }

    /// Adds a directed edge from `from` to `to`.
    /// 
    /// # Arguments
    /// - `from`: The source node.
    /// - `to`: The target node.
    /// 
    /// # Errors
    /// - `GraphError::NodeNotFound` if either node does not exist.
    /// 
    /// # Example
    /// ```
    /// use gbf_rs::graph::directed_graph::DirectedGraph;
    /// 
    /// let mut graph: DirectedGraph<i32> = DirectedGraph::new();
    /// let a = graph.add_node(42);
    /// let b = graph.add_node(43);
    /// graph.add_edge(a, b).unwrap();
    /// ```
    pub fn add_edge(&mut self, from: NodeId, to: NodeId) -> Result<(), GraphError> {
        if from == to {
            return Err(GraphError::SelfLoop(from.0));
        }

        // Check that both nodes exist before modifying the graph
        let from_node_exists = self.nodes.contains_key(&from);
        let to_node_exists = self.nodes.contains_key(&to);

        if !from_node_exists {
            return Err(GraphError::NodeNotFound(format!("NodeId({})", from.0)));
        }
        if !to_node_exists {
            return Err(GraphError::NodeNotFound(format!("NodeId({})", to.0)));
        }

        // Safe to mutate now since we know both nodes exist
        if let Some(from_node) = self.nodes.get_mut(&from) {
            if !from_node.successors.insert(to) {
                return Err(GraphError::EdgeAlreadyExists(from.0, to.0));
            }
        }

        if let Some(to_node) = self.nodes.get_mut(&to) {
            to_node.predecessors.insert(from);
        }

        Ok(())
    }

    /// ```
    /// use gbf_rs::graph::directed_graph::DirectedGraph;
    /// use gbf_rs::graph::directed_graph::{NodeId, NodeResolver, RenderableNode};
    ///
    /// // Define a simple renderable node that returns a fixed label.
    /// struct MyRenderableNode;
    /// impl RenderableNode for MyRenderableNode {
    ///     fn render_node(&self, _padding: usize) -> String {
    ///         "MyNode".to_string() // Example label for each node.
    ///     }
    /// }
    ///
    /// // Define a simple resolver for nodes.
    /// struct SimpleResolver {
    ///     nodes: std::collections::HashMap<NodeId, MyRenderableNode>,
    /// }
    ///
    /// impl NodeResolver for SimpleResolver {
    ///     type NodeData = MyRenderableNode;
    ///
    ///     fn resolve(&self, node_id: NodeId) -> Option<&Self::NodeData> {
    ///         self.nodes.get(&node_id) // Resolve the node by its ID.
    ///     }
    /// }
    ///
    /// // Example usage.
    /// let mut graph: DirectedGraph<i32> = DirectedGraph::new();
    /// let a = graph.add_node(42); // Add a node with the value 42.
    /// let b = graph.add_node(43); // Add a node with the value 43.
    /// graph.add_edge(a, b).unwrap(); // Create an edge between nodes.
    ///
    /// // Create a resolver with the renderable nodes.
    /// let resolver = SimpleResolver {
    ///     nodes: [(a, MyRenderableNode), (b, MyRenderableNode)]
    ///         .into_iter()
    ///         .collect(),
    /// };
    ///
    /// // Generate the DOT output.
    /// let dot = graph.to_dot(&resolver);
    /// println!("{}", dot);
    /// ```
    pub fn to_dot<R>(&self, resolver: &R) -> String
    where
        R: NodeResolver,
    {
        let mut dot = String::new();

        // Start the graph definition.
        dot.push_str("digraph CFG {\n");
        dot.push_str("    graph [rankdir=TB, splines=ortho, overlap=false];\n");
        dot.push_str("    edge [color=\"#333333\", penwidth=\"2\", arrowhead=\"normal\"];\n");
        dot.push_str("    node [shape=\"none\", fontname=\"Courier\", fontsize=\"12\"];\n");

        // Render each node using the resolver.
        for (node_id, _node) in &self.nodes {
            if let Some(data) = resolver.resolve(*node_id) {
                dot.push_str(&format!(
                    "    N{} [shape=plaintext,label=<\n{}    >];\n",
                    node_id.0,      // NodeId
                    data.render_node(8) // Node label with padding
                ));
            }
        }

        // Render edges between nodes.
        for (node_id, node) in &self.nodes {
            for &successor_id in &node.successors {
                dot.push_str(&format!(
                    "    N{} -> N{};\n",
                    node_id.0, successor_id.0 // Edge from source to target
                ));
            }
        }

        // Close the graph definition.
        dot.push_str("}\n");

        dot
    }

    /// Gets the successors of a NodeId.
    /// 
    /// # Arguments
    /// - `value`: The NodeId.
    /// 
    /// # Returns
    /// A vector of references to the successor NodeIds.
    /// 
    /// # Errors
    /// - `GraphError::NodeNotFound` if the node does not exist.
    /// 
    /// # Example
    /// ```
    /// use gbf_rs::graph::directed_graph::DirectedGraph;
    /// 
    /// let mut graph: DirectedGraph<i32> = DirectedGraph::new();
    /// let a = graph.add_node(42);
    /// let b = graph.add_node(43);
    /// graph.add_edge(a, b).unwrap();
    /// let successors = graph.get_successors(a).unwrap();
    /// ```
    pub fn get_successors(&self, node: NodeId) -> Result<Vec<NodeId>, GraphError> {
        let node = self
            .nodes
            .get(&node)
            .ok_or_else(|| GraphError::NodeNotFound(format!("NodeId({})", node.0)))?;
        Ok(node.successors.iter().copied().collect())
    }

    /// Gets the predecessors of a NodeId.
    /// 
    /// # Arguments
    /// - `value`: The value of the NodeId.
    /// 
    /// # Returns
    /// A vector of references to the predecessor NodeIds.
    /// 
    /// # Errors
    /// - `GraphError::NodeNotFound` if the node does not exist.
    /// 
    /// # Example
    /// ```
    /// use gbf_rs::graph::directed_graph::DirectedGraph;
    /// 
    /// let mut graph: DirectedGraph<i32> = DirectedGraph::new();
    /// let a = graph.add_node(42);
    /// let b = graph.add_node(43);
    /// graph.add_edge(a, b).unwrap();
    /// let predecessors = graph.get_predecessors(b).unwrap();
    /// ```
    pub fn get_predecessors(&self, node: NodeId) -> Result<Vec<NodeId>, GraphError> {
        let node = self
            .nodes
            .get(&node)
            .ok_or_else(|| GraphError::NodeNotFound(format!("NodeId({})", node.0)))?;
        Ok(node.predecessors.iter().copied().collect())
    }

    /// Gets the value of a node by its NodeId.
    /// 
    /// # Arguments
    /// - `node`: The NodeId.
    /// 
    /// # Returns
    /// The value of the node.
    /// 
    /// # Errors
    /// - `GraphError::NodeNotFound` if the node does not exist.
    /// 
    /// # Example
    /// ```
    /// use gbf_rs::graph::directed_graph::DirectedGraph;
    /// 
    /// let mut graph: DirectedGraph<i32> = DirectedGraph::new();
    /// let a = graph.add_node(42);
    /// let value = graph.get_node_value(a).unwrap();
    /// ```
    pub fn get_node_value(&self, node: NodeId) -> Result<&T, GraphError> {
        let node = self
            .nodes
            .get(&node)
            .ok_or_else(|| GraphError::NodeNotFound(format!("NodeId({})", node.0)))?;
        Ok(&node.value)
    }

    /// Gets a vector of values for nodes based on their NodeIds.
    /// 
    /// # Arguments
    /// - `nodes`: A vector of NodeIds.
    /// 
    /// # Returns
    /// A vector of values for the given NodeIds.
    /// 
    /// # Errors
    /// - `GraphError::NodeNotFound` if any of the nodes do not exist.
    /// 
    /// # Example
    /// ```
    /// use gbf_rs::graph::directed_graph::DirectedGraph;
    /// 
    /// let mut graph: DirectedGraph<i32> = DirectedGraph::new();
    /// let a = graph.add_node(42);
    /// let b = graph.add_node(43);
    /// let values = graph.get_node_values(vec![a, b]).unwrap();
    /// ```
    pub fn get_node_values(&self, nodes: Vec<NodeId>) -> Result<Vec<T>, GraphError> {
        let mut values = Vec::new();
        for node in nodes {
            let value = self.get_node_value(node)?;
            values.push(value.clone());
        }
        Ok(values)
    }
}

impl<T: Eq + std::hash::Hash + Clone + std::fmt::Debug> DirectedGraph<T> {
    /// Performs a depth-first search (DFS) from the given node.
    /// 
    /// # Arguments
    /// - `start`: The value of the starting node.
    /// 
    /// # Returns
    /// A vector of node ids in DFS order.
    /// 
    /// # Errors
    /// - `GraphError::NodeNotFound` if the starting node does not exist.
    /// 
    /// # Example
    /// ```
    /// use gbf_rs::graph::directed_graph::DirectedGraph;
    /// 
    /// let mut graph: DirectedGraph<i32> = DirectedGraph::new();
    /// let a = graph.add_node(42);
    /// let b = graph.add_node(43);
    /// graph.add_edge(a, b).unwrap();
    /// let result = graph.dfs(a).unwrap();
    /// ```
    pub fn dfs(&self, start: NodeId) -> Result<Vec<NodeId>, GraphError> {
        let mut visited = HashSet::new();
        let mut result = Vec::new();
        self.dfs_util(start, &mut visited, &mut result)?;
        Ok(result)
    }

    /// Helper function for DFS. This is a recursive function that visits each node
    /// and its successors in depth-first order.
    /// 
    /// # Arguments
    /// - `index`: The index of the current node.
    /// - `visited`: A set of visited node indices.
    /// - `result`: The result vector to store node values.
    fn dfs_util(
        &self,
        node: NodeId,
        visited: &mut HashSet<NodeId>,
        result: &mut Vec<NodeId>,
    ) -> Result<(), GraphError> {
        // Skip already visited nodes
        if !visited.insert(node) {
            return Ok(());
        }

        // Ensure the current node exists in the graph
        let current_node = self
            .nodes
            .get(&node)
            .ok_or_else(|| GraphError::NodeNotFound(format!("NodeId({})", node.0)))?;

        // Push the current NodeId to the result
        result.push(node);

        // Sort successors for deterministic results in tests
        #[cfg(test)]
        let successors: Vec<_> = {
            let mut sorted_successors: Vec<_> = current_node.successors.iter().copied().collect();
            sorted_successors.sort();
            sorted_successors
        };

        // Use an iterator for non-test builds
        #[cfg(not(test))]
        let successors = current_node.successors.iter().copied();

        // Recur for each successor
        for successor in successors {
            self.dfs_util(successor, visited, result)?;
        }

        Ok(())
    }

    /// Returns nodes in reverse postorder starting from the given entry node.
    /// This is useful for analyzing and transforming control flow graphs.
    /// 
    /// # Arguments
    /// - `start`: The value of the starting node.
    /// 
    /// # Returns
    /// A vector of node ids in reverse post order.
    /// 
    /// # Errors
    /// - `GraphError::NodeNotFound` if the starting node does not exist.
    /// 
    /// # Example
    /// ```
    /// use gbf_rs::graph::directed_graph::DirectedGraph;
    /// 
    /// let mut graph: DirectedGraph<i32> = DirectedGraph::new();
    /// let a = graph.add_node(42);
    /// let b = graph.add_node(43);
    /// graph.add_edge(a, b).unwrap();
    /// let result = graph.reverse_postorder(a).unwrap();
    /// ```
    pub fn reverse_postorder(&self, start_id: NodeId) -> Result<Vec<NodeId>, GraphError>
    where
        T: Clone,
    {
        // check if the start node exists
        if !self.nodes.contains_key(&start_id) {
            return Err(GraphError::NodeNotFound(format!("NodeId({})", start_id.0)));
        }

        let mut visited = HashSet::new();
        let mut stack = Vec::new();

        // Perform the traversal
        self.reverse_postorder_util(start_id, &mut visited, &mut stack)?;

        // Reverse the stack and collect the NodeIds
        let result: Vec<_> = stack.into_iter().rev().collect();
        Ok(result)
    }

    /// Helper function for reverse postorder traversal.
    /// 
    /// # Arguments
    /// - `node_id`: The id of the current node.
    /// - `visited`: A set of visited node indices.
    /// - `stack`: The stack to store node values in reverse postorder.
    fn reverse_postorder_util(
        &self,
        node_id: NodeId,
        visited: &mut HashSet<NodeId>,
        stack: &mut Vec<NodeId>,
    ) -> Result<(), GraphError> {
        // Skip already visited nodes
        if !visited.insert(node_id) {
            return Ok(());
        }

        // Fetch the current node
        let node = self
            .nodes
            .get(&node_id)
            .ok_or_else(|| GraphError::NodeNotFound(format!("NodeId({})", node_id.0)))?;

        // Sort successors for deterministic results in tests
        #[cfg(test)]
        let successors: Vec<_> = {
            let mut sorted_successors: Vec<_> = node.successors.iter().copied().collect();
            sorted_successors.sort();
            sorted_successors
        };

        // Use an iterator for non-test builds
        #[cfg(not(test))]
        let successors = node.successors.iter().copied();

        // Recur for each successor
        for successor_id in successors {
            self.reverse_postorder_util(successor_id, visited, stack)?;
        }

        // Add the current node to the stack
        stack.push(node_id);

        Ok(())
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_node() {
        let mut graph: DirectedGraph<String> = DirectedGraph::new();
        graph.add_node("a".to_string());
        graph.add_node("b".to_string());
        graph.add_node("c".to_string());
        assert_eq!(graph.node_count(), 3);
    }

    #[test]
    fn test_add_edge() {
        let mut graph: DirectedGraph<String> = DirectedGraph::new();
        let a = graph.add_node("a".to_string());
        let b = graph.add_node("b".to_string());
        let c = graph.add_node("c".to_string());

        graph.add_edge(a, b).unwrap();
        graph.add_edge(b, c).unwrap();

        assert_eq!(graph.get_successors(a).unwrap(), vec![b]);
        assert_eq!(graph.get_predecessors(b).unwrap(), vec![a]);
        assert_eq!(graph.get_successors(b).unwrap(), vec![c]);
        assert_eq!(graph.get_predecessors(c).unwrap(), vec![b]);

        // Test self-loop
        let result = graph.add_edge(a, a);
        assert!(result.is_err());

        // Test invalid node
        let result = graph.add_edge(NodeId(100), NodeId(101));
        assert!(result.is_err());

        // Test edge already exists
        let result = graph.add_edge(a, b);
        assert!(result.is_err());
    }

    #[test]
    fn test_predecessors_successors() {
        let mut graph: DirectedGraph<String> = DirectedGraph::new();
        let a = graph.add_node("a".to_string());
        let b = graph.add_node("b".to_string());
        let c = graph.add_node("c".to_string());

        graph.add_edge(a, b).unwrap();
        graph.add_edge(b, c).unwrap();

        assert_eq!(graph.get_successors(a).unwrap(), vec![b]);
        assert_eq!(graph.get_predecessors(b).unwrap(), vec![a]);
        assert_eq!(graph.get_successors(b).unwrap(), vec![c]);
        assert_eq!(graph.get_predecessors(c).unwrap(), vec![b]);

        // test invalid node
        let result = graph.get_successors(NodeId(100));
        assert!(result.is_err());
        let result = graph.get_predecessors(NodeId(100));
        assert!(result.is_err());
    }

    #[test]
    fn test_get_node_value() {
        let mut graph: DirectedGraph<String> = DirectedGraph::new();
        let a = graph.add_node("a".to_string());
        let b = graph.add_node("b".to_string());
        let c = graph.add_node("c".to_string());

        assert_eq!(graph.get_node_value(a).unwrap(), "a");
        assert_eq!(graph.get_node_value(b).unwrap(), "b");
        assert_eq!(graph.get_node_value(c).unwrap(), "c");

        // Test invalid node
        let result = graph.get_node_value(NodeId(100));
        assert!(result.is_err());
    }

    #[test]
    fn test_get_node_values() {
        let mut graph: DirectedGraph<String> = DirectedGraph::new();
        let a = graph.add_node("a".to_string());
        let b = graph.add_node("b".to_string());
        let c = graph.add_node("c".to_string());

        let values = graph.get_node_values(vec![a, b, c]).unwrap();
        assert_eq!(values, vec!["a", "b", "c"]);

        // Test invalid node
        let result = graph.get_node_values(vec![NodeId(100)]);
        assert!(result.is_err());
    }

    #[test]
    fn test_dfs() {
        // verified using https://graphonline.top/en/
        let mut graph: DirectedGraph<String> = DirectedGraph::new();

        let a = graph.add_node("a".to_string());
        let b = graph.add_node("b".to_string());
        let c = graph.add_node("c".to_string());
        let d = graph.add_node("d".to_string());
        let e = graph.add_node("e".to_string());
        let f = graph.add_node("f".to_string());
        let g = graph.add_node("g".to_string());

        graph.add_edge(a, b).unwrap();
        graph.add_edge(a, c).unwrap();
        graph.add_edge(b, d).unwrap();
        graph.add_edge(b, e).unwrap();
        graph.add_edge(c, f).unwrap();
        graph.add_edge(c, g).unwrap();

        let result = graph.dfs(a).unwrap();
        assert_eq!(result, vec![a, b, d, e, c, f, g]);

        // Test invalid node
        let result = graph.dfs(NodeId(100));
        assert!(result.is_err());
    }

    #[test]
    fn test_reverse_postorder() {
        // verified using https://graphonline.top/en/
        let mut graph: DirectedGraph<String> = DirectedGraph::new();

        let a = graph.add_node("a".to_string());
        let b = graph.add_node("b".to_string());
        let c = graph.add_node("c".to_string());
        let d = graph.add_node("d".to_string());
        let e = graph.add_node("e".to_string());
        let f = graph.add_node("f".to_string());
        let g = graph.add_node("g".to_string());

        graph.add_edge(a, b).unwrap();
        graph.add_edge(a, c).unwrap();
        graph.add_edge(b, d).unwrap();
        graph.add_edge(b, e).unwrap();
        graph.add_edge(c, f).unwrap();
        graph.add_edge(c, g).unwrap();

        let result = graph.reverse_postorder(a).unwrap();
        assert_eq!(result, vec![a, c, g, f, b, e, d]);

        // Test invalid node
        let result = graph.reverse_postorder(NodeId(100));
        assert!(result.is_err());
    }

}
