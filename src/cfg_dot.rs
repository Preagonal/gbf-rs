#![deny(missing_docs)]

use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::{EdgeRef, IntoNodeReferences};

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
}

/// Configuration options for rendering a DOT graph.
#[derive(Debug)]
pub struct CfgDotConfig {
    /// The direction of the graph layout.
    pub rankdir: String,
    /// The type of splines to use for edges.
    pub splines: String,
    /// Whether to allow node overlap.
    pub overlap: String,
    /// The color of the edges.
    pub edge_color: String,
    /// The arrowhead style of the edges.
    pub arrowhead: String,
    /// The shape of the nodes.
    pub node_shape: String,
    /// The font name of the nodes.
    pub fontname: String,
    /// The font size of the nodes.
    pub fontsize: String,
}

impl Default for CfgDotConfig {
    fn default() -> Self {
        Self {
            rankdir: "TB".to_string(),
            splines: "ortho".to_string(),
            overlap: "false".to_string(),
            edge_color: "#ffffff".to_string(),
            arrowhead: "normal".to_string(),
            node_shape: "none".to_string(),
            fontname: "Courier".to_string(),
            fontsize: "12".to_string(),
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

    /// Builds the `CfgDot` instance.
    pub fn build(self) -> CfgDot {
        CfgDot {
            config: self.config,
        }
    }
}

/// The main struct for rendering DOT graphs.
pub struct CfgDot {
    config: CfgDotConfig,
}

impl CfgDot {
    /// Renders the DOT representation of a `DiGraph` using the provided resolver.
    pub fn render<R, N, E>(&self, graph: &DiGraph<N, E>, resolver: &R) -> String
    where
        R: NodeResolver,
    {
        let mut dot = String::new();

        // Start the graph definition.
        dot.push_str("digraph CFG {\n");
        dot.push_str(&format!(
            "    graph [rankdir={}, splines={}, bgcolor=\"0x1c1c1c\", overlap={}];\n",
            self.config.rankdir, self.config.splines, self.config.overlap
        ));
        dot.push_str(&format!(
            "    edge [color=\"{}\", arrowhead=\"{}\"]; \n",
            self.config.edge_color, self.config.arrowhead
        ));
        dot.push_str(&format!(
            "    node [shape=\"{}\", fontname=\"{}\", fontsize=\"{}\"]; \n",
            self.config.node_shape, self.config.fontname, self.config.fontsize
        ));

        // Render each node using the resolver.
        for (node_index, _node_data) in graph.node_references() {
            if let Some(data) = resolver.resolve(node_index) {
                dot.push_str(&format!(
                    "    N{} [shape=plaintext,style=filled,fillcolor=\"#555555\",label=<\n{}    >];\n",
                    node_index.index(),
                    data.render_node(8)
                ));
            }
        }

        // Render edges between nodes.
        for edge in graph.edge_references() {
            let source = edge.source();
            let target = edge.target();
            dot.push_str(&format!(
                "    N{} -> N{};\n",
                source.index(),
                target.index()
            ));
        }

        // Close the graph definition.
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
