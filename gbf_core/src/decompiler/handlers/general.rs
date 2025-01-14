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

/// Handles identifier instructions.
pub struct GeneralHandler;

impl OpcodeHandler for GeneralHandler {
    fn handle_instruction(
        &self,
        context: &mut FunctionDecompilerContext,
        instruction: &Instruction,
    ) -> Result<ProcessedInstruction, FunctionDecompilerError> {
        match instruction.opcode {
            Opcode::Pop => {
                // TODO: Handle popping nodes
                context.pop_one_node()?;
            }
            _ => {
                return Err(FunctionDecompilerError::UnimplementedOpcode {
                    context: context.get_error_context(),
                    backtrace: Backtrace::capture(),
                });
            }
        }

        Ok(ProcessedInstructionBuilder::new().build())
    }
}
