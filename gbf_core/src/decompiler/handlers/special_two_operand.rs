#![deny(missing_docs)]

use std::backtrace::Backtrace;

use crate::{
    decompiler::{
        ast::{assignable::AssignableKind, new_array_access, new_assignment, new_member_access},
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

                let mut ma: AssignableKind = new_member_access(lhs, rhs)?.into();
                let ver = context
                    .ssa_context
                    .current_version_of_or_new(&ma.id_string());
                ma.set_ssa_version(ver);
                Ok(ProcessedInstructionBuilder::new().ssa_id(ma).build())
            }
            Opcode::Assign => {
                let rhs = context.pop_expression()?;
                let mut lhs = context.pop_assignable()?;

                // an assignment bumps the version of the lhs
                let ver = context.ssa_context.new_ssa_version_for(&lhs.id_string());
                lhs.set_ssa_version(ver);
                let stmt = new_assignment(lhs, rhs);

                Ok(ProcessedInstructionBuilder::new()
                    .push_to_region(stmt.into())
                    .build())
            }
            Opcode::AssignArrayIndex => {
                let index = context.pop_expression()?;
                let arr = context.pop_assignable()?;

                let array_access = new_array_access(arr, index);

                context.push_one_node(array_access.into())?;
                Ok(ProcessedInstructionBuilder::new().build())
            }
            _ => Err(FunctionDecompilerError::UnimplementedOpcode {
                context: context.get_error_context(),
                backtrace: Backtrace::capture(),
            }),
        }
    }
}
