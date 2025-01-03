#![deny(missing_docs)]

use crate::basic_block::BasicBlockId;
use crate::cfg_dot::{CfgDotBuilder, DotRenderableGraph, NodeResolver};
use crate::decompiler::region::{Region, RegionId, RegionType};
use crate::function::{Function, FunctionError};
use petgraph::graph::{DiGraph, NodeIndex};
use std::collections::HashMap;
use thiserror::Error;

/// An error when decompiling a function
#[derive(Debug, Error)]
pub enum FunctionDecompilerError {
    /// Encountered FunctionError
    #[error("Encountered FunctionError while decompiling: {0}")]
    FunctionError(#[from] FunctionError),
}

/// A struct to hold the state of a function decompiler
pub struct FunctionDecompiler {
    /// Create a copy of the function to analyze
    function: Function,
    /// Regions vector
    regions: Vec<Region>,
    /// A conversion from block ids to region ids
    block_to_region: HashMap<BasicBlockId, RegionId>,
    /// The region graph of the function
    region_graph: DiGraph<(), ()>,
    /// Used to convert `NodeIndex` to `RegionId`.
    graph_node_to_region: HashMap<NodeIndex, RegionId>,
    /// Used to convert `RegionId` to `NodeIndex`.
    region_to_graph_node: HashMap<RegionId, NodeIndex>,
}

impl FunctionDecompiler {
    /// A new method for the FunctionDecompiler struct.
    ///
    /// # Arguments
    /// - `function`: The function to analyze and decompile.
    ///
    /// # Returns
    /// - A newly constructed `FunctionDecompiler` instance.
    pub fn new(function: Function) -> Self {
        // Stub implementation
        Self {
            function,
            regions: Vec::new(),
            block_to_region: HashMap::new(),
            region_graph: DiGraph::new(),
            graph_node_to_region: HashMap::new(),
            region_to_graph_node: HashMap::new(),
        }
    }
}

// == Private Functions ==
impl FunctionDecompiler {
    fn build_regions(&mut self) -> Result<(), FunctionDecompilerError> {
        // Generate all the regions before doing anything else
        for block in self.function.iter() {
            let new_region_id = RegionId::new(self.regions.len(), RegionType::Linear);
            self.block_to_region.insert(block.id, new_region_id.clone());

            // Add to the graph
            let node_id = self.region_graph.add_node(());
            self.graph_node_to_region
                .insert(node_id, new_region_id.clone());
            self.region_to_graph_node
                .insert(new_region_id.clone(), node_id);

            // Add to the array of regions
            self.regions.push(Region::new(new_region_id));
        }

        // Iterate through all the blocks in reverse post order
        // Get a list of blocks in reverse post order
        let reverse_post_order = self
            .function
            .get_reverse_post_order(self.function.get_entry_basic_block().id)
            .map_err(FunctionDecompilerError::FunctionError)?;
        for _block_id in reverse_post_order {
            todo!()
        }

        Ok(())
    }
}

// == Other Implementations ==
impl DotRenderableGraph for FunctionDecompiler {
    /// Convert the Graph to `dot` format.
    ///
    /// # Returns
    /// - A `String` containing the `dot` representation of the graph.
    fn render_dot(&self) -> String {
        let dot_bulder = CfgDotBuilder::new().build();
        dot_bulder.render(&self.region_graph, self)
    }
}

impl NodeResolver for FunctionDecompiler {
    type NodeData = Region;

    fn resolve(&self, node_index: NodeIndex) -> Option<&Self::NodeData> {
        self.graph_node_to_region
            .get(&node_index)
            .and_then(|region_id| self.regions.get(region_id.index))
    }

    fn resolve_edge_color(&self, _: NodeIndex, _: NodeIndex) -> String {
        // TODO: Change based on CFG patterns
        "#00bbff".to_string()
    }
}
