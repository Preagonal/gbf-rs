#![deny(missing_docs)]

use ast::{expr::ExprKind, identifier::IdentifierNode, ptr::P, AstKind};

/// This provides the AST for the decompiler.
pub mod ast;
/// The state of execution for the decompiler
pub mod execution_frame;
/// This assists in decompiling one function
pub mod function_decompiler;
/// This provides the context for the decompiler
pub mod function_decompiler_context;
/// This provides the handlers for the decompiler
pub mod handlers;
/// This is responsible for control flow analysis
pub mod structure_analysis;

#[derive(Debug, Clone, Default)]
/// Builder for constructing a `ProcessedInstruction`.
pub struct ProcessedInstructionBuilder {
    ssa_id: Option<P<IdentifierNode>>,
    node_to_push: Option<AstKind>,
    function_parameters: Option<Vec<ExprKind>>,
    jump_condition: Option<ExprKind>,
}

impl ProcessedInstructionBuilder {
    /// Creates a new builder instance.
    pub fn new() -> Self {
        Self {
            ssa_id: None,
            node_to_push: None,
            function_parameters: None,
            jump_condition: None,
        }
    }

    /// Sets the SSA ID for the processed instruction.
    ///
    /// # Arguments
    /// - `ssa_id`: The SSA ID to assign.
    ///
    /// # Returns
    /// A mutable reference to the builder for chaining.
    pub fn ssa_id(mut self, ssa_id: P<IdentifierNode>) -> Self {
        self.ssa_id = Some(ssa_id);
        self
    }

    /// Sets the node to push to a region for the processed instruction.
    ///
    /// # Arguments
    /// - `node_to_push`: The AST node to assign.
    ///
    /// # Returns
    /// A mutable reference to the builder for chaining.
    pub fn push_to_region(mut self, node_to_push: AstKind) -> Self {
        self.node_to_push = Some(node_to_push);
        self
    }

    /// Sets the function parameters for the processed instruction.
    ///
    /// # Arguments
    /// - `function_parameters`: The function parameters to assign.
    ///
    /// # Returns
    /// A mutable reference to the builder for chaining.
    pub fn function_parameters(mut self, function_parameters: Vec<ExprKind>) -> Self {
        self.function_parameters = Some(function_parameters);
        self
    }

    /// Sets the jump condition for the processed instruction.
    ///
    /// # Arguments
    /// - `jump_condition`: The jump condition to assign.
    ///
    /// # Returns
    /// A mutable reference to the builder for chaining.
    pub fn jump_condition(mut self, jump_condition: ExprKind) -> Self {
        self.jump_condition = Some(jump_condition);
        self
    }

    /// Builds the `ProcessedInstruction` instance.
    ///
    /// # Returns
    /// A `ProcessedInstruction` populated with the specified values.
    pub fn build(self) -> ProcessedInstruction {
        ProcessedInstruction {
            ssa_id: self.ssa_id,
            node_to_push: self.node_to_push,
            function_parameters: self.function_parameters,
            jump_condition: self.jump_condition,
        }
    }
}

#[derive(Debug, Clone, Default)]
/// Represents a processed instruction
pub struct ProcessedInstruction {
    /// The SSA ID
    pub ssa_id: Option<P<IdentifierNode>>,
    /// The node to push to a region
    pub node_to_push: Option<AstKind>,
    /// The parameters of a function. Returned with Opcode::EndParams.
    pub function_parameters: Option<Vec<ExprKind>>,
    /// The jump condition
    pub jump_condition: Option<ExprKind>,
}
