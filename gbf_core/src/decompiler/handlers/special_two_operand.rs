#![deny(missing_docs)]

use std::backtrace::Backtrace;

use crate::{
    decompiler::{
        ast::{
            expr::ExprKind, new_array_access, new_assignment, new_id_with_version,
            new_member_access, new_new,
        },
        function_decompiler::FunctionDecompilerError,
        function_decompiler_context::FunctionDecompilerContext,
        ProcessedInstruction, ProcessedInstructionBuilder,
    },
    instruction::Instruction,
    opcode::Opcode,
};

use super::OpcodeHandler;

/// Handles other instructions.
pub struct SpecialTwoOperandHandler;

impl OpcodeHandler for SpecialTwoOperandHandler {
    fn handle_instruction(
        &self,
        context: &mut FunctionDecompilerContext,
        instruction: &Instruction,
    ) -> Result<ProcessedInstruction, FunctionDecompilerError> {
        match instruction.opcode {
            Opcode::AccessMember => {
                let mut rhs = context.pop_expression()?;
                let mut lhs = context.pop_expression()?;

                // TODO: Check this logic. If either rhs or lhs is an identifier, strip the version from it
                if let ExprKind::Identifier(mut id) = lhs {
                    id.ssa_version = None;
                    lhs = id.into();
                }
                if let ExprKind::Identifier(mut id) = rhs {
                    id.ssa_version = None;
                    rhs = id.into();
                }

                let ma: ExprKind = new_member_access(lhs, rhs)
                    .map_err(|e| FunctionDecompilerError::AstNodeError {
                        source: e,
                        context: context.get_error_context(),
                        backtrace: Backtrace::capture(),
                    })?
                    .into();
                context.push_one_node(ma.into())?;
                Ok(ProcessedInstructionBuilder::new().build())
            }
            Opcode::Assign => {
                let rhs = context.pop_expression()?;
                let mut lhs = context.pop_expression()?;

                // an assignment bumps the version of the lhs, if it's an identifier
                if let ExprKind::Identifier(mut id) = lhs {
                    let ver = context.ssa_context.new_ssa_version_for(id.id());
                    id.ssa_version = Some(ver);
                    lhs = id.into();
                }
                let stmt = new_assignment(lhs, rhs);

                Ok(ProcessedInstructionBuilder::new()
                    .push_to_region(stmt.into())
                    .build())
            }
            Opcode::AssignArrayIndex => {
                let index = context.pop_expression()?;
                let arr = context.pop_expression()?;

                let array_access = new_array_access(arr, index);

                context.push_one_node(array_access.into())?;
                Ok(ProcessedInstructionBuilder::new().build())
            }
            Opcode::NewObject => {
                let new_type = context.pop_expression()?;
                let arg = context.pop_expression()?;

                let new_node =
                    new_new(new_type, arg).map_err(|e| FunctionDecompilerError::AstNodeError {
                        source: e,
                        context: context.get_error_context(),
                        backtrace: Backtrace::capture(),
                    })?;

                // Create SSA ID for the function call
                let var = context.ssa_context.new_ssa_version_for("new_node");
                let ssa_id = new_id_with_version("new_node", var);
                let stmt = new_assignment(ssa_id.clone(), new_node);

                Ok(ProcessedInstructionBuilder::new()
                    .ssa_id(ssa_id.into())
                    .push_to_region(stmt.into())
                    .build())
            }
            _ => Err(FunctionDecompilerError::UnimplementedOpcode {
                opcode: instruction.opcode,
                context: context.get_error_context(),
                backtrace: Backtrace::capture(),
            }),
        }
    }
}
