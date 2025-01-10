#![deny(missing_docs)]

use crate::{
    decompiler::{
        ast::{new_float, new_num, new_str},
        function_decompiler::FunctionDecompilerError,
        function_decompiler_context::FunctionDecompilerContext,
        ProcessedInstruction, ProcessedInstructionBuilder,
    },
    instruction::Instruction,
    opcode::Opcode,
    operand::Operand,
};

use super::OpcodeHandler;

/// Handles identifier instructions.
pub struct LiteralHandler;

impl OpcodeHandler for LiteralHandler {
    fn handle_instruction(
        &self,
        context: &mut FunctionDecompilerContext,
        instruction: &Instruction,
    ) -> Result<ProcessedInstruction, FunctionDecompilerError> {
        // For literals, the opcode must contain the literal value as an operand.
        let operand = instruction.operand.as_ref().ok_or(
            FunctionDecompilerError::InstructionMustHaveOperand(instruction.opcode),
        )?;

        let literal = match instruction.opcode {
            Opcode::PushString => new_str(operand.get_string_value()?),
            Opcode::PushNumber => match operand {
                Operand::Float(_) => new_float(operand.get_string_value()?),
                Operand::Number(_) => new_num(operand.get_number_value()?),
                _ => {
                    // We should not hit this case ever.
                    panic!("Invalid operand type for PushNumber opcode: {:?}", operand);
                }
            },
            _ => {
                return Err(FunctionDecompilerError::UnimplementedOpcode(
                    instruction.opcode,
                    context.current_block_id.unwrap(),
                ));
            }
        };

        // let ver = context.ssa_context.new_ssa_version_for("lit");
        // let ssa_id = new_id_with_version("lit", ver);
        // let stmt = statement(ssa_id.clone(), literal);

        // Ok(ProcessedInstruction {
        //     ssa_id: Some(ssa_id.into()),
        //     node_to_push: Some(stmt.into()),
        // })
        context.push_one_node(literal.into())?;
        Ok(ProcessedInstructionBuilder::new().build())
    }
}
