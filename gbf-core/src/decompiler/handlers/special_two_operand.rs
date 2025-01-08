#![deny(missing_docs)]

use crate::{
    decompiler::{
        ast::{
            assignable::AssignableKind, expr::ExprKind, member_access::MemberAccessNode,
            statement::StatementNode, AstKind,
        },
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
        lhs: AssignableKind,
        rhs: AssignableKind,
    ) -> Result<AstKind, FunctionDecompilerError> {
        let member_access = MemberAccessNode::new(Box::new(lhs), Box::new(rhs))?;

        // Convert the member access to an AST node
        Ok(AstKind::Expression(ExprKind::Assignable(
            AssignableKind::MemberAccess(member_access),
        )))
    }

    fn create_statement_node(
        lhs: AssignableKind,
        rhs: ExprKind,
    ) -> Result<AstKind, FunctionDecompilerError> {
        let statement = StatementNode::new(Box::new(lhs), Box::new(rhs))?;

        // Convert the statement to an AST node
        Ok(AstKind::Statement(statement))
    }
}

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
                let member_access_node =
                    SpecialTwoOperandHandler::create_member_access_node(lhs, rhs)?;
                context.push_one_node(member_access_node)?;
            }
            Opcode::Assign => {
                let rhs = context.pop_expression()?;
                let lhs = context.pop_assignable()?;
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
