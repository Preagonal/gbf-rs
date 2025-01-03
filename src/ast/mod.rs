#![deny(missing_docs)]

use crate::ast::visitors::AstVisitor;
use expr::ExprNode;
use meta::MetaNode;
use serde::{Deserialize, Serialize};
use statement::StatementNode;
use thiserror::Error;

/// Represents binary operations in the AST.
pub mod bin_op;
/// Contains the specifications for any AstNodes that are expressions
pub mod expr;
/// Contains the specifications for any AstNodes that are identifiers.
pub mod identifier;
/// Contains the specifications for any AstNodes that are literals.
pub mod literal;
/// Contains the specifications for any AstNodes that are member accesses.
pub mod member_access;
/// Contains the specifications for any AstNodes that are metadata.
pub mod meta;
/// Contains the specifications for any AstNodes that are statements
pub mod statement;
/// Represents unary operations in the AST.
pub mod unary_op;
/// Represents the visitor pattern for the AST.
pub mod visitors;

/// Represents an error that occurred while converting an AST node.
#[derive(Debug, Error)]
pub enum AstNodeError {
    /// Invalid conversion from AstNode to another type.
    #[error("Expected {0}, found {1}")]
    InvalidConversion(String, String),

    /// Invalid operand for an AST node.
    #[error("Invalid {0} operand for {1}. Expected types in {2:?}, found {3}")]
    InvalidOperand(String, String, Vec<String>, String),
}

/// Trait for all AST nodes.
pub trait AstNodeTrait: Clone {
    /// Clones the AST node as a boxed trait object.
    fn clone_box(&self) -> Box<Self>
    where
        Self: Sized,
    {
        Box::new(self.clone())
    }

    /// Accepts a visitor for the AST node.
    fn accept(&self, visitor: &mut dyn AstVisitor);
}

/// Represents an AST node.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AstNode {
    /// Represents a statement node in the AST, such as `variable = value;`.
    Statement(StatementNode),
    // ControlFlow(ControlFlowNode),
    /// Represents a literal node in the AST.
    Expression(ExprNode),
    // FunctionCall(FunctionCallNode),
    // Allocation(AllocationNode),
    // Array(ArrayNode),
    // Return(ReturnNode),
    // Phi(PhiNode),
    /// Represents a metadata node in the AST.
    Meta(MetaNode), // Covers comments or annotations
}

impl AstNodeTrait for AstNode {
    fn accept(&self, visitor: &mut dyn AstVisitor) {
        match self {
            AstNode::Expression(expr) => expr.accept(visitor),
            AstNode::Meta(meta) => meta.accept(visitor),
            AstNode::Statement(stmt) => stmt.accept(visitor),
        }
    }
}
