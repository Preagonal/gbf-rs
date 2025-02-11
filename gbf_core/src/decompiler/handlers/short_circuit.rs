#![deny(missing_docs)]

use std::backtrace::Backtrace;

use crate::{
    decompiler::{
        function_decompiler::FunctionDecompilerError,
        function_decompiler_context::FunctionDecompilerContext, ProcessedInstruction,
        ProcessedInstructionBuilder,
    },
    instruction::Instruction,
    opcode::Opcode,
};

use super::OpcodeHandler;

/// Handles unary operations.
pub struct ShortCircuitHandler;

impl OpcodeHandler for ShortCircuitHandler {
    fn handle_instruction(
        &self,
        context: &mut FunctionDecompilerContext,
        instruction: &Instruction,
    ) -> Result<ProcessedInstruction, FunctionDecompilerError> {
        let condition = context.pop_expression()?;

        // Push the condition back to the stack
        context.push_one_node(condition.clone().into())?;

        match instruction.opcode {
            Opcode::ShortCircuitAnd => {
                // If the operand is falsy, ShortCircuitAnd will jump and not evaluate the other operand.

                Ok(ProcessedInstructionBuilder::new()
                    .jump_condition(condition)
                    .build())
            }
            Opcode::ShortCircuitOr => {
                // If the operand is truthy, ShortCircuitOr will jump and not evaluate the other operand.
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
