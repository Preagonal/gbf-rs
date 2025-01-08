#![deny(missing_docs)]

use crate::{
    decompiler::{
        ast::{bin_op::BinOpType, new_bin_op},
        function_decompiler::FunctionDecompilerError,
        function_decompiler_context::FunctionDecompilerContext,
    },
    instruction::Instruction,
    opcode::Opcode,
};

use super::OpcodeHandler;

/// Handles identifier instructions.
pub struct BinaryOperationHandler;

impl OpcodeHandler for BinaryOperationHandler {
    fn handle_instruction(
        &self,
        context: &mut FunctionDecompilerContext,
        instruction: &Instruction,
    ) -> Result<(), FunctionDecompilerError> {
        let rhs = context.pop_expression()?;
        let lhs = context.pop_expression()?;
        match instruction.opcode {
            Opcode::Add => {
                context.push_one_node(new_bin_op(lhs, rhs, BinOpType::Add)?.into())?;
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
