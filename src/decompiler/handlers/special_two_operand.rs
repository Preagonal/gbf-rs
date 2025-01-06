#![deny(missing_docs)]

use crate::{
    decompiler::{
        ast::{expr::ExprNode, member_access::MemberAccessNode, statement::StatementNode, AstNode},
        function_decompiler::FunctionDecompilerError,
        function_decompiler_context::FunctionDecompilerContext,
    },
    instruction::Instruction,
    opcode::Opcode,
};

use super::OpcodeHandler;

/// Handles other instructions.
pub struct SpecialTwoOperandHandler;

impl SpecialTwoOperandHandler {
    fn create_member_access_node(
        lhs: ExprNode,
        rhs: ExprNode,
    ) -> Result<AstNode, FunctionDecompilerError> {
        let member_access = MemberAccessNode::new(Box::new(lhs), Box::new(rhs))?;

        // Convert the member access to an AST node
        Ok(AstNode::Expression(ExprNode::MemberAccess(member_access)))
    }

    fn create_statement_node(
        lhs: ExprNode,
        rhs: ExprNode,
    ) -> Result<AstNode, FunctionDecompilerError> {
        let statement = StatementNode::new(Box::new(lhs), Box::new(rhs))?;

        // Convert the statement to an AST node
        Ok(AstNode::Statement(statement))
    }
}

impl OpcodeHandler for SpecialTwoOperandHandler {
    fn handle_instruction(
        &self,
        context: &mut FunctionDecompilerContext,
        instruction: &Instruction,
    ) -> Result<(), FunctionDecompilerError> {
        let rhs = context.pop_expression()?;
        let lhs = context.pop_expression()?;

        match instruction.opcode {
            Opcode::AccessMember => {
                let member_access_node =
                    SpecialTwoOperandHandler::create_member_access_node(lhs, rhs)?;
                context.push_one_node(member_access_node)?;
            }
            Opcode::Assign => {
                let statement_node = SpecialTwoOperandHandler::create_statement_node(lhs, rhs)?;
                context.push_one_node(statement_node)?;
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
