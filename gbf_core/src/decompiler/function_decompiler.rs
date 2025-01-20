#![deny(missing_docs)]

use crate::basic_block::BasicBlockId;
use crate::function::{Function, FunctionError};
use crate::instruction::Instruction;
use crate::opcode::Opcode;
use crate::operand::OperandError;
use serde::Serialize;
use std::backtrace::Backtrace;
use std::collections::HashMap;
use thiserror::Error;

use super::ast::ast_vec::AstVec;
use super::ast::expr::ExprKind;
use super::ast::function::FunctionNode;
use super::ast::visitors::emit_context::EmitContext;
use super::ast::visitors::emitter::Gs2Emitter;
use super::ast::{AstKind, AstVisitable};
use super::execution_frame::ExecutionFrame;
use super::function_decompiler_context::FunctionDecompilerContext;
use super::structure_analysis::region::{RegionId, RegionType};
use super::structure_analysis::StructureAnalysis;

/// An error when decompiling a function
#[derive(Debug, Error, Serialize)]
pub enum FunctionDecompilerError {
    /// Encountered FunctionError
    #[error("Encountered FunctionError while decompiling: {source}")]
    FunctionError {
        /// The source of the error
        source: FunctionError,
        /// The context of the error
        context: Box<FunctionDecompilerErrorContext>,
        /// The backtrace of the error
        #[serde(skip)]
        backtrace: Backtrace,
    },

    /// Encountered an error while processing the operand
    #[error("Encountered an error while processing the operand: {source}")]
    OperandError {
        /// The source of the error
        source: OperandError,
        /// The context of the error
        context: Box<FunctionDecompilerErrorContext>,
        /// The backtrace of the error
        #[serde(skip)]
        backtrace: Backtrace,
    },

    /// Encountered AstNodeError
    #[error("Encountered AstNodeError while decompiling: {source}")]
    AstNodeError {
        /// The source of the error
        source: super::ast::AstNodeError,
        /// The context of the error
        context: Box<FunctionDecompilerErrorContext>,
        /// The backtrace of the error
        #[serde(skip)]
        backtrace: Backtrace,
    },

    /// The current instruction must have an operand
    #[error("The instruction associated with opcode {opcode} must have an operand.")]
    InstructionMustHaveOperand {
        /// The opcode associated with the instruction
        opcode: Opcode,
        /// The context of the error
        context: Box<FunctionDecompilerErrorContext>,
        /// The backtrace of the error
        #[serde(skip)]
        backtrace: Backtrace,
    },

    /// Invalid node type on stack
    #[error("Unexpected AstNode sub-type on stack. Expected {expected}.")]
    UnexpectedNodeType {
        /// The expected node type
        expected: String,
        /// The context of the error
        context: Box<FunctionDecompilerErrorContext>,
        /// The backtrace of the error
        #[serde(skip)]
        backtrace: Backtrace,
    },

    /// Unimplemented Opcode
    #[error("Unimplemented Opcode.")]
    UnimplementedOpcode {
        /// The context of the error
        context: Box<FunctionDecompilerErrorContext>,
        /// The backtrace of the error
        #[serde(skip)]
        backtrace: Backtrace,
    },

    /// Execution state stack is empty
    #[error("The AST Node stack is empty.")]
    ExecutionStackEmpty {
        /// The context of the error
        context: Box<FunctionDecompilerErrorContext>,
        /// The backtrace of the error
        #[serde(skip)]
        backtrace: Backtrace,
    },

    /// Unexpected execution state
    #[error("Unexpected execution state.")]
    UnexpectedExecutionState {
        /// The context of the error
        context: Box<FunctionDecompilerErrorContext>,
        /// The backtrace of the error
        #[serde(skip)]
        backtrace: Backtrace,
    },

    /// All other errors
    #[error("An error occurred while decompiling the function: {message}")]
    Other {
        /// Message associated with the error
        message: String,
        /// The context of the error
        context: Box<FunctionDecompilerErrorContext>,
        /// The backtrace of the error
        #[serde(skip)]
        backtrace: Backtrace,
    },
}

/// A trait to provide details for a function decompiler error
pub trait FunctionDecompilerErrorDetails {
    /// Get the context for the error
    fn context(&self) -> &FunctionDecompilerErrorContext;
    /// Get the backtrace for the error
    fn backtrace(&self) -> &Backtrace;
}

/// The context for a function decompiler error
#[derive(Debug, Serialize, Clone)]
pub struct FunctionDecompilerErrorContext {
    /// The current block ID when the error occurred
    pub current_block_id: BasicBlockId,
    /// The current instruction when the error occurred
    pub current_instruction: Instruction,
    /// The current AST node stack when the error occurred
    pub current_ast_node_stack: Vec<ExecutionFrame>,
}

// TODO: Map instructions to a reference value (for usage with loop variables, etc.)
// TODO: We should call loop variables instruction references (InstrRef)
// TODO: We should have an AST pass that identifies variables with identifiers that are
// TODO: the same, and wrap them in an InstrRef (for MemberAccess & Identifier) since
// TODO: this will help further analysis

/// A struct to hold the state of a function decompiler
pub struct FunctionDecompiler {
    /// Create a copy of the function to analyze
    function: Function,
    /// A conversion from block ids to region ids
    block_to_region: HashMap<BasicBlockId, RegionId>,
    /// The current context for the decompiler
    context: Option<FunctionDecompilerContext>,
    /// The parameters for the function
    function_parameters: AstVec<ExprKind>,
    /// The structure analysis
    struct_analysis: StructureAnalysis,
}

impl FunctionDecompiler {
    /// A new method for the FunctionDecompiler struct.
    ///
    /// # Arguments
    /// - `function`: The function to analyze and decompile.
    ///
    /// # Returns
    /// - A newly constructed `FunctionDecompiler` instance.
    ///
    /// # Errors
    /// - `FunctionDecompilerError` if there is an error while decompiling the function.
    pub fn new(function: Function) -> Self {
        FunctionDecompiler {
            function,
            block_to_region: HashMap::new(),
            context: None,
            function_parameters: Vec::<ExprKind>::new().into(),
            struct_analysis: StructureAnalysis::new(),
        }
    }
}

// == Private Functions ==
impl FunctionDecompiler {
    /// Decompile the function and emit the AST as a string.
    ///
    /// # Arguments
    /// - `context`: The context for AST emission.
    ///
    /// # Returns
    /// - The emitted AST as a string.
    ///
    /// # Errors
    /// - Returns `FunctionDecompilerError` for any issues encountered during decompilation.
    pub fn decompile(
        &mut self,
        emit_context: EmitContext,
    ) -> Result<String, FunctionDecompilerError> {
        self.process_regions()?;

        let entry_block_id = self.function.get_entry_basic_block().id;
        let entry_region_id = self.block_to_region.get(&entry_block_id).unwrap();
        let entry_region = self
            .struct_analysis
            .get_region(*entry_region_id)
            .expect("[Bug] The entry region should exist.");

        let entry_region_nodes = entry_region.iter_nodes().cloned().collect::<AstVec<_>>();

        let func = AstKind::Function(FunctionNode::new(
            self.function.id.name.clone(),
            self.function_parameters.clone(),
            entry_region_nodes,
        ));

        let mut output = String::new();
        let mut emitter = Gs2Emitter::new(emit_context);
        func.accept(&mut emitter);
        output.push_str(emitter.output());
        output.push('\n');

        Ok(output)
    }

    fn generate_regions(&mut self) -> Result<(), FunctionDecompilerError> {
        for block in self.function.iter() {
            // If the block is the end of the module, it is a tail region
            let successors = self.function.get_successors(block.id).map_err(|e| {
                FunctionDecompilerError::FunctionError {
                    source: e,
                    backtrace: Backtrace::capture(),
                    context: self.context.as_ref().unwrap().get_error_context(),
                }
            })?;
            let region_type = if successors.is_empty() {
                RegionType::Tail
            } else {
                RegionType::Linear
            };

            let new_region_id = self.struct_analysis.add_region(region_type);
            self.block_to_region.insert(block.id, new_region_id);
        }
        Ok(())
    }

    fn process_regions(&mut self) -> Result<(), FunctionDecompilerError> {
        // Generate all the regions before doing anything else
        self.generate_regions()?;

        let mut ctx = FunctionDecompilerContext::new(self.function.get_entry_basic_block_id());

        // Iterate through all the blocks in reverse post order
        let reverse_post_order = self
            .function
            .get_reverse_post_order(self.function.get_entry_basic_block().id)
            .map_err(|e| FunctionDecompilerError::FunctionError {
                source: e,
                backtrace: Backtrace::capture(),
                context: ctx.get_error_context(),
            })?;

        for block_id in reverse_post_order {
            // Get the region id for the block
            let region_id = *self
                .block_to_region
                .get(&block_id)
                .expect("[Bug] We just made the regions, so not sure why it doesn't exist.");

            ctx.start_block_processing(block_id)?;

            // Connect the block's predecessors in the graph
            self.connect_predecessor_regions(block_id, region_id)?;

            // Process instructions in the block
            let instructions: Vec<_> = {
                let block = self.function.get_basic_block_by_id(block_id).map_err(|e| {
                    FunctionDecompilerError::FunctionError {
                        source: e,
                        backtrace: Backtrace::capture(),
                        context: ctx.get_error_context(),
                    }
                })?;
                block.iter().cloned().collect()
            };

            for instr in instructions {
                let processed = ctx.process_instruction(&instr)?;
                if let Some(node) = processed.node_to_push {
                    let current_region_id = self
                        .block_to_region
                        .get(&block_id)
                        .expect("[Bug] The region should exist.");
                    self.struct_analysis
                        .push_to_region(*current_region_id, node);
                }
                if let Some(params) = processed.function_parameters {
                    self.function_parameters = params;
                }
            }
        }

        self.context = Some(ctx);

        Ok(())
    }

    fn connect_predecessor_regions(
        &mut self,
        block_id: BasicBlockId,
        region_id: RegionId,
    ) -> Result<(), FunctionDecompilerError> {
        let predecessors = self.function.get_predecessors(block_id).map_err(|e| {
            FunctionDecompilerError::FunctionError {
                source: e,
                backtrace: Backtrace::capture(),
                context: self.context.as_ref().unwrap().get_error_context(),
            }
        })?;
        let predecessor_regions: Vec<RegionId> = predecessors
            .iter()
            .map(|pred_id| *self.block_to_region.get(pred_id).unwrap())
            .collect();

        for pred_region_id in predecessor_regions {
            self.struct_analysis
                .connect_regions(pred_region_id, region_id);
        }

        Ok(())
    }
}

// == Other Implementations ==

impl FunctionDecompilerErrorDetails for FunctionDecompilerError {
    fn context(&self) -> &FunctionDecompilerErrorContext {
        match self {
            FunctionDecompilerError::FunctionError { context, .. } => context,
            FunctionDecompilerError::OperandError { context, .. } => context,
            FunctionDecompilerError::AstNodeError { context, .. } => context,
            FunctionDecompilerError::InstructionMustHaveOperand { context, .. } => context,
            FunctionDecompilerError::UnexpectedNodeType { context, .. } => context,
            FunctionDecompilerError::UnimplementedOpcode { context, .. } => context,
            FunctionDecompilerError::ExecutionStackEmpty { context, .. } => context,
            FunctionDecompilerError::UnexpectedExecutionState { context, .. } => context,
            FunctionDecompilerError::Other { context, .. } => context,
        }
    }

    fn backtrace(&self) -> &Backtrace {
        match self {
            FunctionDecompilerError::FunctionError { backtrace, .. } => backtrace,
            FunctionDecompilerError::OperandError { backtrace, .. } => backtrace,
            FunctionDecompilerError::AstNodeError { backtrace, .. } => backtrace,
            FunctionDecompilerError::InstructionMustHaveOperand { backtrace, .. } => backtrace,
            FunctionDecompilerError::UnexpectedNodeType { backtrace, .. } => backtrace,
            FunctionDecompilerError::UnimplementedOpcode { backtrace, .. } => backtrace,
            FunctionDecompilerError::ExecutionStackEmpty { backtrace, .. } => backtrace,
            FunctionDecompilerError::UnexpectedExecutionState { backtrace, .. } => backtrace,
            FunctionDecompilerError::Other { backtrace, .. } => backtrace,
        }
    }
}
