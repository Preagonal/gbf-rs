#![deny(missing_docs)]

use crate::{
    decompiler::{
        ast::{expr::ExprNode, identifier::IdentifierNode, AstNode},
        function_decompiler::FunctionDecompilerError,
        function_decompiler_context::FunctionDecompilerContext,
    },
    instruction::Instruction,
    opcode::Opcode,
};

use super::OpcodeHandler;

/// Handles identifier instructions.
pub struct IdentifierHandler;

impl IdentifierHandler {
    fn create_identifier(name: &str) -> AstNode {
        AstNode::Expression(ExprNode::Identifier(IdentifierNode::new(name)))
    }
}

impl OpcodeHandler for IdentifierHandler {
    fn handle_instruction(
        &self,
        context: &mut FunctionDecompilerContext,
        instruction: &Instruction,
    ) -> Result<(), FunctionDecompilerError> {
        let opcode = instruction.opcode;
        // If we have a variable, we need to use the operand as the identifier name.
        let str_operand = if opcode == Opcode::PushVariable {
            let operand = instruction.operand.as_ref().ok_or(
                FunctionDecompilerError::InstructionMustHaveOperand(Opcode::PushVariable),
            )?;
            operand.to_string()
        } else {
            // Otherwise, we can just use the opcode name (e.g. "player", "level", "this", etc.).
            opcode.to_string().to_lowercase()
        };
        context.push_one_node(Self::create_identifier(str_operand.as_str()))?;
        Ok(())
    }
}
