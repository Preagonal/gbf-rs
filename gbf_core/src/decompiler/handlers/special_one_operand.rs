#![deny(missing_docs)]

use crate::{
    decompiler::{
        ast::create_return, function_decompiler::FunctionDecompilerError,
        function_decompiler_context::FunctionDecompilerContext, ProcessedInstruction,
        ProcessedInstructionBuilder,
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

                let ret = create_return(ret_val);
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
            _ => Err(FunctionDecompilerError::UnimplementedOpcode(
                instruction.opcode,
                context.current_block_id.unwrap(),
            )),
        }
    }
}
