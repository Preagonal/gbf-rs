#![deny(missing_docs)]

use std::backtrace::Backtrace;

use crate::{
    decompiler::{
        ast::{
            assignable::AssignableKind, new_array_access, new_assignment, new_id_with_version,
            new_member_access, new_new,
        },
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

                let mut ma: AssignableKind = new_member_access(lhs, rhs)
                    .map_err(|e| FunctionDecompilerError::AstNodeError {
                        source: e,
                        context: context.get_error_context(),
                        backtrace: Backtrace::capture(),
                    })?
                    .into();
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
            Opcode::NewObject => {
                let new_type = context.pop_expression()?;
                let arg = context.pop_expression()?;

                let new_node =
                    new_new(new_type, arg).map_err(|e| FunctionDecompilerError::AstNodeError {
                        source: e,
                        context: context.get_error_context(),
                        backtrace: Backtrace::capture(),
                    })?;

                // Create SSA ID for the function call
                let var = context.ssa_context.new_ssa_version_for("new_node");
                let ssa_id = new_id_with_version("new_node", var);
                let stmt = new_assignment(ssa_id.clone(), new_node);

                Ok(ProcessedInstructionBuilder::new()
                    .ssa_id(ssa_id.into())
                    .push_to_region(stmt.into())
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
