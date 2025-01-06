#![deny(missing_docs)]

use crate::{
    decompiler::{
        function_decompiler::FunctionDecompilerError,
        function_decompiler_context::FunctionDecompilerContext,
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
    ) -> Result<(), FunctionDecompilerError> {
        match instruction.opcode {
            Opcode::Pop => {
                // TODO: Handle popping nodes
                // context.pop_one_node()?;
            }
            _ => {
                return Err(FunctionDecompilerError::UnimplementedOpcode(
                    instruction.opcode,
                    context.current_block_id.unwrap(),
                ));
            }
        }

        Ok(())
    }
}
