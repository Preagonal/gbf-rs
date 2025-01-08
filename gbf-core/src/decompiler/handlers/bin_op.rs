#![deny(missing_docs)]

use crate::{
    decompiler::{
        ast::{
            bin_op::{BinOpType, BinaryOperationNode},
            expr::ExprKind,
            AstKind,
        },
        function_decompiler::FunctionDecompilerError,
        function_decompiler_context::FunctionDecompilerContext,
    },
    instruction::Instruction,
    opcode::Opcode,
};

use super::OpcodeHandler;

/// Handles identifier instructions.
pub struct BinaryOperationHandler;

impl BinaryOperationHandler {
    fn create_binary_operation_node(
        bin_op_type: BinOpType,
        left: ExprKind,
        right: ExprKind,
    ) -> Result<AstKind, FunctionDecompilerError> {
        Ok(AstKind::Expression(ExprKind::BinOp(
            BinaryOperationNode::new(Box::new(left), Box::new(right), bin_op_type)?,
        )))
    }
}

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
                context.push_one_node(Self::create_binary_operation_node(
                    BinOpType::Add,
                    lhs,
                    rhs,
                )?)?;
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
