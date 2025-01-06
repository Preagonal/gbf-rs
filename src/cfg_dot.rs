#![deny(missing_docs)]

use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::{EdgeRef, IntoNodeReferences};

use crate::utils::{GBF_DARK_GRAY, GBF_LIGHT_GRAY};

/// A trait that defines how a node and its edges are rendered.
pub trait RenderableNode {
    /// Renders the node as a Graphviz label.
    fn render_node(&self, padding: usize) -> String;
}

/// Trait for resolving NodeIndex to renderable metadata.
pub trait NodeResolver {
    /// The renderable node type associated with the resolver.
    type NodeData: RenderableNode;

    /// Resolves a NodeIndex to its associated metadata.
    fn resolve(&self, node_index: NodeIndex) -> Option<&Self::NodeData>;

    /// Resolves the color of the edge between two nodes.
    fn resolve_edge_color(&self, source: NodeIndex, target: NodeIndex) -> String;
}

/// Trait to print the graph in DOT format. The must also implement `NodeResolver`.
pub trait DotRenderableGraph: NodeResolver {
    /// Renders the graph in DOT format.
    fn render_dot(&self, config: CfgDotConfig) -> String;
}

/// Configuration options for rendering a DOT graph.
#[derive(Debug)]
pub struct CfgDotConfig {
    /// The direction of the graph layout.
    pub rankdir: String,
    /// The color of the edges.
    pub edge_color: String,
    /// The shape of the nodes.
    pub node_shape: String,
    /// The font name of the nodes.
    pub fontname: String,
    /// The font size of the nodes.
    pub fontsize: String,
    /// The background color of the graph.
    pub bgcolor: String,
    /// The fill color of the nodes.
    pub fillcolor: String,
}

impl Default for CfgDotConfig {
    fn default() -> Self {
        Self {
            rankdir: "TB".to_string(),
            edge_color: "#ffffff".to_string(),
            node_shape: "plaintext".to_string(),
            fontname: "Courier".to_string(),
            fontsize: "12".to_string(),
            bgcolor: GBF_DARK_GRAY.to_string(),
            fillcolor: GBF_LIGHT_GRAY.to_string(),
        }
    }
}

/// A builder for `CfgDot` instances.
pub struct CfgDotBuilder {
    config: CfgDotConfig,
}

impl CfgDotBuilder {
    /// Creates a new `CfgDotBuilder` with default configuration.
    pub fn new() -> Self {
        Self {
            config: CfgDotConfig::default(),
        }
    }

    /// Sets the direction of the graph layout.
    pub fn rankdir(mut self, rankdir: &str) -> Self {
        self.config.rankdir = rankdir.to_string();
        self
    }

    /// Sets the color of the edges.
    pub fn edge_color(mut self, edge_color: &str) -> Self {
        self.config.edge_color = edge_color.to_string();
        self
    }

    /// Sets the shape of the nodes.
    pub fn node_shape(mut self, node_shape: &str) -> Self {
        self.config.node_shape = node_shape.to_string();
        self
    }

    /// Sets the font name of the nodes.
    pub fn fontname(mut self, fontname: &str) -> Self {
        self.config.fontname = fontname.to_string();
        self
    }

    /// Sets the font size of the nodes.
    pub fn fontsize(mut self, fontsize: &str) -> Self {
        self.config.fontsize = fontsize.to_string();
        self
    }

    /// Sets the background color of the graph.
    pub fn bgcolor(mut self, bgcolor: &str) -> Self {
        self.config.bgcolor = bgcolor.to_string();
        self
    }

    /// Sets the fill color of the nodes.
    pub fn fillcolor(mut self, fillcolor: &str) -> Self {
        self.config.fillcolor = fillcolor.to_string();
        self
    }

    /// Builds the `CfgDot` instance.
    pub fn build(self) -> CfgDot {
        CfgDot {
            config: self.config,
        }
    }
}

/// The main struct for rendering DOT graphs.
pub struct CfgDot {
    /// The configuration for rendering the graph.
    pub config: CfgDotConfig,
}

impl CfgDot {
    /// Renders the DOT representation of a `DiGraph` using the provided resolver.
    ///
    /// This method:
    /// - Defines a directed graph (`digraph CFG`).
    /// - Applies graph-level and node-level styles from `self.config`.
    /// - Iterates over each node in the graph, resolving it via `resolver`.
    /// - Calculates the number of incoming edges for each node to create "ports" for the edges.
    /// - Constructs an HTML-like table label for each node with indentation to make it readable.
    /// - Iterates over all edges and connects them to the correct node ports.
    ///
    /// The `data.render_node(8)` call uses an indentation of 8 spaces for the node's content.
    ///
    /// # Type Parameters
    ///
    /// * `R` - A type that implements `NodeResolver`.
    /// * `N` - Node weight type of the `DiGraph`.
    /// * `E` - Edge weight type of the `DiGraph`.
    ///
    /// # Arguments
    ///
    /// * `graph` - The directed graph to render.
    /// * `resolver` - An object that resolves each node index to a data structure that can be rendered.
    ///
    /// # Returns
    ///
    /// A `String` containing the entire DOT (Graphviz) representation of the graph.
    pub fn render<R, N, E>(&self, graph: &DiGraph<N, E>, resolver: &R) -> String
    where
        R: NodeResolver,
    {
        // Prepare a buffer for the DOT output.
        let mut dot = String::new();

        // Start graph definition.
        dot.push_str("digraph CFG {\n");
        dot.push_str(&format!(
            "    graph [rankdir={}, bgcolor=\"{}\", splines=\"ortho\"];\n",
            self.config.rankdir, self.config.bgcolor
        ));
        dot.push_str(&format!(
            "    edge [color=\"{}\"]; \n",
            self.config.edge_color
        ));
        dot.push_str(&format!(
            "    node [shape=\"{}\", fontname=\"{}\", fontsize=\"{}\"]; \n",
            self.config.node_shape, self.config.fontname, self.config.fontsize
        ));

        // Iterate over each node in the graph.
        for (node_index, _node_data) in graph.node_references() {
            // Attempt to resolve the node data. If it's `None`, skip it.
            if let Some(data) = resolver.resolve(node_index) {
                // Start building up the node's DOT string line-by-line.
                dot.push_str(&format!(
                    "    N{} [style=filled, fillcolor=\"{}\", label=<\n{}\n    >];\n",
                    node_index.index(),
                    self.config.fillcolor,
                    data.render_node(8)
                ));
            }
        }

        // Render each edge.
        for edge in graph.edge_references() {
            let source = edge.source();
            let target = edge.target();

            // Only render if both source and target are resolvable.
            if resolver.resolve(source).is_some() && resolver.resolve(target).is_some() {
                let edge_color = resolver.resolve_edge_color(source, target);

                // Connect source -> target:port with the specified edge color.
                dot.push_str(&format!(
                    "    N{} -> N{} [color=\"{}\"]; \n",
                    source.index(),
                    target.index(),
                    edge_color
                ));
            }
        }

        //
        // 4. Close the graph definition
        //
        dot.push_str("}\n");

        dot
    }
}

// == Implementations ==
impl Default for CfgDotBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use petgraph::graph::{DiGraph, NodeIndex};
    use std::collections::HashMap;

    /// Mock RenderableNode for testing purposes.
    struct MockNode {
        label: String,
    }

    impl RenderableNode for MockNode {
        fn render_node(&self, padding: usize) -> String {
            format!("{}{}", " ".repeat(padding), self.label)
        }
    }

    /// Mock NodeResolver for testing purposes.
    struct MockResolver {
        nodes: HashMap<NodeIndex, MockNode>,
    }

    impl NodeResolver for MockResolver {
        type NodeData = MockNode;

        fn resolve(&self, node_index: NodeIndex) -> Option<&Self::NodeData> {
            self.nodes.get(&node_index)
        }

        fn resolve_edge_color(&self, _source: NodeIndex, _target: NodeIndex) -> String {
            "#ff0000".to_string()
        }
    }

    #[test]
    fn test_cfgdot_default_render() {
        // Create a simple graph.
        let mut graph = DiGraph::new();
        let a = graph.add_node(());
        let b = graph.add_node(());
        graph.add_edge(a, b, ());

        // Create a resolver with mock nodes.
        let resolver = MockResolver {
            nodes: vec![
                (
                    a,
                    MockNode {
                        label: "Node A".to_string(),
                    },
                ),
                (
                    b,
                    MockNode {
                        label: "Node B".to_string(),
                    },
                ),
            ]
            .into_iter()
            .collect(),
        };

        // Render the graph with the default configuration.
        let cfg_dot = CfgDotBuilder::new().build();
        let dot_output = cfg_dot.render(&graph, &resolver);

        // Verify the output.
        assert!(dot_output.contains("digraph CFG {"));
        assert!(dot_output.contains("graph [rankdir=TB"));
        assert!(dot_output.contains(&format!(
            "N0 [style=filled, fillcolor=\"{}\"",
            GBF_LIGHT_GRAY
        )));
        assert!(dot_output.contains("Node A"));
        assert!(dot_output.contains("Node B"));
        assert!(dot_output.contains("N0 -> N1"));
    }

    #[test]
    fn test_cfgdot_custom_config() {
        // Create a simple graph.
        let mut graph = DiGraph::new();
        let a = graph.add_node(());
        let b = graph.add_node(());
        graph.add_edge(a, b, ());

        // Create a resolver with mock nodes.
        let resolver = MockResolver {
            nodes: vec![
                (
                    a,
                    MockNode {
                        label: "Node A".to_string(),
                    },
                ),
                (
                    b,
                    MockNode {
                        label: "Node B".to_string(),
                    },
                ),
            ]
            .into_iter()
            .collect(),
        };

        // Render the graph with a custom configuration.
        let cfg_dot = CfgDotBuilder::new()
            .rankdir("LR")
            .edge_color("#ff0000")
            .node_shape("record")
            .fontname("Arial")
            .fontsize("14")
            .bgcolor("#ffffff")
            .fillcolor("#ff0000")
            .build();
        let dot_output = cfg_dot.render(&graph, &resolver);

        // Verify the output.
        assert!(dot_output.contains("digraph CFG {"));
        assert!(dot_output.contains("graph [rankdir=LR"));
        assert!(dot_output.contains("edge [color=\"#ff0000\"]"));
        assert!(dot_output.contains("node [shape=\"record\", fontname=\"Arial\", fontsize=\"14\"]"));
        assert!(dot_output.contains("bgcolor=\"#ffffff\""));
        assert!(dot_output.contains("N0 [style=filled, fillcolor=\"#ff0000\""));
        assert!(dot_output.contains("Node A"));
        assert!(dot_output.contains("Node B"));
        assert!(dot_output.contains("N0 -> N1"));
    }

    #[test]
    fn test_cfgdot_default_config() {
        // Create a simple graph.
        let mut graph = DiGraph::new();
        let a = graph.add_node(());
        let b = graph.add_node(());
        graph.add_edge(a, b, ());

        // Create a resolver with mock nodes.
        let resolver = MockResolver {
            nodes: vec![
                (
                    a,
                    MockNode {
                        label: "Node A".to_string(),
                    },
                ),
                (
                    b,
                    MockNode {
                        label: "Node B".to_string(),
                    },
                ),
            ]
            .into_iter()
            .collect(),
        };

        // Render the graph with the default configuration.
        let cfg_dot = CfgDotBuilder::default().build();
        let dot_output = cfg_dot.render(&graph, &resolver);

        // Verify the output.
        assert!(dot_output.contains("digraph CFG {"));
        assert!(dot_output.contains("graph [rankdir=TB"));
        assert!(dot_output.contains(&format!(
            "N0 [style=filled, fillcolor=\"{}\"",
            GBF_LIGHT_GRAY
        )));
        assert!(dot_output.contains("Node A"));
        assert!(dot_output.contains("Node B"));
        assert!(dot_output.contains("N0 -> N1"));
    }

    #[test]
    fn test_cfgdot_no_nodes() {
        // Create an empty graph.
        let graph: DiGraph<(), ()> = DiGraph::new();

        // Create an empty resolver.
        let resolver = MockResolver {
            nodes: HashMap::new(),
        };

        // Render the graph.
        let cfg_dot = CfgDotBuilder::new().build();
        let dot_output = cfg_dot.render(&graph, &resolver);

        // Verify the output.
        assert!(dot_output.contains("digraph CFG {"));
        assert!(dot_output.contains("}")); // Ensure proper closure.
        assert!(!dot_output.contains("N0")); // No nodes should be rendered.
    }

    #[test]
    fn test_cfgdot_multiple_edges() {
        // Create a graph with multiple edges.
        let mut graph = DiGraph::new();
        let a = graph.add_node(());
        let b = graph.add_node(());
        let c = graph.add_node(());
        graph.add_edge(a, b, ());
        graph.add_edge(b, c, ());
        graph.add_edge(a, c, ());

        // Create a resolver with mock nodes.
        let resolver = MockResolver {
            nodes: vec![
                (
                    a,
                    MockNode {
                        label: "Node A".to_string(),
                    },
                ),
                (
                    b,
                    MockNode {
                        label: "Node B".to_string(),
                    },
                ),
                (
                    c,
                    MockNode {
                        label: "Node C".to_string(),
                    },
                ),
            ]
            .into_iter()
            .collect(),
        };

        // Render the graph.
        let cfg_dot = CfgDotBuilder::new().build();
        let dot_output = cfg_dot.render(&graph, &resolver);

        // Verify the output.
        println!("{}", dot_output);
        assert!(dot_output.contains("N0 -> N1"));
        assert!(dot_output.contains("N1 -> N2"));
        assert!(dot_output.contains("N0 -> N2"));
    }

    // case where resolver returns None for a node
    #[test]
    fn test_cfgdot_missing_node() {
        // Create a simple graph.
        let mut graph = DiGraph::new();
        let a = graph.add_node(());
        let b = graph.add_node(());
        graph.add_edge(a, b, ());

        // Create a resolver with a missing node.
        let resolver = MockResolver {
            nodes: vec![(
                a,
                MockNode {
                    label: "Node A".to_string(),
                },
            )]
            .into_iter()
            .collect(),
        };

        // Render the graph.
        let cfg_dot = CfgDotBuilder::new().build();
        let dot_output = cfg_dot.render(&graph, &resolver);

        // Verify the output.
        assert!(dot_output.contains(&format!(
            "N0 [style=filled, fillcolor=\"{}\"",
            GBF_LIGHT_GRAY
        )));
        assert!(!dot_output.contains("N1")); // Node B should not be rendered.
    }
}
