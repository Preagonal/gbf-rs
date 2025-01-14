#![deny(missing_docs)]

use crate::{
    decompiler::{
        ast::{member_access, new_fn_call, new_id, new_id_with_version, statement},
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
        let (fn_id, args) = match instruction.opcode {
            Opcode::ObjPos => {
                let args: Vec<_> = [context.pop_expression()?].to_vec();
                (
                    member_access(context.pop_assignable()?, new_id("pos"))?,
                    args,
                )
            }
            _ => {
                return Err(FunctionDecompilerError::UnimplementedOpcode(
                    instruction.opcode,
                    context.current_block_id.unwrap(),
                ))
            }
        };

        let fn_call = new_fn_call(fn_id, args);

        let var = context.ssa_context.new_ssa_version_for("builtin_fn_call");
        let ssa_id = new_id_with_version("builtin_fn_call", var);
        let stmt = statement(ssa_id.clone(), fn_call);

        Ok(ProcessedInstructionBuilder::new()
            .ssa_id(ssa_id.into())
            .push_to_region(stmt.into())
            .build())
    }
}
