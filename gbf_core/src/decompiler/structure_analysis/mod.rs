#![deny(missing_docs)]

use std::{backtrace::Backtrace, collections::HashMap};

use linear_region_reducer::LinearRegionReducer;
use petgraph::{
    graph::{DiGraph, NodeIndex},
    visit::DfsPostOrder,
};
use region::{Region, RegionId, RegionType};
use serde::Serialize;

use crate::{
    cfg_dot::{CfgDot, CfgDotConfig, DotRenderableGraph, NodeResolver},
    utils::{GBF_BLUE, STRUCTURE_ANALYSIS_MAX_ITERATIONS},
};

use super::ast::AstKind;

use thiserror::Error;

/// A module that contains the logic for reducing a linear region.
pub mod linear_region_reducer;
/// A module representing a region in the control flow graph.
pub mod region;

/// A trait for reducing a region
pub trait RegionReducer {
    /// Reduces a region.
    fn reduce_region(
        &mut self,
        analysis: &mut StructureAnalysis,
        region_id: RegionId,
    ) -> Result<bool, StructureAnalysisError>;
}

/// Error type for structure analysis.
#[derive(Debug, Error, Serialize)]
pub enum StructureAnalysisError {
    /// Error when a region is not found.
    #[error("Region not found: {:?}", region_id)]
    RegionNotFound {
        /// The region ID that was not found.
        region_id: RegionId,

        /// The error backtrace.
        #[serde(skip)]
        backtrace: Backtrace,
    },

    /// Error when the entry region is not found.
    #[error("Entry region not found")]
    EntryRegionNotFound {
        /// The error backtrace.
        #[serde(skip)]
        backtrace: Backtrace,
    },

    /// When we have reached the maximum number of iterations.
    #[error(
        "Maximum number of iterations reached: {0}",
        STRUCTURE_ANALYSIS_MAX_ITERATIONS
    )]
    MaxIterationsReached {
        /// The error backtrace.
        #[serde(skip)]
        backtrace: Backtrace,
    },

    /// Other errors.
    #[error("A structure analysis error occurred: {message}")]
    Other {
        /// The error message.
        message: String,

        /// The error backtrace.
        #[serde(skip)]
        backtrace: Backtrace,
    },
}

/// This module is responsible for control flow analysis.
#[derive(Default)]
pub struct StructureAnalysis {
    /// Regions vector
    regions: Vec<Region>,
    /// The region graph of the function
    region_graph: DiGraph<(), ()>,
    /// Used to convert `NodeIndex` to `RegionId`.
    graph_node_to_region: HashMap<NodeIndex, RegionId>,
    /// Used to convert `RegionId` to `NodeIndex`.
    region_to_graph_node: HashMap<RegionId, NodeIndex>,
}

impl StructureAnalysis {
    /// Creates a new `StructureAnalysis` instance.
    pub fn new() -> Self {
        Self {
            regions: Vec::new(),
            region_graph: DiGraph::new(),
            graph_node_to_region: HashMap::new(),
            region_to_graph_node: HashMap::new(),
        }
    }

    /// Adds a new region to the control flow graph.
    pub fn add_region(&mut self, region_type: RegionType) -> RegionId {
        let region_id = RegionId::new(self.regions.len());
        self.regions.push(Region::new(region_id, region_type));
        let node_index = self.region_graph.add_node(());
        self.graph_node_to_region.insert(node_index, region_id);
        self.region_to_graph_node.insert(region_id, node_index);
        region_id
    }

    /// Connect two regions in the control flow graph.
    pub fn connect_regions(&mut self, from: RegionId, to: RegionId) {
        let from_node = self.region_to_graph_node[&from];
        let to_node = self.region_to_graph_node[&to];
        self.region_graph.add_edge(from_node, to_node, ());
    }

    /// Gets a region by its ID.
    pub fn get_region(&self, region_id: RegionId) -> Result<&Region, StructureAnalysisError> {
        self.regions
            .get(region_id.index)
            .ok_or(StructureAnalysisError::RegionNotFound {
                region_id,
                backtrace: Backtrace::capture(),
            })
    }

    /// Gets the entry region id
    pub fn get_entry_region(&self) -> Result<RegionId, StructureAnalysisError> {
        // TODO: Find a more robust method for finding the entry region. This is
        // TODO: a costly operation, and not robust. We will introduce some debug
        // TODO: assertions to ensure that this function returns the entry region

        // Iterate through the regions in the map until we find the entry region.
        for (region_id, _) in self.region_to_graph_node.iter() {
            if region_id.index == 0 {
                let region_id = *region_id;

                // debug assertion - basically, ensure no predecessors
                debug_assert_eq!(
                    self.region_graph
                        .neighbors_directed(
                            self.region_to_graph_node[&region_id],
                            petgraph::Direction::Incoming
                        )
                        .count(),
                    0
                );

                return Ok(region_id);
            }
        }

        Err(StructureAnalysisError::EntryRegionNotFound {
            backtrace: Backtrace::capture(),
        })
    }

    /// Executes the control flow analysis.
    pub fn execute(&mut self) -> Result<(), StructureAnalysisError> {
        let iterations = 0;

        // while the region count is still above 1
        while self.region_graph.node_count() > 1 {
            // if we have reached the maximum number of iterations
            if iterations > STRUCTURE_ANALYSIS_MAX_ITERATIONS {
                return Err(StructureAnalysisError::MaxIterationsReached {
                    backtrace: Backtrace::capture(),
                });
            }

            let old_node_count = self.regions.len();

            // Get the nodes in post order
            let entry_region_id = self.get_entry_region()?;
            let mut dfs = DfsPostOrder::new(
                &self.region_graph,
                self.region_to_graph_node[&entry_region_id],
            );

            // Iterate through the nodes in post order
            while let Some(node) = dfs.next(&self.region_graph) {
                let mut reduce_iterations = 0;
                let mut did_reduce = true;

                loop {
                    // Reduce the region
                    did_reduce = self.reduce_acyclic_region(self.graph_node_to_region[&node])?;

                    if did_reduce {
                        reduce_iterations += 1;
                    }

                    if !did_reduce {
                        break;
                    }
                }
            }
        }

        Ok(())
    }

    /// Push a node to a region.
    pub fn push_to_region<T>(&mut self, region_id: RegionId, node: T)
    where
        T: Into<AstKind>,
    {
        let region = self
            .regions
            .get_mut(region_id.index)
            .expect("Region not found");
        region.push_node(node.into());
    }

    /// Get the single successor of a region, if there is only one.
    ///
    /// # Arguments
    /// - `region_id`: The region ID to get the successor of.
    ///
    /// # Returns
    /// - An `Option` containing the successor node index and region ID if there is only one successor.
    pub fn get_single_successor(&self, region_id: RegionId) -> Option<RegionId> {
        let region_node = self.region_to_graph_node[&region_id];
        let successors: Vec<NodeIndex> = self
            .region_graph
            .neighbors_directed(region_node, petgraph::Direction::Outgoing)
            .collect();

        if successors.len() != 1 {
            return None;
        }

        let successor_node = successors[0];
        let successor_region_id = self.graph_node_to_region[&successor_node];

        successor_region_id.into()
    }

    /// Check if a node has a single predecessor.
    ///
    /// # Arguments
    /// - `node_index`: The node index to check.
    ///
    /// # Returns
    /// - `true` if the node has a single predecessor, `false` otherwise.
    pub fn has_single_predecessor(&self, id: RegionId) -> bool {
        let node_index = self.region_to_graph_node[&id];
        self.region_graph
            .neighbors_directed(node_index, petgraph::Direction::Incoming)
            .count()
            == 1
    }

    /// Remove an edge between two regions.
    ///
    /// # Arguments
    /// - `from_region_id`: The region ID of the source region.
    /// - `to_region_id`: The region ID of the destination region.
    ///
    /// # Returns
    /// - `Ok(())` if the operation was successful.
    /// - `Err(StructureAnalysisError)` if an error occurred.
    pub fn remove_edge(
        &mut self,
        from_region_id: RegionId,
        to_region_id: RegionId,
    ) -> Result<(), StructureAnalysisError> {
        let region_node = self.region_to_graph_node[&from_region_id];
        let successor_node = self.region_to_graph_node[&to_region_id];
        let edge_index = self
            .region_graph
            .find_edge(region_node, successor_node)
            .ok_or(StructureAnalysisError::Other {
                message: "Edge not found".to_string(),
                backtrace: Backtrace::capture(),
            })?;
        // Remove the edge between the two nodes
        self.region_graph.remove_edge(edge_index);

        Ok(())
    }

    /// Gets the successors of a region as a vector of region IDs.
    ///
    /// # Arguments
    /// - `region_id`: The region ID to get the successors of.
    ///
    /// # Returns
    /// - A vector of region IDs representing the successors of the region.
    pub fn get_successors(&self, region_id: RegionId) -> Vec<RegionId> {
        let region_node = self.region_to_graph_node[&region_id];
        self.region_graph
            .neighbors_directed(region_node, petgraph::Direction::Outgoing)
            .map(|node| self.graph_node_to_region[&node])
            .collect()
    }

    /// Gets the predecessors of a region as a vector of region IDs.
    ///
    /// # Arguments
    /// - `region_id`: The region ID to get the predecessors of.
    ///
    /// # Returns
    /// - A vector of region IDs representing the predecessors of the region.
    pub fn get_predecessors(&self, region_id: RegionId) -> Vec<RegionId> {
        let region_node = self.region_to_graph_node[&region_id];
        self.region_graph
            .neighbors_directed(region_node, petgraph::Direction::Incoming)
            .map(|node| self.graph_node_to_region[&node])
            .collect()
    }

    /// Removes a node from the region graph.
    ///
    /// # Arguments
    /// - `region_id`: The region ID of the region to remove.
    pub fn remove_node(&mut self, region_id: RegionId) {
        let node_index = self.region_to_graph_node[&region_id];
        self.region_graph.remove_node(node_index);
    }
}

// Private impls
impl StructureAnalysis {
    /// Reduce acyclic regions.
    fn reduce_acyclic_region(
        &mut self,
        region_id: RegionId,
    ) -> Result<bool, StructureAnalysisError> {
        // get the region type from RegionId
        let region =
            self.regions
                .get(region_id.index)
                .ok_or(StructureAnalysisError::RegionNotFound {
                    region_id,
                    backtrace: Backtrace::capture(),
                })?;

        Ok(match region.get_region_type() {
            RegionType::Linear => LinearRegionReducer.reduce_region(self, region_id)?,
            RegionType::Tail => false,
            _ => todo!("Implement other region types"),
        })
    }
}

// == Other impls ==
impl DotRenderableGraph for StructureAnalysis {
    /// Convert the Graph to `dot` format.
    ///
    /// # Returns
    /// - A `String` containing the `dot` representation of the graph.
    fn render_dot(&self, config: CfgDotConfig) -> String {
        let dot = CfgDot { config };
        dot.render(&self.region_graph, self)
    }
}

impl NodeResolver for StructureAnalysis {
    type NodeData = Region;

    fn resolve(&self, node_index: NodeIndex) -> Option<&Self::NodeData> {
        self.graph_node_to_region
            .get(&node_index)
            .and_then(|region_id| self.regions.get(region_id.index))
    }

    fn resolve_edge_color(&self, _: NodeIndex, _: NodeIndex) -> String {
        // TODO: Change based on CFG patterns
        GBF_BLUE.to_string()
    }
}

#[cfg(test)]
mod tests {
    use crate::decompiler::ast::{new_assignment, new_id};

    use super::*;

    #[test]
    fn test_linear_reduce() -> Result<(), StructureAnalysisError> {
        let mut structure_analysis = StructureAnalysis::new();

        let entry_region = structure_analysis.add_region(RegionType::Linear);
        let region_1 = structure_analysis.add_region(RegionType::Linear);
        let region_2 = structure_analysis.add_region(RegionType::Linear);
        let region_3 = structure_analysis.add_region(RegionType::Tail);

        // push nodes to the regions
        structure_analysis
            .push_to_region(entry_region, new_assignment(new_id("foo"), new_id("bar")));
        structure_analysis.push_to_region(region_1, new_assignment(new_id("foo2"), new_id("bar2")));
        structure_analysis.push_to_region(region_1, new_assignment(new_id("foo3"), new_id("bar3")));
        structure_analysis.push_to_region(region_2, new_assignment(new_id("foo4"), new_id("bar4")));
        structure_analysis.push_to_region(region_2, new_assignment(new_id("foo5"), new_id("bar5")));
        structure_analysis.push_to_region(region_3, new_assignment(new_id("foo6"), new_id("bar6")));
        structure_analysis.connect_regions(entry_region, region_1);
        structure_analysis.connect_regions(region_1, region_2);
        structure_analysis.connect_regions(region_2, region_3);
        structure_analysis.execute()?;

        assert_eq!(structure_analysis.region_graph.node_count(), 1);

        let region = structure_analysis.get_entry_region()?;
        let region = structure_analysis.get_region(region)?;
        assert_eq!(region.get_nodes().len(), 6);

        // ensure that the final region is a tail region
        assert_eq!(region.get_region_type(), RegionType::Tail);

        Ok(())
    }
}
