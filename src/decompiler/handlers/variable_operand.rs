#![deny(missing_docs)]

use crate::{
    decompiler::{
        ast::{expr::ExprNode, func_call::FunctionCallNode, identifier::IdentifierNode, AstNode},
        execution_state::ExecutionState,
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
        match instruction.opcode {
            Opcode::Call => {
                // The arguments are in the reverse order on the stack in the execution state since
                // we're building an array of params. We also ensure to pop the last state from the
                // stack, which prevents us from accidentally nesting the elements of the array.
                let last_state = context
                    .execution_state_stack
                    .pop()
                    .ok_or(FunctionDecompilerError::ExecutionStateStackEmpty)?;

                // Check that last state is ExecutionState::BuildingArray
                if let ExecutionState::BuildingArray(mut args) = last_state {
                    // Get the function name
                    debug_assert!(!args.is_empty());
                    let function_name = args.pop().unwrap();
                    let function_name = match function_name {
                        ExprNode::Identifier(ident) => Ok(ident),
                        _ => Err(FunctionDecompilerError::InvalidNodeType(
                            context.current_block_id.unwrap(),
                            "Identifier".to_string(),
                            format!("{:?}", function_name),
                        )),
                    }?;

                    // Get the arguments in reverse order
                    let args = args.into_iter().rev().collect::<Vec<_>>();

                    // Create the function call node
                    let function_call_node =
                        Self::create_function_call_node(function_name, args.into_iter().collect());
                    context.push_one_node(function_call_node)?;
                    return Ok(());
                }

                Err(FunctionDecompilerError::UnexpectedExecutionState(
                    ExecutionState::BuildingArray(Vec::new()),
                    last_state,
                ))
            }
            _ => Err(FunctionDecompilerError::UnimplementedOpcode(
                instruction.opcode,
                context.current_block_id.unwrap(),
            )),
        }
    }
}
