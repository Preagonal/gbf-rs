#![deny(missing_docs)]

use std::backtrace::Backtrace;

use crate::{
    decompiler::{
        ast::{bin_op::BinOpType, new_array_access, new_assignment, new_bin_op, new_range},
        function_decompiler::FunctionDecompilerError,
        function_decompiler_context::FunctionDecompilerContext,
        ProcessedInstruction, ProcessedInstructionBuilder,
    },
    instruction::Instruction,
    opcode::Opcode,
};

use super::OpcodeHandler;

/// Handles other instructions.
pub struct SpecialThreeOperandHandler;

impl OpcodeHandler for SpecialThreeOperandHandler {
    fn handle_instruction(
        &self,
        context: &mut FunctionDecompilerContext,
        instruction: &Instruction,
    ) -> Result<ProcessedInstruction, FunctionDecompilerError> {
        match instruction.opcode {
            Opcode::AssignArray => {
                let rhs = context.pop_expression()?;
                let lhs_ind = context.pop_expression()?;
                let lhs_arr = context.pop_expression()?;

                let arr_access = new_array_access(lhs_arr, lhs_ind);
                let stmt = new_assignment(arr_access, rhs);

                Ok(ProcessedInstructionBuilder::new()
                    .push_to_region(stmt.into())
                    .build())
            }
            Opcode::InRange => {
                let last = context.pop_expression()?;
                let first = context.pop_expression()?;
                let lhs = context.pop_expression()?;

                let range = new_range(first, last);
                let bin_op = new_bin_op(lhs, range, BinOpType::In).map_err(|e| {
                    FunctionDecompilerError::AstNodeError {
                        source: e,
                        context: context.get_error_context(),
                        backtrace: Backtrace::capture(),
                    }
                })?;

                context.push_one_node(bin_op.into())?;
                Ok(ProcessedInstructionBuilder::new().build())
            }
            _ => Err(FunctionDecompilerError::UnimplementedOpcode {
                opcode: instruction.opcode,
                context: context.get_error_context(),
                backtrace: Backtrace::capture(),
            }),
        }
    }
}
