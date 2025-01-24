#![deny(missing_docs)]

use std::backtrace::Backtrace;

use crate::{
    decompiler::{
        ast::{bin_op::BinOpType, new_bin_op},
        function_decompiler::FunctionDecompilerError,
        function_decompiler_context::FunctionDecompilerContext,
        ProcessedInstruction, ProcessedInstructionBuilder,
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
    ) -> Result<ProcessedInstruction, FunctionDecompilerError> {
        let rhs = context.pop_expression()?;
        let lhs = context.pop_expression()?;

        let op_type = match instruction.opcode {
            Opcode::Add => BinOpType::Add,
            Opcode::Subtract => BinOpType::Sub,
            Opcode::Multiply => BinOpType::Mul,
            Opcode::Divide => BinOpType::Div,
            Opcode::Modulo => BinOpType::Mod,
            Opcode::BitwiseAnd => BinOpType::And,
            Opcode::BitwiseOr => BinOpType::Or,
            Opcode::BitwiseXor => BinOpType::Xor,
            Opcode::ShiftLeft => BinOpType::ShiftLeft,
            Opcode::ShiftRight => BinOpType::ShiftRight,
            Opcode::Equal => BinOpType::Equal,
            Opcode::NotEqual => BinOpType::NotEqual,
            Opcode::LessThan => BinOpType::Less,
            Opcode::LessThanOrEqual => BinOpType::LessOrEqual,
            Opcode::GreaterThan => BinOpType::Greater,
            Opcode::GreaterThanOrEqual => BinOpType::GreaterOrEqual,
            Opcode::ShortCircuitAnd => BinOpType::LogicalAnd,
            Opcode::ShortCircuitOr => BinOpType::LogicalOr,
            Opcode::In => BinOpType::In,
            Opcode::Join => BinOpType::Join,
            Opcode::Power => BinOpType::Power,
            _ => {
                return Err(FunctionDecompilerError::UnimplementedOpcode {
                    context: context.get_error_context(),
                    backtrace: Backtrace::capture(),
                });
            }
        };

        let op =
            new_bin_op(lhs, rhs, op_type).map_err(|e| FunctionDecompilerError::AstNodeError {
                source: e,
                context: context.get_error_context(),
                backtrace: Backtrace::capture(),
            })?;
        // let var = context.ssa_context.new_ssa_version_for("bin_op");
        // let ssa_id = new_id_with_version("bin_op", var);
        // let stmt = new_assignment(ssa_id.clone(), op);
        context.push_one_node(op.into())?;

        Ok(ProcessedInstructionBuilder::new().build())
    }
}
