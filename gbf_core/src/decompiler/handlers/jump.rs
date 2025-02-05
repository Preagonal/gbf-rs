#![deny(missing_docs)]

use std::backtrace::Backtrace;

use crate::{
    decompiler::{
        ast::{new_unary_op, unary_op::UnaryOpType},
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
            _ => Err(FunctionDecompilerError::UnimplementedOpcode {
                opcode: instruction.opcode,
                context: context.get_error_context(),
                backtrace: Backtrace::capture(),
            }),
        }
    }
}
