#![deny(missing_docs)]

use crate::basic_block::BasicBlockId;
use crate::function::{Function, FunctionError};
use crate::instruction::Instruction;
use crate::opcode::Opcode;
use crate::operand::OperandError;
use crate::utils::STRUCTURE_ANALYSIS_MAX_ITERATIONS;
use serde::Serialize;
use std::backtrace::Backtrace;
use std::collections::HashMap;
use thiserror::Error;

use super::ast::expr::ExprKind;
use super::ast::function::FunctionNode;
use super::ast::visitors::emit_context::EmitContext;
use super::ast::visitors::emitter::Gs2Emitter;
use super::ast::{new_phi, AstKind, AstVisitable};
use super::execution_frame::ExecutionFrame;
use super::function_decompiler_context::FunctionDecompilerContext;
use super::structure_analysis::region::{RegionId, RegionType};
use super::structure_analysis::{ControlFlowEdgeType, StructureAnalysis, StructureAnalysisError};

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

    /// Register not found
    #[error("Register not found: {register_id}")]
    RegisterNotFound {
        /// The register ID that was not found
        register_id: usize,
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
    #[error("Unimplemented Opcode: {opcode}")]
    UnimplementedOpcode {
        /// The opcode that is unimplemented
        opcode: Opcode,
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

    /// Structure analysis error
    #[error("A structure analysis error occurred while decompiling the function: {source}")]
    StructureAnalysisError {
        /// The source of the error
        source: Box<StructureAnalysisError>,
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
    /// Get the type for the error
    fn error_type(&self) -> String;
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

/// The builder for a function decompiler
pub struct FunctionDecompilerBuilder {
    function: Function,
    emit_context: EmitContext,
    structure_debug_mode: bool,
    structure_analysis_max_iterations: usize,
}

impl FunctionDecompilerBuilder {
    /// Create a new function decompiler builder
    pub fn new(function: Function) -> Self {
        FunctionDecompilerBuilder {
            function,
            emit_context: EmitContext::default(),
            structure_debug_mode: false,
            structure_analysis_max_iterations: STRUCTURE_ANALYSIS_MAX_ITERATIONS,
        }
    }

    /// Set the emit context for the function decompiler
    pub fn emit_context(mut self, emit_context: EmitContext) -> Self {
        self.emit_context = emit_context;
        self
    }

    /// Set the structure debug mode for the function decompiler. These keeps track
    /// of the structure of the function as it is being analyzed with StructureAnalysis.
    pub fn structure_debug_mode(mut self, structure_debug_mode: bool) -> Self {
        self.structure_debug_mode = structure_debug_mode;
        self
    }

    /// Sets the maximum number of iterations for the structure analysis
    pub fn structure_analysis_max_iterations(mut self, max_iterations: usize) -> Self {
        self.structure_analysis_max_iterations = max_iterations;
        self
    }

    /// Build the function decompiler
    pub fn build(self) -> FunctionDecompiler {
        FunctionDecompiler::new(
            self.function,
            self.structure_debug_mode,
            self.structure_analysis_max_iterations,
        )
    }
}

/// A struct to hold the state of a function decompiler
pub struct FunctionDecompiler {
    /// Create a copy of the function to analyze
    function: Function,
    /// A conversion from block ids to region ids
    block_to_region: HashMap<BasicBlockId, RegionId>,
    /// The current context for the decompiler
    context: Option<FunctionDecompilerContext>,
    /// The parameters for the function
    function_parameters: Vec<ExprKind>,
    /// The structure analysis
    struct_analysis: StructureAnalysis,
    /// Whether the analysis has been run
    did_run_analysis: bool,
}

impl FunctionDecompiler {
    /// A new method for the FunctionDecompiler struct.
    ///
    /// # Arguments
    /// - `function`: The function to analyze and decompile.
    /// - `structure_debug_mode`: Whether to enable debug mode for the structure analysis.
    /// - `structure_max_iterations`: The maximum number of iterations for the structure analysis.
    ///
    /// # Returns
    /// - A newly constructed `FunctionDecompiler` instance.
    ///
    /// # Errors
    /// - `FunctionDecompilerError` if there is an error while decompiling the function.
    fn new(
        function: Function,
        structure_debug_mode: bool,
        structure_max_iterations: usize,
    ) -> Self {
        FunctionDecompiler {
            function,
            block_to_region: HashMap::new(),
            context: None,
            function_parameters: Vec::<ExprKind>::new(),
            struct_analysis: StructureAnalysis::new(structure_debug_mode, structure_max_iterations),
            did_run_analysis: false,
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

        self.did_run_analysis = true;
        self.struct_analysis.execute().map_err(|e| {
            FunctionDecompilerError::StructureAnalysisError {
                source: Box::new(e),
                context: self.context.as_ref().unwrap().get_error_context(),
                backtrace: Backtrace::capture(),
            }
        })?;
        let entry_region = {
            let region = self
                .struct_analysis
                .get_region(*entry_region_id)
                .expect("[Bug] The entry region should exist.");
            region.clone()
        };
        let entry_region_nodes = entry_region.iter_nodes().cloned().collect::<Vec<_>>();

        let func = AstKind::Function(
            FunctionNode::new(
                self.function.id.name.clone(),
                self.function_parameters.clone(),
                entry_region_nodes,
            )
            .into(),
        );

        let mut emitter = Gs2Emitter::new(emit_context);
        let output: String = func.accept(&mut emitter).node;

        Ok(output)
    }

    /// Get the structure analysis snapshots
    pub fn get_structure_analysis_snapshots(&self) -> Result<Vec<String>, FunctionDecompilerError> {
        self.struct_analysis
            .get_snapshots()
            .map_err(|e| FunctionDecompilerError::StructureAnalysisError {
                source: Box::new(e),
                context: self.context.as_ref().unwrap().get_error_context(),
                backtrace: Backtrace::capture(),
            })
            .cloned()
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

        for block_id in &reverse_post_order {
            // Get the region id for the block
            let region_id = *self
                .block_to_region
                .get(block_id)
                .expect("[Bug] We just made the regions, so not sure why it doesn't exist.");

            ctx.start_block_processing(*block_id)?;

            // Connect the block's predecessors in the graph
            self.connect_predecessor_regions(*block_id, region_id)?;

            // Process instructions in the block
            let instructions: Vec<_> = {
                let block = self
                    .function
                    .get_basic_block_by_id(*block_id)
                    .map_err(|e| FunctionDecompilerError::FunctionError {
                        source: e,
                        backtrace: Backtrace::capture(),
                        context: ctx.get_error_context(),
                    })?;
                block.iter().cloned().collect()
            };

            // Create a vector of RegionIds for the predecessors
            let mut predecessor_regions: Vec<Vec<(RegionId, ControlFlowEdgeType, AstKind)>> =
                Vec::new();

            // For each predecessor block, see what AST nodes are left on the stack
            // and introduce Phi nodes if necessary
            for pred in self.get_predecessors(*block_id)? {
                let exec = ctx.block_ast_node_stack.get(&pred.0);

                // There's a chance that we haven't processed the predecessor block yet, especially
                // if we're in a self-referential loop. In that case, we can't introduce phi nodes
                // because we don't know what the stack will look like.
                // TODO: Double-check this logic
                let exec = if let Some(exec) = exec {
                    exec
                } else {
                    continue;
                };

                // Create empty list of region ids
                for (i, frame) in exec.iter().rev().enumerate() {
                    match frame {
                        ExecutionFrame::StandaloneNode(n) => {
                            // Ensure we have a slot in predecessor_regions for this index.
                            if predecessor_regions.len() <= i {
                                // If not, extend the vector so that index i is available.
                                predecessor_regions.resize(i + 1, Vec::new());
                            }
                            // Record the region info (e.g., pred.1 and pred.2) for this phi candidate.
                            predecessor_regions[i].push((pred.1, pred.2, n.clone()));
                        }
                        // TODO: Bug. BuildingArray is another possible frame type.
                        _ => {
                            return Err(FunctionDecompilerError::Other {
                                message: "Expected StandaloneNode".to_string(),
                                context: ctx.get_error_context(),
                                backtrace: Backtrace::capture(),
                            });
                        }
                    }
                }

                // Validate that this predecessor's execution stack length matches what we expect.
                if !predecessor_regions.is_empty() && exec.len() != predecessor_regions.len() {
                    // TODO: This will happen with short-circuit operators. We should handle this case
                    // more gracefully.
                    log::warn!(
                        "Inconsistent number of phi candidates in predecessor block {:?}: expected {}, got {}",
                        pred.0,
                        predecessor_regions.len(),
                        exec.len()
                    );
                }
            }

            // Inject phi nodes into the AST
            for (index, raw_phi) in predecessor_regions.iter().enumerate() {
                if raw_phi.len() == 1 || raw_phi.iter().all(|(_, _, node)| node == &raw_phi[0].2) {
                    // If there's only one predecessor or all nodes are equal, simply push the node onto the stack.
                    let (_, _, node) = &raw_phi[0];
                    ctx.push_one_node(node.clone())?;
                    continue;
                }
                let mut phi = new_phi(index);
                phi.add_regions(raw_phi.iter().map(|x| (x.0, x.1)).collect());
                ctx.push_one_node(phi.into())?;
            }

            for instr in instructions {
                let processed = ctx.process_instruction(&instr)?;
                if let Some(node) = processed.node_to_push {
                    let current_region_id = self
                        .block_to_region
                        .get(block_id)
                        .expect("[Bug] The region should exist.");
                    self.struct_analysis
                        .push_to_region(*current_region_id, node);
                }

                if let Some(params) = processed.function_parameters {
                    self.function_parameters = params;
                }

                if let Some(jmp) = &processed.jump_condition {
                    let current_region_id = self
                        .block_to_region
                        .get(block_id)
                        .expect("[Bug] The region should exist.");
                    let region = self
                        .struct_analysis
                        .get_region_mut(*current_region_id)
                        .expect("[Bug] The region should exist.");

                    region.set_jump_expr(Some(jmp.clone()));
                    region.set_region_type(RegionType::ControlFlow);
                    region.set_branch_opcode(instr.opcode);
                }
            }
        }

        for blk in reverse_post_order {
            let region_id = *self
                .block_to_region
                .get(&blk)
                .expect("[Bug] We just made the regions, so not sure why it doesn't exist.");

            // for any nodes left in the block push them to the region
            let exec = ctx.block_ast_node_stack.get(&blk);
            if let Some(exec) = exec {
                for frame in exec.iter().rev() {
                    match frame {
                        ExecutionFrame::StandaloneNode(n) => {
                            // Push the unresolved node into the region
                            self.struct_analysis
                                .push_unresolved_to_region(region_id, n.clone());
                        }
                        // TODO: Bug. BuildingArray is another possible frame type.
                        _ => {
                            return Err(FunctionDecompilerError::Other {
                                message: "Expected StandaloneNode".to_string(),
                                context: ctx.get_error_context(),
                                backtrace: Backtrace::capture(),
                            });
                        }
                    }
                }
            }
        }

        self.context = Some(ctx);

        Ok(())
    }

    /// Get predecessors of a block and return the results as a vector of tuples
    fn get_predecessors(
        &self,
        block_id: BasicBlockId,
    ) -> Result<Vec<(BasicBlockId, RegionId, ControlFlowEdgeType)>, FunctionDecompilerError> {
        // Step 1: Get the predecessors of the current block
        let predecessors = self.function.get_predecessors(block_id).map_err(|e| {
            FunctionDecompilerError::FunctionError {
                source: e,
                backtrace: Backtrace::capture(),
                context: self.context.as_ref().unwrap().get_error_context(),
            }
        })?;

        // Step 2: Map each predecessor to its region ID and determine the edge type
        let predecessor_regions: Vec<(BasicBlockId, RegionId, ControlFlowEdgeType)> = predecessors
            .iter()
            .map(|pred_id| {
                let pred_region_id = *self.block_to_region.get(pred_id).unwrap();

                // Get the predecessor block
                let pred_block = self
                    .function
                    .get_basic_block_by_id(*pred_id)
                    .expect("Predecessor block not found");

                // Get the last instruction of the predecessor block
                // TODO: This is a bug if the block is empty; maybe in this case we should
                // just get the address of the block?
                let pred_last_instruction = pred_block.last().expect("Empty block");

                // Get the target block address
                let target_block = self
                    .function
                    .get_basic_block_by_id(block_id)
                    .expect("Target block not found");
                let target_address = target_block.id.address;

                // Determine the edge type based on control flow
                let edge_type = if pred_last_instruction.address + 1 != target_address {
                    // The target address is NOT the next address, so it's a branch
                    ControlFlowEdgeType::Branch
                } else {
                    ControlFlowEdgeType::Fallthrough
                };

                (*pred_id, pred_region_id, edge_type)
            })
            .collect();
        Ok(predecessor_regions)
    }

    fn connect_predecessor_regions(
        &mut self,
        block_id: BasicBlockId,
        region_id: RegionId,
    ) -> Result<Vec<(BasicBlockId, RegionId, ControlFlowEdgeType)>, FunctionDecompilerError> {
        // Step 1: Get the predecessors of the current block
        let predecessor_regions = self.get_predecessors(block_id)?;

        // Step 2: Connect the predecessor regions to the target region in the graph
        for (_, pred_region_id, edge_type) in &predecessor_regions {
            self.struct_analysis
                .connect_regions(*pred_region_id, region_id, *edge_type)
                .map_err(|e| FunctionDecompilerError::StructureAnalysisError {
                    source: Box::new(e),
                    context: self.context.as_ref().unwrap().get_error_context(),
                    backtrace: Backtrace::capture(),
                })?;
        }

        // Step 4: Return the vector of predecessor regions and their edge types
        Ok(predecessor_regions)
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
            FunctionDecompilerError::StructureAnalysisError { context, .. } => context,
            FunctionDecompilerError::RegisterNotFound { context, .. } => context,
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
            FunctionDecompilerError::StructureAnalysisError { source, .. } => source.backtrace(),
            FunctionDecompilerError::RegisterNotFound { backtrace, .. } => backtrace,
        }
    }

    fn error_type(&self) -> String {
        match self {
            FunctionDecompilerError::FunctionError { .. } => "FunctionError".to_string(),
            FunctionDecompilerError::OperandError { .. } => "OperandError".to_string(),
            FunctionDecompilerError::AstNodeError { .. } => "AstNodeError".to_string(),
            FunctionDecompilerError::InstructionMustHaveOperand { .. } => {
                "InstructionMustHaveOperand".to_string()
            }
            FunctionDecompilerError::UnexpectedNodeType { .. } => "UnexpectedNodeType".to_string(),
            FunctionDecompilerError::UnimplementedOpcode { .. } => {
                "UnimplementedOpcode".to_string()
            }
            FunctionDecompilerError::ExecutionStackEmpty { .. } => {
                "ExecutionStackEmpty".to_string()
            }
            FunctionDecompilerError::UnexpectedExecutionState { .. } => {
                "UnexpectedExecutionState".to_string()
            }
            FunctionDecompilerError::Other { .. } => "Other".to_string(),
            FunctionDecompilerError::StructureAnalysisError { .. } => {
                "StructureAnalysisError".to_string()
            }
            FunctionDecompilerError::RegisterNotFound { .. } => "RegisterNotFound".to_string(),
        }
    }
}
