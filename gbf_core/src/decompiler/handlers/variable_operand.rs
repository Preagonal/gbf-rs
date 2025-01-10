#![deny(missing_docs)]

use crate::{
    decompiler::{
        ast::{
            assignable::AssignableKind, expr::ExprKind, new_fn_call, new_id_with_version, statement,
        },
        execution_frame::ExecutionFrame,
        function_decompiler::FunctionDecompilerError,
        function_decompiler_context::FunctionDecompilerContext,
        ProcessedInstruction, ProcessedInstructionBuilder,
    },
    instruction::Instruction,
    opcode::Opcode,
};

use super::OpcodeHandler;

/// Handles other instructions.
pub struct VariableOperandHandler;

impl OpcodeHandler for VariableOperandHandler {
    fn handle_instruction(
        &self,
        context: &mut FunctionDecompilerContext,
        instruction: &Instruction,
    ) -> Result<ProcessedInstruction, FunctionDecompilerError> {
        let current_block_id = context.current_block_id.expect("Block ID should be set");
        match instruction.opcode {
            Opcode::Call => {
                // Ensure the current execution state stack has a frame to pop
                let last_frame = context
                    .block_ast_node_stack
                    .get_mut(&current_block_id)
                    .ok_or(FunctionDecompilerError::CannotPopNode(current_block_id))?
                    .pop()
                    .ok_or(FunctionDecompilerError::ExecutionStackEmpty)?;

                // Ensure the last frame is a BuildingArray
                if let ExecutionFrame::BuildingArray(mut args) = last_frame {
                    // Ensure there is at least one argument (the function name)
                    if args.is_empty() {
                        return Err(FunctionDecompilerError::InvalidNodeType(
                            current_block_id,
                            "Identifier".to_string(),
                            "Empty argument list for Call".to_string(),
                        ));
                    }

                    // Pop the function name (last argument in the array)
                    let function_name = args.pop().unwrap();
                    let function_name = match function_name {
                        ExprKind::Assignable(AssignableKind::Identifier(ident)) => Ok(ident),
                        _ => Err(FunctionDecompilerError::InvalidNodeType(
                            current_block_id,
                            "Identifier".to_string(),
                            format!("{:?}", function_name),
                        )),
                    }?;

                    // Reverse the remaining arguments to get the correct order
                    let args = args.into_iter().rev().collect::<Vec<_>>();

                    // Create the function call node
                    // TODO: Handle method call case
                    let function_call_node = new_fn_call(function_name, args);

                    // Create SSA ID for the function call
                    let var = context.ssa_context.new_ssa_version_for("fn_call");
                    let ssa_id = new_id_with_version("fn_call", var);
                    let stmt = statement(ssa_id.clone(), function_call_node);

                    return Ok(ProcessedInstructionBuilder::new()
                        .ssa_id(ssa_id.into())
                        .push_to_region(stmt.into())
                        .build());
                }

                // Handle unexpected execution state
                Err(FunctionDecompilerError::UnexpectedExecutionState(
                    ExecutionFrame::BuildingArray(Vec::new()),
                    last_frame,
                ))
            }
            Opcode::EndParams => {
                // Ensure the current execution state stack has a frame to pop
                let last_frame = context
                    .block_ast_node_stack
                    .get_mut(&current_block_id)
                    .ok_or(FunctionDecompilerError::CannotPopNode(current_block_id))?
                    .pop()
                    .ok_or(FunctionDecompilerError::ExecutionStackEmpty)?;

                // Ensure the last frame is a BuildingArray
                if let ExecutionFrame::BuildingArray(args) = last_frame {
                    // Reverse the arguments to get the correct order
                    let args = args.into_iter().rev().collect::<Vec<_>>();

                    return Ok(ProcessedInstructionBuilder::new()
                        .function_parameters(args.into())
                        .build());
                }

                // Handle unexpected execution state
                Err(FunctionDecompilerError::UnexpectedExecutionState(
                    ExecutionFrame::BuildingArray(Vec::new()),
                    last_frame,
                ))
            }
            _ => Err(FunctionDecompilerError::UnimplementedOpcode(
                instruction.opcode,
                current_block_id,
            )),
        }
    }
}
