#![deny(missing_docs)]

use crate::{
    decompiler::{
        ast::{new_num, new_str},
        function_decompiler::FunctionDecompilerError,
        function_decompiler_context::FunctionDecompilerContext,
    },
    instruction::Instruction,
    opcode::Opcode,
};

use super::OpcodeHandler;

/// Handles identifier instructions.
pub struct LiteralHandler;

impl OpcodeHandler for LiteralHandler {
    fn handle_instruction(
        &self,
        context: &mut FunctionDecompilerContext,
        instruction: &Instruction,
    ) -> Result<(), FunctionDecompilerError> {
        // For literals, the opcode must contain the literal value as an operand.
        let operand = instruction.operand.as_ref().ok_or(
            FunctionDecompilerError::InstructionMustHaveOperand(instruction.opcode),
        )?;

        match instruction.opcode {
            Opcode::PushString => {
                context.push_one_node(new_str(operand.get_string_value()?).into())?;
            }
            Opcode::PushNumber => {
                context.push_one_node(new_num(operand.get_number_value()?).into())?;
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
