#![deny(missing_docs)]

use crate::{
    decompiler::{
        ast::{assignable::AssignableKind, member_access, statement},
        function_decompiler::FunctionDecompilerError,
        function_decompiler_context::FunctionDecompilerContext,
        ProcessedInstruction, ProcessedInstructionBuilder,
    },
    instruction::Instruction,
    opcode::Opcode,
};

use super::OpcodeHandler;

/// Handles other instructions.
pub struct SpecialTwoOperandHandler;

impl OpcodeHandler for SpecialTwoOperandHandler {
    fn handle_instruction(
        &self,
        context: &mut FunctionDecompilerContext,
        instruction: &Instruction,
    ) -> Result<ProcessedInstruction, FunctionDecompilerError> {
        match instruction.opcode {
            Opcode::AccessMember => {
                let rhs = context.pop_assignable()?;
                let lhs = context.pop_assignable()?;

                let mut ma: AssignableKind = member_access(lhs, rhs)?.into();
                let ver = context
                    .ssa_context
                    .current_version_of_or_new(&ma.ssa_string());
                ma.set_ssa_version(ver);
                Ok(ProcessedInstructionBuilder::new().ssa_id(ma).build())
            }
            Opcode::Assign => {
                let rhs = context.pop_expression()?;
                let mut lhs = context.pop_assignable()?;

                // an assignment bumps the version of the lhs
                let ver = context.ssa_context.new_ssa_version_for(&lhs.ssa_string());
                lhs.set_ssa_version(ver);
                let stmt = statement(lhs, rhs);

                Ok(ProcessedInstructionBuilder::new()
                    .push_to_region(stmt.into())
                    .build())
            }
            _ => Err(FunctionDecompilerError::UnimplementedOpcode(
                instruction.opcode,
                context.current_block_id.unwrap(),
            )),
        }
    }
}
