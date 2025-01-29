#![deny(missing_docs)]

use std::backtrace::Backtrace;

use crate::decompiler::ast::{
    control_flow::ControlFlowNode, expr::ExprKind, new_acylic_condition, new_else, new_if, AstKind,
};

use super::{
    region::{RegionId, RegionType},
    ControlFlowEdgeType, RegionReducer, StructureAnalysis, StructureAnalysisError,
};

/// Reduces a tail region.
pub struct TailRegionReducer;

impl TailRegionReducer {
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

    /// Remove the given node and its adjacent edges from the region graph.
    fn cleanup_region(
        analysis: &mut StructureAnalysis,
        remove_node: RegionId,
        start_node: RegionId,
    ) -> Result<(), StructureAnalysisError> {
        analysis.remove_edge(start_node, remove_node)?;
        analysis.remove_node(remove_node)?;
        Ok(())
    }

    /// Handles merging the tail structure into the original region.
    fn merge_tail(
        analysis: &mut StructureAnalysis,
        region_id: RegionId,
        tail: Vec<ControlFlowNode>,
    ) -> Result<(), StructureAnalysisError> {
        let region = analysis.regions.get_mut(region_id.index).ok_or(
            StructureAnalysisError::RegionNotFound {
                region_id,
                backtrace: Backtrace::capture(),
            },
        )?;
        region.push_nodes(tail.into_iter().map(AstKind::ControlFlow).collect());
        region.set_region_type(RegionType::Tail);
        region.remove_jump_expr();
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

impl RegionReducer for TailRegionReducer {
    fn reduce_region(
        &mut self,
        analysis: &mut StructureAnalysis,
        region_id: RegionId,
    ) -> Result<bool, StructureAnalysisError> {
        // This logic applies to control flow regions only
        if analysis.get_region_type(region_id)? != RegionType::ControlFlow {
            return Ok(false);
        }

        // Step 1: Extract the jump expression
        let jump_expr = Self::extract_jump_expr(analysis, region_id)?;

        // Step 2: Get successors
        let successors = analysis.get_successors(region_id)?;
        if successors.len() != 2 {
            return Err(StructureAnalysisError::Other {
                message: "Tail reduction requires exactly two successors".to_string(),
                backtrace: Backtrace::capture(),
            });
        }

        let branch_region_id = successors
            .iter()
            .find(|(_, edge_type)| *edge_type == ControlFlowEdgeType::Branch)
            .map(|(id, _)| *id)
            .ok_or(StructureAnalysisError::Other {
                message: "Control flow region must have a branch successor".to_string(),
                backtrace: Backtrace::capture(),
            })?;

        let fallthrough_region_id = successors
            .iter()
            .find(|(_, edge_type)| *edge_type == ControlFlowEdgeType::Fallthrough)
            .map(|(id, _)| *id)
            .ok_or(StructureAnalysisError::Other {
                message: "Control flow region must have a fallthrough successor".to_string(),
                backtrace: Backtrace::capture(),
            })?;

        let branch_single_pred = analysis.has_single_predecessor(branch_region_id)?;
        let fallthrough_single_pred = analysis.has_single_predecessor(fallthrough_region_id)?;

        // Step 3: Merge if both successors are tail regions with a single predecessor
        if analysis.get_region_type(branch_region_id)? == RegionType::Tail
            && branch_single_pred
            && analysis.get_region_type(fallthrough_region_id)? == RegionType::Tail
            && fallthrough_single_pred
        {
            analysis.before_reduce(region_id);

            let branch_statements = Self::get_region_nodes(analysis, branch_region_id)?;
            let fallthrough_statements = Self::get_region_nodes(analysis, fallthrough_region_id)?;

            let if_else = new_if(jump_expr, fallthrough_statements);
            let else_stmt = new_else(branch_statements);

            Self::merge_tail(analysis, region_id, vec![if_else, else_stmt])?;
            Self::cleanup_region(analysis, branch_region_id, region_id)?;
            Self::cleanup_region(analysis, fallthrough_region_id, region_id)?;
            return Ok(true);
        }

        // Step 4: Merge if the first successor is a tail region with a single predecessor
        if analysis.get_region_type(branch_region_id)? == RegionType::Tail && branch_single_pred {
            analysis.before_reduce(region_id);

            let branch_statements = Self::get_region_nodes(analysis, branch_region_id)?;
            let if_stmt = new_acylic_condition(
                jump_expr,
                branch_statements,
                analysis.get_branch_opcode(region_id)?,
            )
            .map_err(|e| StructureAnalysisError::AstNodeError {
                source: Box::new(e),
                backtrace: Backtrace::capture(),
            })?;

            Self::merge_tail(analysis, region_id, vec![if_stmt])?;
            Self::cleanup_region(analysis, branch_region_id, region_id)?;
            return Ok(true);
        }

        // Step 5: Merge if the second successor is a tail region with a single predecessor
        if analysis.get_region_type(fallthrough_region_id)? == RegionType::Tail
            && fallthrough_single_pred
        {
            analysis.before_reduce(region_id);

            let fallthrough_statements = Self::get_region_nodes(analysis, fallthrough_region_id)?;
            let if_stmt = new_acylic_condition(
                jump_expr,
                fallthrough_statements,
                analysis.get_branch_opcode(region_id)?,
            )
            .map_err(|e| StructureAnalysisError::AstNodeError {
                source: Box::new(e),
                backtrace: Backtrace::capture(),
            })?;

            Self::merge_tail(analysis, region_id, vec![if_stmt])?;
            Self::cleanup_region(analysis, fallthrough_region_id, region_id)?;
            return Ok(true);
        }

        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use crate::decompiler::ast::{emit, new_assignment, new_id};

    use super::*;

    #[test]
    fn test_tail_reduce() -> Result<(), StructureAnalysisError> {
        let mut structure_analysis = StructureAnalysis::new(false, 100);

        let entry_region = structure_analysis.add_region(RegionType::ControlFlow);
        let region_1 = structure_analysis.add_region(RegionType::Tail);
        let region_2 = structure_analysis.add_region(RegionType::Tail);

        // Push nodes to regions
        structure_analysis
            .push_to_region(entry_region, new_assignment(new_id("foo"), new_id("bar")));
        structure_analysis
            .get_region_mut(entry_region)?
            .set_jump_expr(Some(new_id("foo").into()));
        structure_analysis.push_to_region(region_1, new_assignment(new_id("x"), new_id("y")));
        structure_analysis.push_to_region(region_2, new_assignment(new_id("a"), new_id("b")));

        structure_analysis.connect_regions(entry_region, region_1, ControlFlowEdgeType::Branch)?;
        structure_analysis.connect_regions(
            entry_region,
            region_2,
            ControlFlowEdgeType::Fallthrough,
        )?;

        structure_analysis.execute()?;
        assert_eq!(structure_analysis.region_graph.node_count(), 1);

        let region = structure_analysis.get_entry_region();
        let region = structure_analysis.get_region(region)?;

        for node in region.get_nodes() {
            println!("{}", emit(node.clone()));
        }

        assert_eq!(region.get_nodes().len(), 3);

        // Ensure that the final region is a tail region
        assert_eq!(region.get_region_type(), RegionType::Tail);

        Ok(())
    }
}
