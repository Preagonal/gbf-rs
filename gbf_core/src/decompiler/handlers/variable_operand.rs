#![deny(missing_docs)]

use std::backtrace::Backtrace;

use crate::{
    decompiler::{
        ast::{expr::ExprKind, new_array, new_assignment, new_fn_call, new_id_with_version},
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
        let current_block_id = context.current_block_id;
        let error_context = context.get_error_context();
        match instruction.opcode {
            Opcode::Call => {
                // Ensure the current execution state stack has a frame to pop
                let last_frame = context
                    .block_ast_node_stack
                    .get_mut(&current_block_id)
                    .ok_or(FunctionDecompilerError::Other {
                        message: "The AST node stack does not contain a vector of execution frames for this basic block.".to_string(),
                        context: error_context.clone(),
                        backtrace: Backtrace::capture(),
                    })?
                    .pop()
                    .ok_or(FunctionDecompilerError::ExecutionStackEmpty {
                        backtrace: Backtrace::capture(),
                        context: context.get_error_context(),
                    })?;

                // Ensure the last frame is a BuildingArray
                if let ExecutionFrame::BuildingArray(mut args) = last_frame {
                    // Ensure there is at least one argument (the function name)
                    if args.is_empty() {
                        return Err(FunctionDecompilerError::Other {
                            message: "The function call ExecutionFrame was empty when it was expected to have the function name.".to_string(),
                            context: error_context.clone(),
                            backtrace: Backtrace::capture(),
                        });
                    }

                    // Pop the function name (last argument in the array)
                    let function_name = args.pop().unwrap();
                    let function_name = match function_name {
                        ExprKind::Assignable(ident) => Ok(ident),
                        _ => Err(FunctionDecompilerError::UnexpectedNodeType {
                            expected: "Identifier".to_string(),
                            context: error_context.clone(),
                            backtrace: Backtrace::capture(),
                        }),
                    }?;

                    // Reverse the remaining arguments to get the correct order
                    let args = args.into_iter().rev().collect::<Vec<_>>();

                    // Create the function call node
                    // TODO: Handle method call case
                    let function_call_node = new_fn_call(function_name, args);

                    // Create SSA ID for the function call
                    let var = context.ssa_context.new_ssa_version_for("fn_call");
                    let ssa_id = new_id_with_version("fn_call", var);
                    let stmt = new_assignment(ssa_id.clone(), function_call_node);

                    return Ok(ProcessedInstructionBuilder::new()
                        .ssa_id(ssa_id.into())
                        .push_to_region(stmt.into())
                        .build());
                }

                // Handle unexpected execution state
                Err(FunctionDecompilerError::UnexpectedExecutionState {
                    backtrace: Backtrace::capture(),
                    context: context.get_error_context(),
                })
            }
            Opcode::EndParams => {
                // Ensure the current execution state stack has a frame to pop
                let last_frame = context
                    .block_ast_node_stack
                    .get_mut(&current_block_id)
                    .ok_or(FunctionDecompilerError::Other {
                        message: "The AST node stack does not contain a vector of execution frames for this basic block.".to_string(),
                        context: error_context.clone(),
                        backtrace: Backtrace::capture(),
                    })?
                    .pop()
                    .ok_or(FunctionDecompilerError::ExecutionStackEmpty {
                        backtrace: Backtrace::capture(),
                        context: error_context.clone(),
                    })?;

                // Ensure the last frame is a BuildingArray
                if let ExecutionFrame::BuildingArray(args) = last_frame {
                    // Reverse the arguments to get the correct order
                    let args = args.into_iter().rev().collect::<Vec<_>>();

                    return Ok(ProcessedInstructionBuilder::new()
                        .function_parameters(args.into())
                        .build());
                }

                // Handle unexpected execution state
                Err(FunctionDecompilerError::UnexpectedExecutionState {
                    backtrace: Backtrace::capture(),
                    context: context.get_error_context(),
                })
            }
            Opcode::EndArray => {
                // Ensure the current execution state stack has a frame to pop
                let last_frame = context
                    .block_ast_node_stack
                    .get_mut(&current_block_id)
                    .ok_or(FunctionDecompilerError::Other {
                        message: "The AST node stack does not contain a vector of execution frames for this basic block.".to_string(),
                        context: error_context.clone(),
                        backtrace: Backtrace::capture(),
                    })?
                    .pop()
                    .ok_or(FunctionDecompilerError::ExecutionStackEmpty {
                        backtrace: Backtrace::capture(),
                        context: error_context.clone(),
                    })?;

                // Ensure the last frame is a BuildingArray
                if let ExecutionFrame::BuildingArray(args) = last_frame {
                    // Reverse the arguments to get the correct order
                    let args = args.into_iter().rev().collect::<Vec<_>>();
                    let array_node = new_array(args);
                    context.push_one_node(array_node.into())?;
                    return Ok(ProcessedInstructionBuilder::new().build());
                }

                // Handle unexpected execution state
                Err(FunctionDecompilerError::UnexpectedExecutionState {
                    backtrace: Backtrace::capture(),
                    context: context.get_error_context(),
                })
            }
            _ => Err(FunctionDecompilerError::UnimplementedOpcode {
                opcode: instruction.opcode,
                context: context.get_error_context(),
                backtrace: Backtrace::capture(),
            }),
        }
    }
}
