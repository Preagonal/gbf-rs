#![deny(missing_docs)]

use std::backtrace::Backtrace;

use crate::{
    decompiler::{
        ast::{
            new_assignment, new_bool, new_float, new_id_with_version, new_null, new_num, new_str,
        },
        function_decompiler::FunctionDecompilerError,
        function_decompiler_context::FunctionDecompilerContext,
        ProcessedInstruction, ProcessedInstructionBuilder,
    },
    instruction::Instruction,
    opcode::Opcode,
    operand::Operand,
};

use super::OpcodeHandler;

/// Handles identifier instructions.
pub struct LiteralHandler;

impl OpcodeHandler for LiteralHandler {
    fn handle_instruction(
        &self,
        context: &mut FunctionDecompilerContext,
        instruction: &Instruction,
    ) -> Result<ProcessedInstruction, FunctionDecompilerError> {
        // For PushTrue, PushFalse, we do not pop an operand
        let literal = if instruction.opcode == Opcode::PushTrue
            || instruction.opcode == Opcode::PushFalse
        {
            new_bool(instruction.opcode == Opcode::PushTrue)
        } else if instruction.opcode == Opcode::PushNull {
            new_null()
        } else {
            // For literals, the opcode must contain the literal value as an operand.
            let operand = instruction.operand.as_ref().ok_or(
                FunctionDecompilerError::InstructionMustHaveOperand {
                    opcode: instruction.opcode,
                    context: context.get_error_context(),
                    backtrace: Backtrace::capture(),
                },
            )?;

            match instruction.opcode {
                Opcode::PushString => new_str(operand.get_string_value().map_err(|e| {
                    FunctionDecompilerError::OperandError {
                        source: e,
                        context: context.get_error_context(),
                        backtrace: Backtrace::capture(),
                    }
                })?),
                Opcode::PushNumber => match operand {
                    Operand::Float(_) => new_float(operand.get_string_value().map_err(|e| {
                        FunctionDecompilerError::OperandError {
                            source: e,
                            context: context.get_error_context(),
                            backtrace: Backtrace::capture(),
                        }
                    })?),
                    Operand::Number(_) => new_num(operand.get_number_value().map_err(|e| {
                        FunctionDecompilerError::OperandError {
                            source: e,
                            context: context.get_error_context(),
                            backtrace: Backtrace::capture(),
                        }
                    })?),
                    _ => {
                        return Err(FunctionDecompilerError::Other {
                            message: format!("Invalid operand type for PushNumber: {:?}", operand),
                            context: context.get_error_context(),
                            backtrace: Backtrace::capture(),
                        });
                    }
                },
                _ => {
                    return Err(FunctionDecompilerError::UnimplementedOpcode {
                        opcode: instruction.opcode,
                        context: context.get_error_context(),
                        backtrace: Backtrace::capture(),
                    });
                }
            }
        };

        let ver = context.ssa_context.new_ssa_version_for("lit");
        let ssa_id = new_id_with_version("lit", ver);
        let stmt = new_assignment(ssa_id.clone(), literal);

        Ok(ProcessedInstructionBuilder::new()
            .push_to_region(stmt.into())
            .ssa_id(ssa_id.into())
            .build())
        // context.push_one_node(literal.into())?;
        // Ok(ProcessedInstructionBuilder::new().build())
    }
}
