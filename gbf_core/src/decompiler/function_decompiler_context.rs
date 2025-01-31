#![deny(missing_docs)]

use crate::basic_block::BasicBlockId;
use crate::decompiler::ast::new_id;
use crate::instruction::Instruction;
use crate::opcode::Opcode;
use std::backtrace::Backtrace;
use std::collections::HashMap;

use super::ast::assignable::AssignableKind;
use super::ast::expr::ExprKind;
use super::ast::identifier::IdentifierNode;
use super::ast::literal::LiteralNode;
use super::ast::ptr::P;
use super::ast::ssa::SsaContext;
use super::ast::AstKind;
use super::execution_frame::ExecutionFrame;
use super::function_decompiler::{FunctionDecompilerError, FunctionDecompilerErrorContext};
use super::handlers::{global_opcode_handlers, OpcodeHandler};
use super::{ProcessedInstruction, ProcessedInstructionBuilder};

/// Manages the state of the decompiler, including per-block AST stacks and current processing context.
pub struct FunctionDecompilerContext {
    /// AST node stacks for each basic block.
    pub block_ast_node_stack: HashMap<BasicBlockId, Vec<ExecutionFrame>>,
    /// The current basic block being processed.
    pub current_block_id: BasicBlockId,
    /// The handlers for each opcode.
    pub opcode_handlers: HashMap<Opcode, Box<dyn OpcodeHandler>>,
    /// The SSA Context
    pub ssa_context: SsaContext,
    /// The current instruction being processed.
    pub current_instruction: Instruction,
    /// Register mapping for the current function
    pub register_mapping: HashMap<usize, AssignableKind>,
}

impl FunctionDecompilerContext {
    /// Creates a new, empty context. We initialize with the starting block ID and region ID.
    pub fn new(start_block_id: BasicBlockId) -> Self {
        Self {
            block_ast_node_stack: HashMap::new(),
            current_block_id: start_block_id,
            opcode_handlers: HashMap::new(),
            ssa_context: SsaContext::new(),
            current_instruction: Instruction::default(),
            register_mapping: HashMap::new(),
        }
    }

    /// Starts processing a new basic block.
    ///
    /// # Arguments
    /// - `block_id`: The ID of the basic block to start processing.
    /// - `region_id`: The ID of the region the basic block belongs to.
    ///
    /// # Errors
    /// - Returns `FunctionDecompilerError` if there is an issue initializing the block stack.
    pub fn start_block_processing(
        &mut self,
        block_id: BasicBlockId,
    ) -> Result<(), FunctionDecompilerError> {
        self.current_block_id = block_id;
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
    ) -> Result<ProcessedInstruction, FunctionDecompilerError> {
        self.current_instruction = instr.clone();

        let current_block_id = self.current_block_id;

        // TODO: Better handle PushArray
        if instr.opcode == Opcode::PushArray {
            let stack = self
                .block_ast_node_stack
                .get_mut(&current_block_id)
                .expect("Critical error: stack should always be set for each basic block");

            stack.push(ExecutionFrame::BuildingArray(Vec::new()));
            return Ok(ProcessedInstructionBuilder::new().build());
        }

        let handlers = global_opcode_handlers();

        let handler =
            handlers
                .get(&instr.opcode)
                .ok_or(FunctionDecompilerError::UnimplementedOpcode {
                    opcode: instr.opcode,
                    context: self.get_error_context(),
                    backtrace: Backtrace::capture(),
                })?;

        // Handle the instruction
        // TODO: Since we have the instruction in the context, we may delete it from the
        // TODO: arguments to avoid passing it around everywhere
        let op = handler.handle_instruction(self, instr)?;

        // Push the SSA ID onto the stack if it exists
        if let Some(ssa_id) = &op.ssa_id {
            self.push_one_node(ssa_id.clone().into())?;
        }

        Ok(op)
    }

    /// If an error happens, this helper function will return the context of the error.
    pub fn get_error_context(&self) -> Box<FunctionDecompilerErrorContext> {
        Box::new(FunctionDecompilerErrorContext {
            current_block_id: self.current_block_id,
            current_instruction: self.current_instruction.clone(),
            current_ast_node_stack: self
                .block_ast_node_stack
                .get(&self.current_block_id)
                .unwrap_or(&Vec::new())
                .to_vec(),
        })
    }

    /// Retrieves the AST stack for a basic block.
    pub fn get_stack(&self, block_id: &BasicBlockId) -> Option<&Vec<ExecutionFrame>> {
        self.block_ast_node_stack.get(block_id)
    }

    /// Pops an AST node from the current basic block's stack.
    pub fn pop_one_node(&mut self) -> Result<AstKind, FunctionDecompilerError> {
        let block_id = self.current_block_id;
        let error_context = self.get_error_context();
        let stack = self
            .block_ast_node_stack
            .get_mut(&block_id)
            .expect("Critical error: stack should always be set for each basic block");

        // Ensure there's a frame to pop from
        let mut last_frame = stack
            .pop()
            .ok_or(FunctionDecompilerError::ExecutionStackEmpty {
                backtrace: Backtrace::capture(),
                context: error_context.clone(),
            })?;

        let result = match &mut last_frame {
            ExecutionFrame::BuildingArray(array) => {
                // Pop the node from the array
                if let Some(node) = array.pop() {
                    Ok(AstKind::Expression(node))
                } else {
                    Err(FunctionDecompilerError::ExecutionStackEmpty {
                        backtrace: Backtrace::capture(),
                        context: error_context.clone(),
                    })
                }
            }
            ExecutionFrame::StandaloneNode(node) => Ok(node.clone()),
            ExecutionFrame::None => Err(FunctionDecompilerError::ExecutionStackEmpty {
                backtrace: Backtrace::capture(),
                context: error_context.clone(),
            }),
        };

        // Push the last frame back onto the stack, even if it's empty
        if let ExecutionFrame::BuildingArray(_) = last_frame {
            stack.push(last_frame);
        }

        result
    }

    /// Pops an expression from the current basic block's stack.
    pub fn pop_expression(&mut self) -> Result<ExprKind, FunctionDecompilerError> {
        let node = self.pop_one_node()?;
        match node {
            AstKind::Expression(expr) => Ok(expr.clone()),
            _ => Err(FunctionDecompilerError::UnexpectedNodeType {
                expected: "Expression".to_string(),
                context: self.get_error_context(),
                backtrace: Backtrace::capture(),
            }),
        }
    }

    /// Pops an assignable expression from the current basic block's stack.
    pub fn pop_assignable(&mut self) -> Result<AssignableKind, FunctionDecompilerError> {
        let node = self.pop_expression()?;

        match node {
            ExprKind::Assignable(assignable) => Ok(assignable.clone()),

            ExprKind::Literal(lit) => {
                if let LiteralNode::String(s) = lit.as_ref() {
                    log::warn!(
                    "String literal used as assignable: {}. Technically this is allowed in GS2. God help us all.",
                    s
                );
                    Ok(new_id(format!("\"{}\"", &s.clone()).as_str()).into())
                } else {
                    Err(FunctionDecompilerError::UnexpectedNodeType {
                        expected: "Assignable".to_string(),
                        context: self.get_error_context(),
                        backtrace: Backtrace::capture(),
                    })
                }
            }

            _ => Err(FunctionDecompilerError::UnexpectedNodeType {
                expected: "Assignable".to_string(),
                context: self.get_error_context(),
                backtrace: Backtrace::capture(),
            }),
        }
    }

    /// Pops an identifier from the current basic block's stack.
    pub fn pop_identifier(&mut self) -> Result<P<IdentifierNode>, FunctionDecompilerError> {
        let node = self.pop_assignable()?;

        match node {
            AssignableKind::Identifier(ident) => Ok(ident.clone()),
            _ => Err(FunctionDecompilerError::UnexpectedNodeType {
                expected: "Identifier".to_string(),
                context: self.get_error_context(),
                backtrace: Backtrace::capture(),
            }),
        }
    }

    /// Pushes an AST node to the current basic block's stack.
    pub fn push_one_node(&mut self, node: AstKind) -> Result<(), FunctionDecompilerError> {
        let block_id = self.current_block_id;

        let stack = self
            .block_ast_node_stack
            .get_mut(&block_id)
            .expect("Critical error: stack should always be set for each basic block");

        // Check if we're in a frame that requires special handling
        if let Some(last_frame) = stack.last_mut() {
            match last_frame {
                ExecutionFrame::BuildingArray(array) => {
                    // Ensure the node is an expression before adding to the array
                    if let AstKind::Expression(expr) = node {
                        array.push(expr.clone());
                        return Ok(());
                    } else {
                        return Err(FunctionDecompilerError::UnexpectedNodeType {
                            expected: "Expression".to_string(),
                            context: self.get_error_context(),
                            backtrace: Backtrace::capture(),
                        });
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
