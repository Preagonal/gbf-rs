#![deny(missing_docs)]

use crate::{
    decompiler::{
        ast::{member_access, statement},
        function_decompiler::FunctionDecompilerError,
        function_decompiler_context::FunctionDecompilerContext,
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
    ) -> Result<(), FunctionDecompilerError> {
        match instruction.opcode {
            Opcode::AccessMember => {
                let rhs = context.pop_assignable()?;
                let lhs = context.pop_assignable()?;
                context.push_one_node(member_access(lhs, rhs)?.into())?;
            }
            Opcode::Assign => {
                let rhs = context.pop_expression()?;
                let lhs = context.pop_assignable()?;
                context.push_one_node(statement(lhs, rhs).into())?;
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
