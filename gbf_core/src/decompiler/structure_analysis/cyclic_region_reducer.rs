#![deny(missing_docs)]

use std::backtrace::Backtrace;

use crate::decompiler::ast::{
    control_flow::ControlFlowNode, expr::ExprKind, new_cyclic_condition, new_do_while, ptr::P,
    AstKind,
};

use super::{
    region::{RegionId, RegionType},
    RegionReducer, StructureAnalysis, StructureAnalysisError,
};

/// Reduces a linear region.
pub struct CyclicRegionReducer;

impl CyclicRegionReducer {
    /// Extracts the jump expression from a region, if available.
    fn extract_jump_expr(
        analysis: &mut StructureAnalysis,
        region_id: RegionId,
    ) -> Result<ExprKind, StructureAnalysisError> {
        let region = analysis.regions.get_mut(region_id.index).ok_or(
            StructureAnalysisError::RegionNotFound {
                region_id,
                backtrace: Backtrace::capture(),
            },
        )?;
        region
            .get_jump_expr()
            .ok_or(StructureAnalysisError::ExpectedConditionNotFound {
                backtrace: Backtrace::capture(),
            })
            .cloned()
    }

    /// Remove the given node and its adjacent edges from the region.
    fn cleanup_region(
        analysis: &mut StructureAnalysis,
        remove_node: RegionId,
        start_node: RegionId,
        final_node: RegionId,
    ) -> Result<(), StructureAnalysisError> {
        analysis.remove_edge(start_node, remove_node)?;
        analysis.remove_edge(remove_node, final_node)?;
        analysis.remove_node(remove_node)?;
        Ok(())
    }

    /// Handles merging the conditional structure into the original region.
    fn merge_conditional(
        analysis: &mut StructureAnalysis,
        region_id: RegionId,
        cond: P<ControlFlowNode>,
    ) -> Result<(), StructureAnalysisError> {
        let region = analysis.regions.get_mut(region_id.index).ok_or(
            StructureAnalysisError::RegionNotFound {
                region_id,
                backtrace: Backtrace::capture(),
            },
        )?;
        region.push_node(cond.into());
        region.set_region_type(RegionType::Linear);
        region.remove_jump_expr();
        Ok(())
    }

    /// Replace the region nodes with the given nodes (for do-while loops).
    fn replace_region_nodes(
        analysis: &mut StructureAnalysis,
        region_id: RegionId,
        node: P<ControlFlowNode>,
    ) -> Result<(), StructureAnalysisError> {
        let region = analysis.regions.get_mut(region_id.index).ok_or(
            StructureAnalysisError::RegionNotFound {
                region_id,
                backtrace: Backtrace::capture(),
            },
        )?;
        region.clear_nodes();
        region.push_node(node.into());
        region.set_region_type(RegionType::Linear);
        region.remove_jump_expr();

        // Remove edge between the region and itself
        analysis.remove_edge(region_id, region_id)?;
        Ok(())
    }

    /// Extracts the nodes of a given region.
    fn get_region_nodes(
        analysis: &StructureAnalysis,
        region_id: RegionId,
    ) -> Result<Vec<AstKind>, StructureAnalysisError> {
        let region = analysis.regions.get(region_id.index).ok_or(
            StructureAnalysisError::RegionNotFound {
                region_id,
                backtrace: Backtrace::capture(),
            },
        )?;
        Ok(region.get_nodes().to_vec())
    }
}

impl RegionReducer for CyclicRegionReducer {
    fn reduce_region(
        &mut self,
        analysis: &mut StructureAnalysis,
        region_id: RegionId,
    ) -> Result<bool, StructureAnalysisError> {
        let successors = analysis.get_successors(region_id)?;
        let len = successors.len();

        // Case 1: doWhile, if the region has an expression AND the successor is the same region.
        for successor in &successors {
            if successor.0 == region_id {
                // Call the before_reduce hook
                analysis.before_reduce(region_id);

                if len == 1 {
                    // TODO: Infinite loop: not implemented yet
                    return Err(StructureAnalysisError::Other {
                        message: "Infinite loop not implemented yet".to_string(),
                        backtrace: Backtrace::capture(),
                    });
                }

                let jump_expr = Self::extract_jump_expr(analysis, region_id)?;
                let region_nodes = Self::get_region_nodes(analysis, region_id)?;
                let cond: P<ControlFlowNode> = new_do_while(jump_expr, region_nodes).into();

                Self::replace_region_nodes(analysis, region_id, cond)?;
                return Ok(true);
            }
        }

        // Case 2: while, if the region has an expression AND the successor's successor is the same region.
        for successor in &successors {
            if analysis.get_single_linear_successor(successor.0)? == Some(region_id)
                && analysis.get_single_predecessor(successor.0)? == Some(region_id)
            {
                // Call the before_reduce hook
                analysis.before_reduce(region_id);

                // We have a while loop! Merge the regions.
                let jump_expr = Self::extract_jump_expr(analysis, region_id)?;
                let region_nodes = Self::get_region_nodes(analysis, successor.0)?;
                let cond: P<ControlFlowNode> = new_cyclic_condition(
                    jump_expr,
                    region_nodes,
                    analysis.get_branch_opcode(region_id)?,
                )
                .map_err(|e| StructureAnalysisError::AstNodeError {
                    source: Box::new(e),
                    backtrace: Backtrace::capture(),
                })?
                .into();

                Self::merge_conditional(analysis, region_id, cond)?;
                Self::cleanup_region(analysis, successor.0, region_id, region_id)?;
                return Ok(true);
            }
        }
        Ok(false)
    }
}
