#![deny(missing_docs)]

use std::backtrace::Backtrace;

use crate::{
    decompiler::{
        ast::new_id_with_version, function_decompiler::FunctionDecompilerError,
        function_decompiler_context::FunctionDecompilerContext, ProcessedInstruction,
        ProcessedInstructionBuilder,
    },
    instruction::Instruction,
    opcode::Opcode,
};

use super::OpcodeHandler;

/// Handles identifier instructions.
pub struct IdentifierHandler;

impl OpcodeHandler for IdentifierHandler {
    fn handle_instruction(
        &self,
        context: &mut FunctionDecompilerContext,
        instruction: &Instruction,
    ) -> Result<ProcessedInstruction, FunctionDecompilerError> {
        let opcode = instruction.opcode;
        // If we have a variable, we need to use the operand as the identifier name.
        let str_operand = if opcode == Opcode::PushVariable {
            let operand = instruction.operand.as_ref().ok_or(
                FunctionDecompilerError::InstructionMustHaveOperand {
                    opcode: instruction.opcode,
                    context: context.get_error_context(),
                    backtrace: Backtrace::capture(),
                },
            )?;
            operand.to_string()
        } else {
            // Otherwise, we can just use the opcode name (e.g. "player", "level", "this", etc.).
            opcode.to_string().to_lowercase()
        };

        let id = new_id_with_version(
            str_operand.as_str(),
            context.ssa_context.current_version_of_or_new(&str_operand),
        );
        Ok(ProcessedInstructionBuilder::new().ssa_id(id.into()).build())
    }
}
