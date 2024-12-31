#![deny(missing_docs)]

use std::fmt::Display;

use emit::EmitContext;
use literal::LiteralNode;
use meta::MetaNode;
use serde::{Deserialize, Serialize};

/// Contains the emitting context for the AST.
pub mod emit;
/// Contains the specifications for any AstNodes that are literals.
pub mod literal;
/// Contains the specifications for any AstNodes that are metadata.
pub mod meta;

/// Trait for all AST nodes.
pub trait AstNodeTrait {
    /// Emits the AST node as a string.
    ///
    /// # Arguments
    /// - `ctx` - The context in which to emit the AST node.
    ///
    /// # Returns
    /// The AST node as a string.
    fn emit(&self, ctx: &EmitContext) -> String;
}

/// Represents an AST node.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AstNode {
    // Assignment(AssignmentNode),
    // BinaryOperation(BinaryOperationNode),
    // UnaryOperation(UnaryOperationNode),
    // ControlFlow(ControlFlowNode),
    /// Represents a literal node in the AST.
    Literal(LiteralNode),
    // Identifier(IdentifierNode),
    // MemberAccess(MemberAccessNode),
    // FunctionCall(FunctionCallNode),
    // Allocation(AllocationNode),
    // Array(ArrayNode),
    // Return(ReturnNode),
    // Phi(PhiNode),
    /// Represents a metadata node in the AST.
    Meta(MetaNode), // Covers comments or annotations
}

impl AstNodeTrait for AstNode {
    fn emit(&self, ctx: &EmitContext) -> String {
        match self {
            AstNode::Literal(literal) => literal.emit(ctx),
            AstNode::Meta(meta) => meta.emit(ctx),
        }
    }
}

// == Other Implementations for AstNode ==
impl Display for AstNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // TODO: Implement EmitContext for AstNode
        write!(f, "{}", self.emit(&EmitContext::default()))
    }
}
