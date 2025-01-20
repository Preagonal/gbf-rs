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

        to_region.push_nodes(from_nodes);
        to_region.set_jump_expr(from_jump_expr);
        to_region.set_region_type(region_type);

        let from_region = analysis.regions.get_mut(from_region_id.index).ok_or(
            StructureAnalysisError::RegionNotFound {
                region_id: from_region_id,
                backtrace: Backtrace::capture(),
            },
        )?;
        from_region.set_region_type(RegionType::Inactive);

        Ok(())
    }
}

impl RegionReducer for LinearRegionReducer {
    fn reduce_region(
        &mut self,
        analysis: &mut StructureAnalysis,
        region_id: RegionId,
    ) -> Result<bool, StructureAnalysisError> {
        let succ = analysis.get_single_successor(region_id).ok_or_else(|| {
            StructureAnalysisError::Other {
                message: "Linear region does not have exactly one successor".to_string(),
                backtrace: Backtrace::capture(),
            }
        })?;

        if !analysis.has_single_predecessor(succ) {
            return Ok(false);
        }

        self.merge_regions(analysis, succ, region_id)?;

        analysis.remove_edge(region_id, succ)?;
        analysis.remove_node(succ);
        Ok(true)
    }
}
