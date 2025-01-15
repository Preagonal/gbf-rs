#![deny(missing_docs)]

use std::backtrace::Backtrace;

use crate::{
    decompiler::{
        ast::{
            assignable::AssignableKind, new_assignment, new_fn_call, new_id, new_id_with_version,
            new_member_access,
        },
        execution_frame::ExecutionFrame,
        function_decompiler::FunctionDecompilerError,
        function_decompiler_context::FunctionDecompilerContext,
        ProcessedInstruction, ProcessedInstructionBuilder,
    },
    instruction::Instruction,
    opcode::Opcode,
};

use super::OpcodeHandler;

/// Handles other instructions.
pub struct BuiltinsHandler;

impl OpcodeHandler for BuiltinsHandler {
    fn handle_instruction(
        &self,
        context: &mut FunctionDecompilerContext,
        instruction: &Instruction,
    ) -> Result<ProcessedInstruction, FunctionDecompilerError> {
        let current_block_id = context.current_block_id;

        let (fn_id, args): (AssignableKind, Vec<_>) = match instruction.opcode {
            Opcode::Char => {
                let args: Vec<_> = vec![context.pop_expression()?];
                (new_id("char").into(), args)
            }
            Opcode::Int => {
                let args: Vec<_> = vec![context.pop_expression()?];
                (new_id("int").into(), args)
            }
            Opcode::Random => {
                let param2 = context.pop_expression()?;
                let param1 = context.pop_expression()?;
                let args: Vec<_> = [param1, param2].to_vec();
                (new_id("random").into(), args)
            }
            Opcode::Abs => {
                let args: Vec<_> = vec![context.pop_expression()?];
                (new_id("abs").into(), args)
            }
            Opcode::Sin => {
                let args: Vec<_> = vec![context.pop_expression()?];
                (new_id("sin").into(), args)
            }
            Opcode::Cos => {
                let args: Vec<_> = vec![context.pop_expression()?];
                (new_id("cos").into(), args)
            }
            Opcode::VecX => {
                let args: Vec<_> = vec![context.pop_expression()?];
                (new_id("vecx").into(), args)
            }
            Opcode::VecY => {
                let args: Vec<_> = vec![context.pop_expression()?];
                (new_id("vecy").into(), args)
            }
            Opcode::Sleep => {
                let args: Vec<_> = vec![context.pop_expression()?];
                (new_id("sleep").into(), args)
            }
            Opcode::ArcTan => {
                let args: Vec<_> = vec![context.pop_expression()?];
                (new_id("arctan").into(), args)
            }
            Opcode::MakeVar => {
                let args: Vec<_> = vec![context.pop_expression()?];
                (new_id("makevar").into(), args)
            }
            Opcode::GetTranslation => {
                let args: Vec<_> = vec![context.pop_expression()?];
                (new_id("_").into(), args)
            }
            Opcode::Min => {
                let param2 = context.pop_expression()?;
                let param1 = context.pop_expression()?;
                let args: Vec<_> = [param1, param2].to_vec();
                (new_id("min").into(), args)
            }
            Opcode::Max => {
                let param2 = context.pop_expression()?;
                let param1 = context.pop_expression()?;
                let args: Vec<_> = [param1, param2].to_vec();
                (new_id("max").into(), args)
            }
            Opcode::WaitFor => {
                let param3 = context.pop_expression()?;
                let param2 = context.pop_expression()?;
                let param1 = context.pop_expression()?;
                let args: Vec<_> = [param1, param2, param3].to_vec();
                (new_id("waitfor").into(), args)
            }
            Opcode::GetAngle => {
                let param2 = context.pop_expression()?;
                let param1 = context.pop_expression()?;
                let args: Vec<_> = [param1, param2].to_vec();
                (new_id("getangle").into(), args)
            }
            Opcode::GetDir => {
                let param2 = context.pop_expression()?;
                let param1 = context.pop_expression()?;
                let args: Vec<_> = [param1, param2].to_vec();
                (new_id("getdir").into(), args)
            }
            Opcode::ObjSubstring => {
                let param2 = context.pop_expression()?;
                let param1 = context.pop_expression()?;
                let args: Vec<_> = [param1, param2].to_vec();
                (
                    new_member_access(context.pop_assignable()?, new_id("substring"))
                        .map_err(|e| FunctionDecompilerError::AstNodeError {
                            source: e,
                            context: context.get_error_context(),
                            backtrace: Backtrace::capture(),
                        })?
                        .into(),
                    args,
                )
            }
            Opcode::ObjTokenize => {
                let args: Vec<_> = vec![context.pop_expression()?];
                (
                    new_member_access(context.pop_assignable()?, new_id("tokenize"))
                        .map_err(|e| FunctionDecompilerError::AstNodeError {
                            source: e,
                            context: context.get_error_context(),
                            backtrace: Backtrace::capture(),
                        })?
                        .into(),
                    args,
                )
            }
            Opcode::ObjStarts => {
                let args: Vec<_> = vec![context.pop_expression()?];
                (
                    new_member_access(context.pop_assignable()?, new_id("starts"))
                        .map_err(|e| FunctionDecompilerError::AstNodeError {
                            source: e,
                            context: context.get_error_context(),
                            backtrace: Backtrace::capture(),
                        })?
                        .into(),
                    args,
                )
            }
            Opcode::ObjEnds => {
                let args: Vec<_> = vec![context.pop_expression()?];
                (
                    new_member_access(context.pop_assignable()?, new_id("ends"))
                        .map_err(|e| FunctionDecompilerError::AstNodeError {
                            source: e,
                            context: context.get_error_context(),
                            backtrace: Backtrace::capture(),
                        })?
                        .into(),
                    args,
                )
            }
            Opcode::ObjPos => {
                let args: Vec<_> = [context.pop_expression()?].to_vec();
                (
                    new_member_access(context.pop_assignable()?, new_id("pos"))
                        .map_err(|e| FunctionDecompilerError::AstNodeError {
                            source: e,
                            context: context.get_error_context(),
                            backtrace: Backtrace::capture(),
                        })?
                        .into(),
                    args,
                )
            }
            Opcode::ObjCharAt => {
                let args: Vec<_> = [context.pop_expression()?].to_vec();
                (
                    new_member_access(context.pop_assignable()?, new_id("charat"))
                        .map_err(|e| FunctionDecompilerError::AstNodeError {
                            source: e,
                            context: context.get_error_context(),
                            backtrace: Backtrace::capture(),
                        })?
                        .into(),
                    args,
                )
            }
            Opcode::ObjLength => {
                let args = vec![];
                (
                    new_member_access(context.pop_assignable()?, new_id("length"))
                        .map_err(|e| FunctionDecompilerError::AstNodeError {
                            source: e,
                            context: context.get_error_context(),
                            backtrace: Backtrace::capture(),
                        })?
                        .into(),
                    args,
                )
            }
            Opcode::ObjLink => {
                let args: Vec<_> = vec![];
                (
                    new_member_access(context.pop_assignable()?, new_id("link"))
                        .map_err(|e| FunctionDecompilerError::AstNodeError {
                            source: e,
                            context: context.get_error_context(),
                            backtrace: Backtrace::capture(),
                        })?
                        .into(),
                    args,
                )
            }
            Opcode::ObjTrim => {
                let args: Vec<_> = vec![];
                (
                    new_member_access(context.pop_assignable()?, new_id("trim"))
                        .map_err(|e| FunctionDecompilerError::AstNodeError {
                            source: e,
                            context: context.get_error_context(),
                            backtrace: Backtrace::capture(),
                        })?
                        .into(),
                    args,
                )
            }
            Opcode::ObjSize => {
                let args: Vec<_> = vec![];
                (
                    new_member_access(context.pop_assignable()?, new_id("size"))
                        .map_err(|e| FunctionDecompilerError::AstNodeError {
                            source: e,
                            context: context.get_error_context(),
                            backtrace: Backtrace::capture(),
                        })?
                        .into(),
                    args,
                )
            }
            Opcode::ObjIndex => {
                let args: Vec<_> = vec![context.pop_expression()?];
                (
                    new_member_access(context.pop_assignable()?, new_id("index"))
                        .map_err(|e| FunctionDecompilerError::AstNodeError {
                            source: e,
                            context: context.get_error_context(),
                            backtrace: Backtrace::capture(),
                        })?
                        .into(),
                    args,
                )
            }
            Opcode::ObjPositions => {
                let args: Vec<_> = vec![context.pop_expression()?];
                (
                    new_member_access(context.pop_assignable()?, new_id("positions"))
                        .map_err(|e| FunctionDecompilerError::AstNodeError {
                            source: e,
                            context: context.get_error_context(),
                            backtrace: Backtrace::capture(),
                        })?
                        .into(),
                    args,
                )
            }
            // TODO: This has no return value
            Opcode::ObjAddString => {
                let args: Vec<_> = vec![context.pop_expression()?];
                (
                    new_member_access(context.pop_assignable()?, new_id("add"))
                        .map_err(|e| FunctionDecompilerError::AstNodeError {
                            source: e,
                            context: context.get_error_context(),
                            backtrace: Backtrace::capture(),
                        })?
                        .into(),
                    args,
                )
            }
            // TODO: This has no return value
            Opcode::ObjRemoveString => {
                let args: Vec<_> = vec![context.pop_expression()?];
                (
                    new_member_access(context.pop_assignable()?, new_id("remove"))
                        .map_err(|e| FunctionDecompilerError::AstNodeError {
                            source: e,
                            context: context.get_error_context(),
                            backtrace: Backtrace::capture(),
                        })?
                        .into(),
                    args,
                )
            }
            // TODO: This has no return value
            Opcode::ObjDeleteString => {
                let args: Vec<_> = vec![context.pop_expression()?];
                (
                    new_member_access(context.pop_assignable()?, new_id("delete"))
                        .map_err(|e| FunctionDecompilerError::AstNodeError {
                            source: e,
                            context: context.get_error_context(),
                            backtrace: Backtrace::capture(),
                        })?
                        .into(),
                    args,
                )
            }
            Opcode::Format => {
                // Ensure the current execution state stack has a frame to pop
                let last_frame = context
                    .block_ast_node_stack
                    .get_mut(&current_block_id)
                    .expect("Block AST node stack should exist. This is a bug.")
                    .pop()
                    .ok_or(FunctionDecompilerError::ExecutionStackEmpty {
                        backtrace: Backtrace::capture(),
                        context: context.get_error_context(),
                    })?;

                // Ensure the last frame is a BuildingArray
                if let ExecutionFrame::BuildingArray(args) = last_frame {
                    // Reverse the arguments to get the correct order
                    let args = args.into_iter().rev().collect::<Vec<_>>();
                    (new_id("format").into(), args)
                } else {
                    // Handle unexpected execution state
                    return Err(FunctionDecompilerError::UnexpectedExecutionState {
                        backtrace: Backtrace::capture(),
                        context: context.get_error_context(),
                    });
                }
            }
            _ => {
                return Err(FunctionDecompilerError::UnimplementedOpcode {
                    context: context.get_error_context(),
                    backtrace: Backtrace::capture(),
                })
            }
        };

        let fn_call = new_fn_call(fn_id, args);

        let var = context.ssa_context.new_ssa_version_for("builtin_fn_call");
        let ssa_id = new_id_with_version("builtin_fn_call", var);
        let stmt = new_assignment(ssa_id.clone(), fn_call);

        Ok(ProcessedInstructionBuilder::new()
            .ssa_id(ssa_id.into())
            .push_to_region(stmt.into())
            .build())
    }
}
