#![deny(missing_docs)]

use crate::basic_block::BasicBlockId;
use crate::instruction::Instruction;
use crate::opcode::Opcode;
use std::collections::HashMap;

use super::ast::expr::ExprNode;
use super::ast::identifier::IdentifierNode;
use super::ast::AstNode;
use super::execution_state::ExecutionState;
use super::function_decompiler::FunctionDecompilerError;
use super::handlers::{global_opcode_handlers, OpcodeHandler};

/// Manages the state of the decompiler, including per-block AST stacks and current processing context.
pub struct FunctionDecompilerContext {
    /// AST node stacks for each basic block.
    pub block_ast_node_stack: HashMap<BasicBlockId, Vec<AstNode>>,
    /// The current basic block being processed.
    pub current_block_id: Option<BasicBlockId>,
    /// The handlers for each opcode.
    pub opcode_handlers: HashMap<Opcode, Box<dyn OpcodeHandler>>,
    /// The stack of states for the decompiler (used for nested constructs).
    pub execution_state_stack: Vec<ExecutionState>,
}

impl FunctionDecompilerContext {
    /// Creates a new, empty context.
    pub fn new() -> Self {
        Self {
            block_ast_node_stack: HashMap::new(),
            current_block_id: None,
            opcode_handlers: HashMap::new(),
            execution_state_stack: Vec::new(),
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
        // TODO: Handle pop instructions
        if instr.opcode == Opcode::Pop {
            //self.pop_one_node()?;
            return Ok(());
        }
        // TODO: Better handle PushArray
        if instr.opcode == Opcode::PushArray {
            self.execution_state_stack
                .push(ExecutionState::BuildingArray(Vec::new()));
            return Ok(());
        }

        let handlers = global_opcode_handlers();

        let handler =
            handlers
                .get(&instr.opcode)
                .ok_or(FunctionDecompilerError::UnimplementedOpcode(
                    instr.opcode,
                    self.current_block_id.unwrap(),
                ))?;

        handler.handle_instruction(self, instr)
    }

    /// Retrieves the AST stack for a basic block.
    pub fn get_stack(&self, block_id: &BasicBlockId) -> Option<&Vec<AstNode>> {
        self.block_ast_node_stack.get(block_id)
    }

    /// Pops an AST node from the current basic block's stack.
    pub fn pop_one_node(&mut self) -> Result<AstNode, FunctionDecompilerError> {
        // If we are building an array, we need pop the last node from the last state in the stack.
        // TODO: Replace this code with a more robust solution. I don't think that this function should
        // know or care about the execution stack. We should figure out a better way to handle this.
        if let Some(ExecutionState::BuildingArray(array)) = self.execution_state_stack.last_mut() {
            // TODO: Return error instead
            let last_context = array
                .pop()
                .expect("Critical error: array should not be empty");
            return Ok(AstNode::Expression(last_context));
        }

        let block_id = self
            .current_block_id
            .expect("Critical error: current block id should always be set");
        let stack = self
            .block_ast_node_stack
            .get_mut(&block_id)
            .expect("Critical error: stack should always be set for each basic block");
        stack
            .pop()
            .ok_or(FunctionDecompilerError::CannotPopNode(block_id))
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

    /// Pops an identifier from the current basic block's stack.
    pub fn pop_identifier(&mut self) -> Result<IdentifierNode, FunctionDecompilerError> {
        let node = self.pop_expression()?;
        match node {
            ExprNode::Identifier(ident) => Ok(ident),
            _ => Err(FunctionDecompilerError::InvalidNodeType(
                self.current_block_id.unwrap(),
                "Identifier".to_string(),
                format!("{:?}", node),
            )),
        }
    }

    /// Pushes an AST node to the current basic block's stack.
    pub fn push_one_node(&mut self, node: AstNode) -> Result<(), FunctionDecompilerError> {
        // If we are building an array, we need to add the node to the array instead of the stack.
        if let Some(ExecutionState::BuildingArray(array)) = self.execution_state_stack.last_mut() {
            // first, ensure that the node is an expression
            if let AstNode::Expression(expr) = node {
                array.push(expr);
            } else {
                return Err(FunctionDecompilerError::InvalidNodeType(
                    self.current_block_id.unwrap(),
                    "Expression".to_string(),
                    format!("{:?}", node),
                ));
            }
            return Ok(());
        }

        let stack = self
            .block_ast_node_stack
            .get_mut(&self.current_block_id.unwrap())
            .expect("Critical error: stack should always be set for each basic block");
        stack.push(node);
        Ok(())
    }
}

// == Other Implementations ==
impl Default for FunctionDecompilerContext {
    fn default() -> Self {
        Self::new()
    }
}
