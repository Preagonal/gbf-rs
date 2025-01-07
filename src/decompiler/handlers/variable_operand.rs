#![deny(missing_docs)]

use crate::{
    decompiler::{
        ast::{
            expr::{AssignableExpr, ExprNode},
            func_call::FunctionCallNode,
            identifier::IdentifierNode,
            AstNode,
        },
        execution_frame::ExecutionFrame,
        function_decompiler::FunctionDecompilerError,
        function_decompiler_context::FunctionDecompilerContext,
    },
    instruction::Instruction,
    opcode::Opcode,
};

use super::OpcodeHandler;

/// Handles other instructions.
pub struct VariableOperandHandler;

impl VariableOperandHandler {
    fn create_function_call_node(function_name: IdentifierNode, args: Vec<ExprNode>) -> AstNode {
        AstNode::Expression(ExprNode::FunctionCall(FunctionCallNode::new(
            function_name,
            args,
            None,
        )))
    }
}

impl OpcodeHandler for VariableOperandHandler {
    fn handle_instruction(
        &self,
        context: &mut FunctionDecompilerContext,
        instruction: &Instruction,
    ) -> Result<(), FunctionDecompilerError> {
        let current_block_id = context.current_block_id.expect("Block ID should be set");
        match instruction.opcode {
            Opcode::Call => {
                // Ensure the current execution state stack has a frame to pop
                let last_frame = context
                    .block_ast_node_stack
                    .get_mut(&current_block_id)
                    .ok_or(FunctionDecompilerError::CannotPopNode(current_block_id))?
                    .pop()
                    .ok_or(FunctionDecompilerError::ExecutionStackEmpty)?;

                // Ensure the last frame is a BuildingArray
                if let ExecutionFrame::BuildingArray(mut args) = last_frame {
                    // Ensure there is at least one argument (the function name)
                    if args.is_empty() {
                        return Err(FunctionDecompilerError::InvalidNodeType(
                            current_block_id,
                            "Identifier".to_string(),
                            "Empty argument list for Call".to_string(),
                        ));
                    }

                    // Pop the function name (last argument in the array)
                    let function_name = args.pop().unwrap();
                    let function_name = match function_name {
                        ExprNode::Assignable(AssignableExpr::Identifier(ident)) => Ok(ident),
                        _ => Err(FunctionDecompilerError::InvalidNodeType(
                            current_block_id,
                            "Identifier".to_string(),
                            format!("{:?}", function_name),
                        )),
                    }?;

                    // Reverse the remaining arguments to get the correct order
                    let args = args.into_iter().rev().collect::<Vec<_>>();

                    // Create the function call node
                    let function_call_node = Self::create_function_call_node(function_name, args);

                    // Push the function call node back to the execution stack
                    context.push_one_node(function_call_node)?;
                    return Ok(());
                }

                // Handle unexpected execution state
                Err(FunctionDecompilerError::UnexpectedExecutionState(
                    ExecutionFrame::BuildingArray(Vec::new()),
                    last_frame,
                ))
            }
            _ => Err(FunctionDecompilerError::UnimplementedOpcode(
                instruction.opcode,
                current_block_id,
            )),
        }
    }
}
