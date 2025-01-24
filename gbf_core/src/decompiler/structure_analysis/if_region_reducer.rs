#![deny(missing_docs)]

use std::backtrace::Backtrace;

use crate::decompiler::ast::{
    control_flow::ControlFlowNode, expr::ExprKind, new_else, new_if, AstKind,
};

use super::{
    region::{RegionId, RegionType},
    ControlFlowEdgeType, RegionReducer, StructureAnalysis, StructureAnalysisError,
};

/// Reduces an if region.
pub struct IfRegionReducer;

impl IfRegionReducer {
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
        cond: Vec<ControlFlowNode>,
    ) -> Result<(), StructureAnalysisError> {
        let region = analysis.regions.get_mut(region_id.index).ok_or(
            StructureAnalysisError::RegionNotFound {
                region_id,
                backtrace: Backtrace::capture(),
            },
        )?;
        region.push_nodes(cond.into_iter().map(AstKind::ControlFlow).collect());
        region.set_region_type(RegionType::Linear);
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

impl RegionReducer for IfRegionReducer {
    fn reduce_region(
        &mut self,
        analysis: &mut StructureAnalysis,
        region_id: RegionId,
    ) -> Result<bool, StructureAnalysisError> {
        // Step 1: Extract the jump expression
        let jump_expr = Self::extract_jump_expr(analysis, region_id)?;

        // Step 2: Get successors and classify them
        let successors = analysis.get_successors(region_id)?;
        if successors.len() != 2 {
            return Err(StructureAnalysisError::Other {
                message: "Control flow region must have exactly two successors".to_string(),
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

        // Step 3: Determine linear successors
        let branch_linear_successor = analysis.get_single_linear_successor(branch_region_id)?;
        let fallthrough_linear_successor =
            analysis.get_single_linear_successor(fallthrough_region_id)?;
        // Step 4: Handle the different cases
        if let Some(successor) = branch_linear_successor {
            if successor == fallthrough_region_id {
                // Branch linear successor aligns with fallthrough region
                let fallthrough_statements =
                    IfRegionReducer::get_region_nodes(analysis, fallthrough_region_id)?;
                let cond = new_if(jump_expr, fallthrough_statements);

                Self::merge_conditional(analysis, region_id, vec![cond])?;
                Self::cleanup_region(analysis, branch_region_id, region_id, successor)?;
                return Ok(true);
            }
        }

        if let Some(successor) = fallthrough_linear_successor {
            if successor == branch_region_id {
                // Fallthrough linear successor aligns with branch region
                let branch_statements =
                    IfRegionReducer::get_region_nodes(analysis, branch_region_id)?;
                let cond = new_if(jump_expr, branch_statements);

                Self::merge_conditional(analysis, region_id, vec![cond])?;
                Self::cleanup_region(analysis, fallthrough_region_id, region_id, successor)?;
                return Ok(true);
            }
        }

        if let (Some(branch_successor), Some(fallthrough_successor)) =
            (branch_linear_successor, fallthrough_linear_successor)
        {
            // Create if / else statement
            if branch_successor == fallthrough_successor {
                // Both linear successors are the same
                let branch_statements =
                    IfRegionReducer::get_region_nodes(analysis, branch_region_id)?;
                let fallthrough_statements =
                    IfRegionReducer::get_region_nodes(analysis, fallthrough_region_id)?;
                let if_stmnt = new_if(jump_expr, fallthrough_statements);
                let else_stmt = new_else(branch_statements);

                Self::merge_conditional(analysis, region_id, vec![if_stmnt, else_stmt])?;
                Self::cleanup_region(analysis, branch_region_id, region_id, branch_successor)?;
                Self::cleanup_region(
                    analysis,
                    fallthrough_region_id,
                    region_id,
                    fallthrough_successor,
                )?;

                // Finally, add the edge between the original region and the common successor
                analysis.connect_regions(
                    region_id,
                    branch_successor,
                    ControlFlowEdgeType::Branch,
                )?;
                return Ok(true);
            }
        }

        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use crate::decompiler::ast::{new_assignment, new_id};

    use super::*;

    #[test]
    fn test_if_reduce() -> Result<(), StructureAnalysisError> {
        let mut structure_analysis = StructureAnalysis::new();

        let entry_region = structure_analysis.add_region(RegionType::ControlFlow);
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
        structure_analysis.execute()?;

        assert_eq!(structure_analysis.region_graph.node_count(), 1);

        let region = structure_analysis.get_entry_region();
        let region = structure_analysis.get_region(region)?;
        assert_eq!(region.get_nodes().len(), 4);

        // ensure that the final region is a tail region
        assert_eq!(region.get_region_type(), RegionType::Tail);

        Ok(())
    }

    #[test]
    fn test_if_reduce_single_condition_two_ret() -> Result<(), StructureAnalysisError> {
        let mut structure_analysis = StructureAnalysis::new();

        let entry_region = structure_analysis.add_region(RegionType::ControlFlow);
        let region_1 = structure_analysis.add_region(RegionType::Tail);
        let region_2 = structure_analysis.add_region(RegionType::Tail);

        // push nodes to the regions
        structure_analysis
            .push_to_region(entry_region, new_assignment(new_id("foo"), new_id("bar")));
        // set condition for the region
        structure_analysis
            .get_region_mut(entry_region)?
            .set_jump_expr(Some(new_id("foo").into()));
        structure_analysis.push_to_region(region_1, new_assignment(new_id("foo2"), new_id("bar2")));
        structure_analysis.push_to_region(region_2, new_assignment(new_id("foo3"), new_id("bar3")));
        structure_analysis.connect_regions(entry_region, region_1, ControlFlowEdgeType::Branch)?;
        structure_analysis.connect_regions(
            entry_region,
            region_2,
            ControlFlowEdgeType::Fallthrough,
        )?;
        structure_analysis.execute()?;
        assert_eq!(structure_analysis.region_graph.node_count(), 1);
        Ok(())
    }

    #[test]
    fn test_if_else_case() -> Result<(), StructureAnalysisError> {
        let mut structure_analysis = StructureAnalysis::new();

        let entry_region = structure_analysis.add_region(RegionType::ControlFlow);
        let region_1 = structure_analysis.add_region(RegionType::Linear);
        let region_2 = structure_analysis.add_region(RegionType::Linear);
        let region_3 = structure_analysis.add_region(RegionType::Tail);

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
        structure_analysis.push_to_region(region_3, new_assignment(new_id("foo6"), new_id("bar6")));

        structure_analysis.connect_regions(entry_region, region_1, ControlFlowEdgeType::Branch)?;
        structure_analysis.connect_regions(
            entry_region,
            region_2,
            ControlFlowEdgeType::Fallthrough,
        )?;
        structure_analysis.connect_regions(region_1, region_3, ControlFlowEdgeType::Branch)?;
        structure_analysis.connect_regions(region_2, region_3, ControlFlowEdgeType::Branch)?;
        structure_analysis.execute()?;
        assert_eq!(structure_analysis.region_graph.node_count(), 1);

        Ok(())
    }
}
