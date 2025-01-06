#![deny(missing_docs)]

use crate::{
    decompiler::{
        ast::{expr::ExprNode, literal::LiteralNode, AstNode},
        function_decompiler::FunctionDecompilerError,
        function_decompiler_context::FunctionDecompilerContext,
    },
    instruction::Instruction,
    opcode::Opcode,
};

use super::OpcodeHandler;

/// Handles identifier instructions.
pub struct LiteralHandler;

impl LiteralHandler {
    fn create_string_literal(name: &str) -> AstNode {
        AstNode::Expression(ExprNode::Literal(LiteralNode::new_string(name)))
    }
    fn create_number_literal(value: i32) -> AstNode {
        AstNode::Expression(ExprNode::Literal(LiteralNode::new_number(value)))
    }
}

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
                context.push_one_node(Self::create_string_literal(operand.get_string_value()?));
            }
            Opcode::PushNumber => {
                context.push_one_node(Self::create_number_literal(operand.get_number_value()?));
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
