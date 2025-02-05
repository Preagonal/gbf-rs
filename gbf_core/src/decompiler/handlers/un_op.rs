#![deny(missing_docs)]

use std::backtrace::Backtrace;

use crate::{
    decompiler::{
        ast::{new_unary_op, unary_op::UnaryOpType},
        function_decompiler::FunctionDecompilerError,
        function_decompiler_context::FunctionDecompilerContext,
        ProcessedInstruction, ProcessedInstructionBuilder,
    },
    instruction::Instruction,
    opcode::Opcode,
};

use super::OpcodeHandler;

/// Handles unary operations.
pub struct UnaryOperationHandler;

impl OpcodeHandler for UnaryOperationHandler {
    fn handle_instruction(
        &self,
        context: &mut FunctionDecompilerContext,
        instruction: &Instruction,
    ) -> Result<ProcessedInstruction, FunctionDecompilerError> {
        let expr = context.pop_expression()?;

        let op_type: UnaryOpType = match instruction.opcode {
            Opcode::LogicalNot => UnaryOpType::LogicalNot,
            Opcode::BitwiseInvert => UnaryOpType::BitwiseNot,
            Opcode::UnarySubtract => UnaryOpType::Negate,
            _ => {
                return Err(FunctionDecompilerError::UnimplementedOpcode {
                    opcode: instruction.opcode,
                    context: context.get_error_context(),
                    backtrace: Backtrace::capture(),
                });
            }
        };

        let op =
            new_unary_op(expr, op_type).map_err(|e| FunctionDecompilerError::AstNodeError {
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
