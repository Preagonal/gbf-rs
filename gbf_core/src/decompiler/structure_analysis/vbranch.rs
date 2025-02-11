#![deny(missing_docs)]

use std::backtrace::Backtrace;

use crate::decompiler::ast::new_virtual_branch;

use super::{
    region::{RegionId, RegionType},
    RegionReducer, StructureAnalysis, StructureAnalysisError,
};

/// If the region has a jump, create a virtual branch
pub struct VirtualBranchReducer;

impl VirtualBranchReducer {}

impl RegionReducer for VirtualBranchReducer {
    fn reduce_region(
        &mut self,
        analysis: &mut StructureAnalysis,
        region_id: RegionId,
    ) -> Result<bool, StructureAnalysisError> {
        // This logic applies to control flow regions only
        if analysis.get_region_type(region_id)? != RegionType::Linear {
            return Ok(false);
        }

        // Ensure that this linear region has one successor node
        let successors = analysis.get_successors(region_id)?;
        if successors.len() != 1 {
            return Err(StructureAnalysisError::Other {
                message: "Linear region does not have exactly one successor".to_string(),
                backtrace: Backtrace::capture(),
            });
        }

        // Get the successor region
        let successor_id = successors[0];

        // Insert the virtual branch
        let vbranch = new_virtual_branch(successor_id.0);
        let region = analysis.get_region_mut(region_id)?;
        region.push_node(vbranch.into());
        region.set_region_type(RegionType::Tail);
        analysis.remove_edge(region_id, successor_id.0)?;
        Ok(true)
    }
}
