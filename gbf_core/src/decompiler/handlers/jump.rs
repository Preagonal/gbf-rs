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
            _ => Err(FunctionDecompilerError::UnimplementedOpcode {
                context: context.get_error_context(),
                backtrace: Backtrace::capture(),
            }),
        }
    }
}
