#![deny(missing_docs)]

use crate::basic_block::{BasicBlockId, BasicBlockType};
use crate::cfg_dot::{CfgDot, CfgDotConfig, DotRenderableGraph, NodeResolver};
use crate::decompiler::region::{Region, RegionId, RegionType};
use crate::function::{Function, FunctionError};
use crate::opcode::Opcode;
use crate::utils::GBF_BLUE;
use petgraph::graph::{DiGraph, NodeIndex};
use std::collections::HashMap;
use thiserror::Error;

use super::ast::visitors::emit_context::EmitContext;
use super::ast::visitors::emitter::Gs2Emitter;
use super::ast::{AstKind, AstVisitable};
use super::execution_frame::ExecutionFrame;
use super::function_decompiler_context::FunctionDecompilerContext;

/// An error when decompiling a function
#[derive(Debug, Error)]
pub enum FunctionDecompilerError {
    /// Cannot pop a node from the stack
    #[error("Cannot pop a node from the empty stack at BasicBlockId: {0}")]
    CannotPopNode(BasicBlockId),

    /// Encountered FunctionError
    #[error("Encountered FunctionError while decompiling: {0}")]
    FunctionError(#[from] FunctionError),

    /// Encountered an error while processing the operand
    #[error("Encountered an error while processing the operand: {0}")]
    OperandError(#[from] crate::operand::OperandError),

    /// The current instruction must have an operand
    #[error("The instruction associated with opcode {0:?} must have an operand")]
    InstructionMustHaveOperand(Opcode),

    /// Invalid node type on stack
    #[error("Invalid AstNode type on stack for BasicBlockId {0}. Expected {1}, found {2}")]
    InvalidNodeType(BasicBlockId, String, String),

    /// Encountered AstNodeError
    #[error("Encountered AstNodeError while decompiling: {0}")]
    AstNodeError(#[from] super::ast::AstNodeError),

    /// Unimplemented Opcode
    #[error("Unimplemented Opcode {0:?} in BasicBlockId {1}")]
    UnimplementedOpcode(Opcode, BasicBlockId),

    /// Execution state stack is empty
    #[error("Execution stack is empty")]
    ExecutionStackEmpty,

    /// Unexpected execution state
    #[error("Unexpected execution state. Expected {0}, but found {1}")]
    UnexpectedExecutionState(ExecutionFrame, ExecutionFrame),
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
    /// Regions vector
    regions: Vec<Region>,
    /// A conversion from block ids to region ids
    block_to_region: HashMap<BasicBlockId, RegionId>,
    /// The region graph of the function
    region_graph: DiGraph<(), ()>,
    /// Used to convert `NodeIndex` to `RegionId`.
    graph_node_to_region: HashMap<NodeIndex, RegionId>,
    /// Used to convert `RegionId` to `NodeIndex`.
    region_to_graph_node: HashMap<RegionId, NodeIndex>,
    /// The current context for the decompiler
    context: FunctionDecompilerContext,
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
    pub fn new(function: Function) -> Result<Self, FunctionDecompilerError> {
        let func_decompiler = FunctionDecompiler {
            function,
            regions: Vec::new(),
            block_to_region: HashMap::new(),
            region_graph: DiGraph::new(),
            graph_node_to_region: HashMap::new(),
            region_to_graph_node: HashMap::new(),
            context: FunctionDecompilerContext::new(),
        };

        Ok(func_decompiler)
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
        let entry_stack = self
            .context
            .get_stack(&entry_block_id)
            .expect("Critical error: stack should always be set for each basic block");

        let mut output = String::new();
        for node in entry_stack {
            // Each node should be StandaloneNode.
            if let ExecutionFrame::StandaloneNode(node) = node {
                let mut emitter = Gs2Emitter::new(emit_context);
                node.accept(&mut emitter);
                output.push_str(emitter.output());
                output.push('\n');
            } else {
                return Err(FunctionDecompilerError::UnexpectedExecutionState(
                    ExecutionFrame::StandaloneNode(AstKind::Empty),
                    node.clone(),
                ));
            }
        }

        Ok(output)
    }

    fn generate_regions(&mut self) {
        for block in self.function.iter() {
            // If the block is the end of the module, it is a tail region
            let region_type = if block.id.block_type == BasicBlockType::ModuleEnd {
                RegionType::Tail
            } else {
                RegionType::Linear
            };

            let new_region_id: RegionId = RegionId::new(self.regions.len(), region_type);
            self.block_to_region.insert(block.id, new_region_id.clone());

            // Add to the graph
            let node_id = self.region_graph.add_node(());
            self.graph_node_to_region
                .insert(node_id, new_region_id.clone());
            self.region_to_graph_node
                .insert(new_region_id.clone(), node_id);

            // Add to the array of regions
            self.regions.push(Region::new(new_region_id));
        }
    }

    fn process_regions(&mut self) -> Result<(), FunctionDecompilerError> {
        // Generate all the regions before doing anything else
        self.generate_regions();

        // Iterate through all the blocks in reverse post order
        let reverse_post_order = self
            .function
            .get_reverse_post_order(self.function.get_entry_basic_block().id)
            .map_err(FunctionDecompilerError::FunctionError)?;

        for block_id in reverse_post_order {
            self.context.start_block_processing(block_id)?;

            // Get the region id for the block
            let region_id = self.block_to_region.get(&block_id).unwrap().clone();

            // Connect the block's predecessors in the graph
            self.connect_predecessor_regions(block_id, region_id)?;

            // Process instructions in the block
            let instructions: Vec<_> = {
                let block = self.function.get_basic_block_by_id(block_id)?;
                block.iter().cloned().collect()
            };
            for instr in instructions {
                self.context.process_instruction(&instr)?;
            }
        }

        Ok(())
    }

    fn connect_predecessor_regions(
        &mut self,
        block_id: BasicBlockId,
        region_id: RegionId,
    ) -> Result<(), FunctionDecompilerError> {
        let predecessors = self
            .function
            .get_predecessors(block_id)
            .map_err(FunctionDecompilerError::FunctionError)?;
        let predecessor_regions: Vec<RegionId> = predecessors
            .iter()
            .map(|pred_id| self.block_to_region.get(pred_id).unwrap().clone())
            .collect();

        for pred_region_id in predecessor_regions {
            let pred_node_id = self.region_to_graph_node.get(&pred_region_id).unwrap();
            let current_node_id = self.region_to_graph_node.get(&region_id).unwrap();
            self.region_graph
                .add_edge(*pred_node_id, *current_node_id, ());
        }

        Ok(())
    }
}

// == Other Implementations ==
impl DotRenderableGraph for FunctionDecompiler {
    /// Convert the Graph to `dot` format.
    ///
    /// # Returns
    /// - A `String` containing the `dot` representation of the graph.
    fn render_dot(&self, config: CfgDotConfig) -> String {
        let dot = CfgDot { config };
        dot.render(&self.region_graph, self)
    }
}

impl NodeResolver for FunctionDecompiler {
    type NodeData = Region;

    fn resolve(&self, node_index: NodeIndex) -> Option<&Self::NodeData> {
        self.graph_node_to_region
            .get(&node_index)
            .and_then(|region_id| self.regions.get(region_id.index))
    }

    fn resolve_edge_color(&self, _: NodeIndex, _: NodeIndex) -> String {
        // TODO: Change based on CFG patterns
        GBF_BLUE.to_string()
    }
}
