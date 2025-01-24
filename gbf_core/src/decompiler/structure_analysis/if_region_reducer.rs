#![deny(missing_docs)]

use std::backtrace::Backtrace;

use crate::decompiler::ast::new_if;

use super::{
    region::{RegionId, RegionType},
    ControlFlowEdgeType, RegionReducer, StructureAnalysis, StructureAnalysisError,
};

/// Reduces an if region.
pub struct IfRegionReducer;

impl IfRegionReducer {}

impl RegionReducer for IfRegionReducer {
    fn reduce_region(
        &mut self,
        analysis: &mut StructureAnalysis,
        region_id: RegionId,
    ) -> Result<bool, StructureAnalysisError> {
        // Step 1: Extract the current region
        let jump_expr = {
            let region = analysis.regions.get_mut(region_id.index).ok_or(
                StructureAnalysisError::RegionNotFound {
                    region_id,
                    backtrace: Backtrace::capture(),
                },
            )?;

            // Get the jump expression and drop the mutable borrow
            region
                .get_jump_expr()
                .ok_or(StructureAnalysisError::ExpectedConditionNotFound {
                    backtrace: Backtrace::capture(),
                })?
                .clone()
        };

        // Step 2: Get the successors of the region
        let successors = analysis.get_successors(region_id)?;
        if successors.len() != 2 {
            return Err(StructureAnalysisError::Other {
                message: "Control flow region must have exactly two successors".to_string(),
                backtrace: Backtrace::capture(),
            });
        }

        // Step 3: Identify branch and fallthrough successors
        let branch_region_id = successors
            .iter()
            .find(|successor| successor.1 == ControlFlowEdgeType::Branch)
            .ok_or(StructureAnalysisError::Other {
                message: "Control flow region must have a branch successor".to_string(),
                backtrace: Backtrace::capture(),
            })?
            .0;

        let fallthrough_region_id = successors
            .iter()
            .find(|successor| successor.1 == ControlFlowEdgeType::Fallthrough)
            .ok_or(StructureAnalysisError::Other {
                message: "Control flow region must have a fallthrough successor".to_string(),
                backtrace: Backtrace::capture(),
            })?
            .0;

        // Step 4: Determine linear successors
        let branch_linear_successor = analysis.get_single_linear_successor(branch_region_id);
        let fallthrough_linear_successor =
            analysis.get_single_linear_successor(fallthrough_region_id)?;

        if let Some(fallthrough_linear_successor) = fallthrough_linear_successor {
            if fallthrough_linear_successor == branch_region_id {
                // Ensure that the fallthrough region has only one predecessor
                let fallthrough_predecessors = analysis.get_predecessors(fallthrough_region_id)?;
                if fallthrough_predecessors.len() != 1 {
                    return Ok(false);
                }

                // Step 5: Extract the fallthrough statements
                let fallthrough_statements = {
                    let fallthrough_region = analysis
                        .regions
                        .get(fallthrough_region_id.index)
                        .ok_or(StructureAnalysisError::RegionNotFound {
                            region_id: fallthrough_region_id,
                            backtrace: Backtrace::capture(),
                        })?;

                    fallthrough_region.get_nodes().to_vec()
                };

                // Step 6: Create the if statement
                let cond = new_if(jump_expr, fallthrough_statements);

                // Step 1: Add the if statement to the region and set its type
                {
                    let region = analysis.regions.get_mut(region_id.index).ok_or(
                        StructureAnalysisError::RegionNotFound {
                            region_id,
                            backtrace: Backtrace::capture(),
                        },
                    )?;
                    region.push_node(cond.into());
                    region.set_region_type(RegionType::Linear);
                    region.remove_jump_expr();
                } // Mutable borrow on `analysis.regions` ends here

                // Step 2: Remove the edges
                analysis.remove_edge(region_id, fallthrough_region_id)?;
                analysis.remove_edge(fallthrough_region_id, fallthrough_linear_successor)?;

                // Step 3: Remove the fallthrough region
                analysis.remove_node(fallthrough_region_id)?;
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
}
