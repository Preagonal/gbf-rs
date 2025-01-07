#![deny(missing_docs)]

use crate::basic_block::BasicBlockId;
use crate::instruction::Instruction;
use crate::opcode::Opcode;
use std::collections::HashMap;

use super::ast::expr::{AssignableExpr, ExprNode};
use super::ast::identifier::IdentifierNode;
use super::ast::AstNode;
use super::execution_frame::ExecutionFrame;
use super::function_decompiler::FunctionDecompilerError;
use super::handlers::{global_opcode_handlers, OpcodeHandler};

/// Manages the state of the decompiler, including per-block AST stacks and current processing context.
pub struct FunctionDecompilerContext {
    /// AST node stacks for each basic block.
    pub block_ast_node_stack: HashMap<BasicBlockId, Vec<ExecutionFrame>>,
    /// The current basic block being processed.
    pub current_block_id: Option<BasicBlockId>,
    /// The handlers for each opcode.
    pub opcode_handlers: HashMap<Opcode, Box<dyn OpcodeHandler>>,
}

impl FunctionDecompilerContext {
    /// Creates a new, empty context.
    pub fn new() -> Self {
        Self {
            block_ast_node_stack: HashMap::new(),
            current_block_id: None,
            opcode_handlers: HashMap::new(),
        }
    }

    /// Starts processing a new basic block.
    ///
    /// # Arguments
    /// - `block_id`: The ID of the basic block to start processing.
    ///
    /// # Errors
    /// - Returns `FunctionDecompilerError` if there is an issue initializing the block stack.
    pub fn start_block_processing(
        &mut self,
        block_id: BasicBlockId,
    ) -> Result<(), FunctionDecompilerError> {
        self.current_block_id = Some(block_id);
        self.block_ast_node_stack.insert(block_id, Vec::new());
        Ok(())
    }

    /// Processes an instruction and updates the AST stack.
    ///
    /// # Arguments
    /// - `instr`: The instruction to process.
    ///
    /// # Errors
    /// - Returns `FunctionDecompilerError` for invalid or unexpected instructions.
    pub fn process_instruction(
        &mut self,
        instr: &Instruction,
    ) -> Result<(), FunctionDecompilerError> {
        let current_block_id = self
            .current_block_id
            .expect("Critical error: current block id should always be set");

        // TODO: Better handle PushArray
        if instr.opcode == Opcode::PushArray {
            let stack = self
                .block_ast_node_stack
                .get_mut(&current_block_id)
                .expect("Critical error: stack should always be set for each basic block");

            stack.push(ExecutionFrame::BuildingArray(Vec::new()));
            return Ok(());
        }

        let handlers = global_opcode_handlers();

        let handler =
            handlers
                .get(&instr.opcode)
                .ok_or(FunctionDecompilerError::UnimplementedOpcode(
                    instr.opcode,
                    current_block_id,
                ))?;

        handler.handle_instruction(self, instr)
    }

    /// Retrieves the AST stack for a basic block.
    pub fn get_stack(&self, block_id: &BasicBlockId) -> Option<&Vec<ExecutionFrame>> {
        self.block_ast_node_stack.get(block_id)
    }

    /// Pops an AST node from the current basic block's stack.
    pub fn pop_one_node(&mut self) -> Result<AstNode, FunctionDecompilerError> {
        let block_id = self
            .current_block_id
            .expect("Critical error: current block id should always be set");
        let stack = self
            .block_ast_node_stack
            .get_mut(&block_id)
            .expect("Critical error: stack should always be set for each basic block");

        // Ensure there's a frame to pop from
        let mut last_frame = stack
            .pop()
            .ok_or(FunctionDecompilerError::ExecutionStackEmpty)?;

        let result = match &mut last_frame {
            ExecutionFrame::BuildingArray(array) => {
                // Pop the node from the array
                if let Some(node) = array.pop() {
                    Ok(AstNode::Expression(node))
                } else {
                    Err(FunctionDecompilerError::ExecutionStackEmpty)
                }
            }
            ExecutionFrame::StandaloneNode(node) => Ok(node.clone()),
            ExecutionFrame::None => Err(FunctionDecompilerError::ExecutionStackEmpty),
        };

        // Push the last frame back onto the stack, even if it's empty
        if let ExecutionFrame::BuildingArray(_) = last_frame {
            stack.push(last_frame);
        }

        result
    }

    /// Pops an expression from the current basic block's stack.
    pub fn pop_expression(&mut self) -> Result<ExprNode, FunctionDecompilerError> {
        let node = self.pop_one_node()?;
        match node {
            AstNode::Expression(expr) => Ok(expr),
            _ => Err(FunctionDecompilerError::InvalidNodeType(
                self.current_block_id.unwrap(),
                "Expression".to_string(),
                format!("{:?}", node),
            )),
        }
    }

    /// Pops an assignable expression from the current basic block's stack.
    pub fn pop_assignable(&mut self) -> Result<AssignableExpr, FunctionDecompilerError> {
        let node = self.pop_expression()?;
        match node {
            ExprNode::Assignable(assignable) => Ok(assignable),
            _ => Err(FunctionDecompilerError::InvalidNodeType(
                self.current_block_id.unwrap(),
                "Assignable".to_string(),
                format!("{:?}", node),
            )),
        }
    }

    /// Pops an identifier from the current basic block's stack.
    pub fn pop_identifier(&mut self) -> Result<IdentifierNode, FunctionDecompilerError> {
        let node = self.pop_expression()?;
        match node {
            ExprNode::Assignable(AssignableExpr::Identifier(ident)) => Ok(ident),
            _ => Err(FunctionDecompilerError::InvalidNodeType(
                self.current_block_id.unwrap(),
                "Identifier".to_string(),
                format!("{:?}", node),
            )),
        }
    }

    /// Pushes an AST node to the current basic block's stack.
    pub fn push_one_node(&mut self, node: AstNode) -> Result<(), FunctionDecompilerError> {
        let block_id = self
            .current_block_id
            .expect("Critical error: current block id should always be set");

        let stack = self
            .block_ast_node_stack
            .get_mut(&block_id)
            .expect("Critical error: stack should always be set for each basic block");

        // Check if we're in a frame that requires special handling
        if let Some(last_frame) = stack.last_mut() {
            match last_frame {
                ExecutionFrame::BuildingArray(array) => {
                    // Ensure the node is an expression before adding to the array
                    if let AstNode::Expression(expr) = node {
                        array.push(expr);
                        return Ok(());
                    } else {
                        return Err(FunctionDecompilerError::InvalidNodeType(
                            block_id,
                            "Expression".to_string(),
                            format!("{:?}", node),
                        ));
                    }
                }
                ExecutionFrame::StandaloneNode(_) | ExecutionFrame::None => {
                    // Do nothing; let it fall through to the standalone push below
                }
            }
        }

        // If no special frame handling is required, push the node directly onto the stack
        stack.push(ExecutionFrame::StandaloneNode(node));
        Ok(())
    }
}

// == Other Implementations ==
impl Default for FunctionDecompilerContext {
    fn default() -> Self {
        Self::new()
    }
}
