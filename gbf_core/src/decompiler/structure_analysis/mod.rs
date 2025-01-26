#![deny(missing_docs)]

use std::backtrace::Backtrace;

use if_region_reducer::IfRegionReducer;
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

/// A module representing a region that is an if
pub mod if_region_reducer;
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

    /// When we've expected a condition in this region, but it's not there.
    #[error("Expected condition not found")]
    ExpectedConditionNotFound {
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

/// The type of control flow edge in the CFG.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlFlowEdgeType {
    /// A branch
    Branch,
    /// A fallthrough
    Fallthrough,
}

/// This module is responsible for control flow analysis.
#[derive(Default)]
pub struct StructureAnalysis {
    /// Regions vector
    regions: Vec<Region>,
    /// The region graph of the function
    region_graph: DiGraph<RegionId, ControlFlowEdgeType>,
}

impl StructureAnalysis {
    /// Creates a new `StructureAnalysis` instance.
    pub fn new() -> Self {
        Self {
            regions: Vec::new(),
            region_graph: DiGraph::new(),
        }
    }

    /// Adds a new region to the control flow graph.
    pub fn add_region(&mut self, region_type: RegionType) -> RegionId {
        let region_id = RegionId::new(self.regions.len());
        self.regions.push(Region::new(region_type));
        self.region_graph.add_node(region_id);
        region_id
    }

    /// Connect two regions in the control flow graph.
    pub fn connect_regions(
        &mut self,
        from: RegionId,
        to: RegionId,
        edge_type: ControlFlowEdgeType,
    ) -> Result<(), StructureAnalysisError> {
        let from_node = self.get_node_index(from)?;
        let to_node = self.get_node_index(to)?;
        self.region_graph.add_edge(from_node, to_node, edge_type);
        Ok(())
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

    /// Gets a mutable region by its ID.
    pub fn get_region_mut(
        &mut self,
        region_id: RegionId,
    ) -> Result<&mut Region, StructureAnalysisError> {
        self.regions
            .get_mut(region_id.index)
            .ok_or(StructureAnalysisError::RegionNotFound {
                region_id,
                backtrace: Backtrace::capture(),
            })
    }

    /// Gets the entry region id
    pub fn get_entry_region(&self) -> RegionId {
        // TODO: Find a more robust method for finding the entry region. Our assumption
        // is that the entry region is the region with index 0.
        RegionId::new(0)
    }

    /// Gets the node index of a region.
    pub fn get_node_index(&self, region_id: RegionId) -> Result<NodeIndex, StructureAnalysisError> {
        self.region_graph
            .node_indices()
            .find(|&idx| self.region_graph[idx] == region_id)
            .ok_or(StructureAnalysisError::RegionNotFound {
                region_id,
                backtrace: Backtrace::capture(),
            })
    }

    /// Gets the region ID of a node index.
    pub fn get_region_id(&self, node_index: NodeIndex) -> Result<RegionId, StructureAnalysisError> {
        self.region_graph
            .node_weight(node_index)
            .cloned()
            .ok_or(StructureAnalysisError::Other {
                message: "Node index not found".to_string(),
                backtrace: Backtrace::capture(),
            })
    }

    /// Executes the control flow analysis.
    pub fn execute(&mut self) -> Result<(), StructureAnalysisError> {
        let mut iterations = 0;

        // while the region count is still above 1
        while self.region_graph.node_count() > 1 {
            // if we have reached the maximum number of iterations
            if iterations > STRUCTURE_ANALYSIS_MAX_ITERATIONS {
                return Err(StructureAnalysisError::MaxIterationsReached {
                    backtrace: Backtrace::capture(),
                });
            }

            // TODO: We will use the old node count in our post reduction check
            let _old_node_count = self.regions.len();

            // Get the nodes in post order
            let entry_region_id = self.get_entry_region();
            let entry_region_node_index = self.get_node_index(entry_region_id)?;

            let mut dfs = DfsPostOrder::new(&self.region_graph, entry_region_node_index);

            // Iterate through the nodes in post order
            while let Some(node) = dfs.next(&self.region_graph) {
                let region_id = self.get_region_id(node)?;
                // If the region is inactive, skip it
                if self.regions[region_id.index].get_region_type() == RegionType::Inactive {
                    continue;
                }
                loop {
                    // Reduce the region
                    let did_reduce = self.reduce_acyclic_region(region_id)?;

                    if !did_reduce {
                        break;
                    }
                }
            }

            iterations += 1;
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
    pub fn get_single_successor(
        &self,
        region_id: RegionId,
    ) -> Result<Option<RegionId>, StructureAnalysisError> {
        let successors = self.get_successors(region_id)?;

        if successors.len() != 1 {
            return Ok(None);
        }

        Ok(Some(successors[0].0))
    }

    /// Get the single linear successor of a region, if the region type is linear.
    ///
    /// # Arguments
    /// - `region_id`: The region ID to get the successor of.
    ///
    /// # Returns
    /// - An `Option` containing the successor node index and region ID if there is only one linear successor.
    pub fn get_single_linear_successor(
        &self,
        region_id: RegionId,
    ) -> Result<Option<RegionId>, StructureAnalysisError> {
        // Get the region type
        let region_type = self.regions[region_id.index].get_region_type();

        // If the region type is not linear, return None
        if region_type != RegionType::Linear {
            return Ok(None);
        }

        self.get_single_successor(region_id)
    }

    /// Check if a node has a single predecessor.
    ///
    /// # Arguments
    /// - `node_index`: The node index to check.
    ///
    /// # Returns
    /// - `true` if the node has a single predecessor, `false` otherwise.
    pub fn has_single_predecessor(&self, id: RegionId) -> Result<bool, StructureAnalysisError> {
        let node_index = self.get_node_index(id)?;
        Ok(self
            .region_graph
            .neighbors_directed(node_index, petgraph::Direction::Incoming)
            .count()
            == 1)
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
        let region_node = self.get_node_index(from_region_id)?;
        let successor_node = self.get_node_index(to_region_id)?;
        let edge_index = self
            .region_graph
            .find_edge(region_node, successor_node)
            .ok_or(StructureAnalysisError::Other {
                message: "Edge not found".to_string(),
                backtrace: Backtrace::capture(),
            })?;
        // Remove the edge between the two nodes
        debug_assert!(self.region_graph.remove_edge(edge_index).is_some());

        Ok(())
    }

    /// Gets the successors of a region as a vector of region IDs.
    ///
    /// # Arguments
    /// - `region_id`: The region ID to get the successors of.
    ///
    /// # Returns
    /// - A vector of region IDs representing the successors of the region.
    pub fn get_successors(
        &self,
        region_id: RegionId,
    ) -> Result<Vec<(RegionId, ControlFlowEdgeType)>, StructureAnalysisError> {
        let region_node = self.get_node_index(region_id)?;
        self.region_graph
            .neighbors_directed(region_node, petgraph::Direction::Outgoing)
            .map(|node| {
                let region_id = self.get_region_id(node)?;
                let edge = self
                    .region_graph
                    .find_edge(region_node, node)
                    .ok_or_else(|| StructureAnalysisError::Other {
                        message: "Edge not found".to_string(),
                        backtrace: Backtrace::capture(),
                    })?;
                let edge_weight = self.region_graph.edge_weight(edge).ok_or_else(|| {
                    StructureAnalysisError::Other {
                        message: "Edge weight not found".to_string(),
                        backtrace: Backtrace::capture(),
                    }
                })?;
                Ok((region_id, *edge_weight))
            })
            .collect()
    }

    /// Gets the predecessors of a region as a vector of region IDs.
    ///
    /// # Arguments
    /// - `region_id`: The region ID to get the predecessors of.
    ///
    /// # Returns
    /// - A vector of region IDs representing the predecessors of the region.
    pub fn get_predecessors(
        &self,
        region_id: RegionId,
    ) -> Result<Vec<RegionId>, StructureAnalysisError> {
        let region_node = self.get_node_index(region_id)?;
        self.region_graph
            .neighbors_directed(region_node, petgraph::Direction::Incoming)
            .map(|node| self.get_region_id(node))
            .collect()
    }

    /// Removes a node from the region graph.
    ///
    /// # Arguments
    /// - `region_id`: The region ID of the region to remove.
    pub fn remove_node(&mut self, region_id: RegionId) -> Result<(), StructureAnalysisError> {
        let node_index = self.get_node_index(region_id)?;
        debug_assert!(self.region_graph.remove_node(node_index).is_some());

        // set the region to inactive
        self.regions[region_id.index].set_region_type(RegionType::Inactive);

        Ok(())
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
            RegionType::Inactive => Err(StructureAnalysisError::Other {
                message: "Inactive region".to_string(),
                backtrace: Backtrace::capture(),
            })?,
            RegionType::ControlFlow => IfRegionReducer.reduce_region(self, region_id)?,
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
        let region_id = self.get_region_id(node_index).ok()?;
        self.regions.get(region_id.index)
    }

    fn resolve_edge_color(&self, _: NodeIndex, _: NodeIndex) -> String {
        // TODO: Change based on CFG patterns
        GBF_BLUE.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decompiler::ast::{new_assignment, new_id};

    #[test]
    fn test_remove_edge() -> Result<(), StructureAnalysisError> {
        let mut structure_analysis = StructureAnalysis::new();

        let entry_region = structure_analysis.add_region(RegionType::Linear);
        let region_1 = structure_analysis.add_region(RegionType::Linear);
        let region_2 = structure_analysis.add_region(RegionType::Tail);

        // push nodes to the regions
        structure_analysis
            .push_to_region(entry_region, new_assignment(new_id("foo"), new_id("bar")));
        // set condition for the region
        structure_analysis
            .get_region_mut(entry_region)?
            .set_jump_expr(Some(new_id("foo").into()));
        structure_analysis.push_to_region(region_1, new_assignment(new_id("foo2"), new_id("bar2")));
        structure_analysis.push_to_region(region_1, new_assignment(new_id("foo3"), new_id("bar3")));
        structure_analysis.push_to_region(region_2, new_assignment(new_id("foo4"), new_id("bar4")));
        structure_analysis.push_to_region(region_2, new_assignment(new_id("foo5"), new_id("bar5")));
        structure_analysis.connect_regions(
            entry_region,
            region_1,
            ControlFlowEdgeType::Fallthrough,
        )?;
        structure_analysis.connect_regions(entry_region, region_2, ControlFlowEdgeType::Branch)?;
        structure_analysis.connect_regions(region_1, region_2, ControlFlowEdgeType::Fallthrough)?;

        // print graph

        // remove edge from entry_region to region_1
        structure_analysis.remove_edge(entry_region, region_1)?;
        // remove edge from region_1 to region_2
        structure_analysis.remove_edge(region_1, region_2)?;
        // remove node region_1
        structure_analysis.remove_node(region_1)?;
        // get successors of entry_region
        let successors = structure_analysis.get_successors(entry_region)?;
        // check if the entry region has only one successor
        assert_eq!(successors.len(), 1);
        // ensure successors[0] is region_2
        assert_eq!(successors[0].0, region_2);
        // do structure analysis
        structure_analysis.execute()?;
        // check if the region graph has only one node
        assert_eq!(structure_analysis.region_graph.node_count(), 1);

        Ok(())
    }
}
