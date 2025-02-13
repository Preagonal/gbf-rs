#![deny(missing_docs)]

use std::backtrace::Backtrace;

use crate::{
    decompiler::{
        ast::{bin_op::BinOpType, new_bin_op, new_unary_op, unary_op::UnaryOpType},
        function_decompiler::FunctionDecompilerError,
        function_decompiler_context::FunctionDecompilerContext,
        ProcessedInstruction, ProcessedInstructionBuilder,
    },
    instruction::Instruction,
    opcode::Opcode,
};

use super::OpcodeHandler;

/// Handles jump instructions.
pub struct JumpHandler;

impl OpcodeHandler for JumpHandler {
    fn handle_instruction(
        &self,
        context: &mut FunctionDecompilerContext,
        instruction: &Instruction,
    ) -> Result<ProcessedInstruction, FunctionDecompilerError> {
        match instruction.opcode {
            Opcode::Jne => {
                let condition = context.pop_expression()?;

                Ok(ProcessedInstructionBuilder::new()
                    .jump_condition(condition)
                    .build())
            }
            Opcode::Jeq => {
                let condition = context.pop_expression()?;
                let wrapped = new_unary_op(condition, UnaryOpType::LogicalNot).map_err(|e| {
                    FunctionDecompilerError::AstNodeError {
                        source: e,
                        context: context.get_error_context(),
                        backtrace: Backtrace::capture(),
                    }
                })?;

                Ok(ProcessedInstructionBuilder::new()
                    .jump_condition(wrapped.into())
                    .build())
            }
            Opcode::With => {
                let condition = context.pop_expression()?;

                Ok(ProcessedInstructionBuilder::new()
                    .jump_condition(condition)
                    .build())
            }
            Opcode::ForEach => {
                // iter_var is the variable that will be assigned to each element in the array
                let iter_var = context.pop_expression()?;
                // arr is the array to iterate over
                let arr = context.pop_expression()?;

                // push arr and iter_var back onto the stack
                context.push_one_node(arr.clone().into())?;
                context.push_one_node(iter_var.clone().into())?;

                // construct a new binary operation node with Foreach as the type
                let bin_op = new_bin_op(iter_var, arr, BinOpType::Foreach).map_err(|e| {
                    FunctionDecompilerError::AstNodeError {
                        source: e,
                        context: context.get_error_context(),
                        backtrace: Backtrace::capture(),
                    }
                })?;

                Ok(ProcessedInstructionBuilder::new()
                    .jump_condition(bin_op.into())
                    .build())
            }
            _ => Err(FunctionDecompilerError::UnimplementedOpcode {
                opcode: instruction.opcode,
                context: context.get_error_context(),
                backtrace: Backtrace::capture(),
            }),
        }
    }
}
