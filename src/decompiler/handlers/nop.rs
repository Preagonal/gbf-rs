#![deny(missing_docs)]

use crate::{
    decompiler::{
        function_decompiler::FunctionDecompilerError,
        function_decompiler_context::FunctionDecompilerContext,
    },
    instruction::Instruction,
};

use super::OpcodeHandler;

/// Handles instructions that are not useful to our decompiler.
pub struct NopHandler;

impl OpcodeHandler for NopHandler {
    fn handle_instruction(
        &self,
        _context: &mut FunctionDecompilerContext,
        _instruction: &Instruction,
    ) -> Result<(), FunctionDecompilerError> {
        Ok(())
    }
}
