#![deny(missing_docs)]

use std::backtrace::Backtrace;

use crate::{
    decompiler::{
        ast::{
            bin_op::BinOpType, expr::ExprKind, new_assignment, new_bin_op, new_id_with_version,
            new_num, new_return,
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
pub struct SpecialOneOperandHandler;

impl OpcodeHandler for SpecialOneOperandHandler {
    fn handle_instruction(
        &self,
        context: &mut FunctionDecompilerContext,
        instruction: &Instruction,
    ) -> Result<ProcessedInstruction, FunctionDecompilerError> {
        match instruction.opcode {
            Opcode::Ret => {
                let ret_val = context.pop_expression()?;

                let ret = new_return(ret_val);
                Ok(ProcessedInstructionBuilder::new()
                    .push_to_region(ret.into())
                    .build())
            }
            Opcode::Copy => {
                let operand = context.pop_expression()?;
                context.push_one_node(operand.clone().into())?;
                context.push_one_node(operand.clone().into())?;
                Ok(ProcessedInstructionBuilder::new().build())
            }
            Opcode::GetRegister => {
                let register_id = instruction
                    .operand
                    .as_ref()
                    .ok_or(FunctionDecompilerError::InstructionMustHaveOperand {
                        opcode: instruction.opcode,
                        context: context.get_error_context(),
                        backtrace: Backtrace::capture(),
                    })?
                    .get_number_value()
                    .map_err(|e| FunctionDecompilerError::OperandError {
                        source: e,
                        context: context.get_error_context(),
                        backtrace: Backtrace::capture(),
                    })?;

                let ssa_id = context
                    .register_mapping
                    .get(&(register_id as usize))
                    .ok_or(FunctionDecompilerError::RegisterNotFound {
                        register_id: register_id as usize,
                        context: context.get_error_context(),
                        backtrace: Backtrace::capture(),
                    })?;
                context.push_one_node(ssa_id.clone().into())?;
                Ok(ProcessedInstructionBuilder::new().build())
            }
            Opcode::SetRegister => {
                let register_id = instruction
                    .operand
                    .as_ref()
                    .ok_or(FunctionDecompilerError::InstructionMustHaveOperand {
                        opcode: instruction.opcode,
                        context: context.get_error_context(),
                        backtrace: Backtrace::capture(),
                    })?
                    .get_number_value()
                    .map_err(|e| FunctionDecompilerError::OperandError {
                        source: e,
                        context: context.get_error_context(),
                        backtrace: Backtrace::capture(),
                    })?;

                let register_store = context.pop_expression()?;

                // If register_store is an AssignableKind, we can use it directly
                let (register_map_add, processed_instruction) = match register_store.clone() {
                    ExprKind::Assignable(assignable) => (
                        assignable.clone(),
                        ProcessedInstructionBuilder::new().build(),
                    ),
                    _ => {
                        let var = context.ssa_context.new_ssa_version_for("set_register");
                        let ssa_id = new_id_with_version("set_register", var);
                        let stmt = new_assignment(ssa_id.clone(), register_store.clone());
                        (
                            ssa_id.clone().into(),
                            ProcessedInstructionBuilder::new()
                                .push_to_region(stmt.into())
                                .ssa_id(ssa_id.into())
                                .build(),
                        )
                    }
                };

                // push to the stack
                context.push_one_node(register_store.clone().into())?;

                context
                    .register_mapping
                    .insert(register_id as usize, register_map_add);

                Ok(processed_instruction)
            }
            Opcode::Inc => {
                // Pop the last assignable from the stack, create AST node for assignment + 1, push it back to the stack
                let expr = context.pop_assignable()?;
                let bin_op = new_bin_op(expr.clone(), new_num(1), BinOpType::Add).map_err(|e| {
                    FunctionDecompilerError::AstNodeError {
                        source: e,
                        context: context.get_error_context(),
                        backtrace: Backtrace::capture(),
                    }
                })?;

                // an assignment bumps the version of the lhs
                let mut lhs = expr;
                let ver = context.ssa_context.new_ssa_version_for(&lhs.id_string());
                lhs.set_ssa_version(ver);
                let stmt = new_assignment(lhs.clone(), bin_op);

                context.push_one_node(lhs.clone().into())?;

                Ok(ProcessedInstructionBuilder::new()
                    .push_to_region(stmt.into())
                    .build())
            }
            Opcode::Dec => {
                // Pop the last assignable from the stack, create AST node for assignment + 1, push it back to the stack
                let expr = context.pop_assignable()?;
                let bin_op = new_bin_op(expr.clone(), new_num(1), BinOpType::Sub).map_err(|e| {
                    FunctionDecompilerError::AstNodeError {
                        source: e,
                        context: context.get_error_context(),
                        backtrace: Backtrace::capture(),
                    }
                })?;

                // an assignment bumps the version of the lhs
                let mut lhs = expr;
                let ver = context.ssa_context.new_ssa_version_for(&lhs.id_string());
                lhs.set_ssa_version(ver);
                let stmt = new_assignment(lhs.clone(), bin_op);

                context.push_one_node(lhs.clone().into())?;

                Ok(ProcessedInstructionBuilder::new()
                    .push_to_region(stmt.into())
                    .build())
            }
            Opcode::New => {
                // Pop the last expr from the stack, create AST node for new expr, push it back to the stack
                let new_type = context.pop_expression()?;

                // Create assignment for new node
                let var = context.ssa_context.new_ssa_version_for("gbf_new_obj");
                let ssa_id = new_id_with_version("gbf_new_obj", var);
                let stmt = new_assignment(ssa_id.clone(), new_type.clone());

                context.push_one_node(ssa_id.clone().into())?;

                Ok(ProcessedInstructionBuilder::new()
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
