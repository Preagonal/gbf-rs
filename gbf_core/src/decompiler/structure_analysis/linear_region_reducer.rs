#![deny(missing_docs)]

use std::backtrace::Backtrace;

use super::{
    region::{RegionId, RegionType},
    RegionReducer, StructureAnalysis, StructureAnalysisError,
};

/// Reduces a linear region.
pub struct LinearRegionReducer;

impl LinearRegionReducer {
    /// Merge two regions.
    fn merge_regions(
        &mut self,
        analysis: &mut StructureAnalysis,
        from_region_id: RegionId,
        to_region_id: RegionId,
    ) -> Result<(), StructureAnalysisError> {
        let (from_nodes, from_jump_expr, region_type) = {
            let from_region = analysis.regions.get_mut(from_region_id.index).ok_or(
                StructureAnalysisError::RegionNotFound {
                    region_id: from_region_id,
                    backtrace: Backtrace::capture(),
                },
            )?;
            (
                from_region.get_nodes().to_vec(),
                from_region.get_jump_expr().cloned(),
                *from_region.region_type(),
            )
        };

        let to_region = analysis.regions.get_mut(to_region_id.index).ok_or(
            StructureAnalysisError::RegionNotFound {
                region_id: to_region_id,
                backtrace: Backtrace::capture(),
            },
        )?;

        if region_type == RegionType::Inactive {
            return Err(StructureAnalysisError::Other {
                message: "Cannot merge inactive region".to_string(),
                backtrace: Backtrace::capture(),
            });
        }

        to_region.push_nodes(from_nodes);
        to_region.set_jump_expr(from_jump_expr);
        to_region.set_region_type(region_type);

        Ok(())
    }
}

impl RegionReducer for LinearRegionReducer {
    fn reduce_region(
        &mut self,
        analysis: &mut StructureAnalysis,
        region_id: RegionId,
    ) -> Result<bool, StructureAnalysisError> {
        let succ = analysis.get_single_successor(region_id)?.ok_or_else(|| {
            StructureAnalysisError::Other {
                message: "Linear region does not have exactly one successor".to_string(),
                backtrace: Backtrace::capture(),
            }
        })?;

        if !analysis.has_single_predecessor(succ)? {
            return Ok(false);
        }

        // Call the before_reduce hook
        analysis.before_reduce(region_id);
        self.merge_regions(analysis, succ, region_id)?;
        analysis.remove_edge(region_id, succ)?;

        // For each successor of the successor region, add an edge from the region to the successor
        for (succ_succ, edge_type) in analysis.get_successors(succ)? {
            analysis.connect_regions(region_id, succ_succ, edge_type)?;
            analysis.remove_edge(succ, succ_succ)?;
        }

        // Remove the successor region
        analysis.remove_node(succ)?;
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use crate::decompiler::{
        ast::{new_assignment, new_id},
        structure_analysis::ControlFlowEdgeType,
    };

    use super::*;

    #[test]
    fn test_linear_reduce() -> Result<(), StructureAnalysisError> {
        let mut structure_analysis = StructureAnalysis::new(false, 100);

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
        structure_analysis.connect_regions(entry_region, region_1, ControlFlowEdgeType::Branch)?;
        structure_analysis.connect_regions(region_1, region_2, ControlFlowEdgeType::Branch)?;
        structure_analysis.connect_regions(region_2, region_3, ControlFlowEdgeType::Branch)?;
        structure_analysis.execute()?;

        assert_eq!(structure_analysis.region_graph.node_count(), 1);

        let region = structure_analysis.get_entry_region();
        let region = structure_analysis.get_region(region)?;
        assert_eq!(region.get_nodes().len(), 6);

        // ensure that the final region is a tail region
        assert_eq!(region.get_region_type(), RegionType::Tail);

        Ok(())
    }
}
