#![deny(missing_docs)]

use crate::decompiler::ast::visitors::AstVisitor;
use expr::{AssignableExpr, ExprNode};
use func_call::FunctionCallNode;
use identifier::IdentifierNode;
use literal::LiteralNode;
use member_access::MemberAccessNode;
use meta::MetaNode;
use serde::{Deserialize, Serialize};
use statement::StatementNode;
use thiserror::Error;
use visitors::{emit_context::EmitContext, emitter::Gs2Emitter};

/// Represents binary operations in the AST.
pub mod bin_op;
/// Contains the specifications for any AstNodes that are expressions
pub mod expr;
/// Contains the specifications for any AstNodes that are function calls.
pub mod func_call;
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
    // Allocation(AllocationNode),
    // Array(ArrayNode),
    // Return(ReturnNode),
    // Phi(PhiNode),
    /// Represents a metadata node in the AST.
    Meta(MetaNode), // Covers comments or annotations
    /// This node does nothing. It should only be used for debugging purposes.
    Empty,
}

impl AstNodeTrait for AstNode {
    fn accept(&self, visitor: &mut dyn AstVisitor) {
        match self {
            AstNode::Expression(expr) => expr.accept(visitor),
            AstNode::Meta(meta) => meta.accept(visitor),
            AstNode::Statement(stmt) => stmt.accept(visitor),
            AstNode::Empty => {}
        }
    }
}

/// Emits a node into a string.
pub fn emit(node: &AstNode) -> String {
    let mut emit = Gs2Emitter::new(EmitContext::default());
    node.accept(&mut emit);
    emit.output().to_string()
}

/// Creates a new AstNode for a statement.
pub fn statement(lhs: Box<AssignableExpr>, rhs: Box<ExprNode>) -> StatementNode {
    StatementNode::new(lhs, rhs).unwrap()
}

/// Creates a new member access node.
pub fn member_access(lhs: Box<AssignableExpr>, rhs: Box<AssignableExpr>) -> MemberAccessNode {
    MemberAccessNode::new(lhs, rhs).unwrap()
}

/// Creates a new AssignableExpr for an identifier
pub fn identifier(name: &str) -> IdentifierNode {
    IdentifierNode::new(name)
}

/// Creates a new function call node.
pub fn call(name: &str, args: Vec<ExprNode>) -> FunctionCallNode {
    FunctionCallNode::new(identifier::IdentifierNode::new(name), args, None)
}

// == Literals ==

/// Creates a new ExprNode for a literal string.
pub fn literal_string(value: &str) -> LiteralNode {
    LiteralNode::String(value.to_string())
}

/// Creates a new ExprNode for a literal number.
pub fn literal_number(value: i32) -> LiteralNode {
    LiteralNode::Number(value)
}

/// Creates a new ExprNode for a literal float.
pub fn literal_float(value: String) -> LiteralNode {
    LiteralNode::Float(value)
}

/// Creates a new ExprNode for a literal boolean.
pub fn literal_bool(value: bool) -> LiteralNode {
    LiteralNode::Boolean(value)
}

#[cfg(test)]
mod tests {
    use crate::decompiler::ast::expr::{ToBoxedAssignableExpr, ToBoxedExpr};

    use super::{emit, identifier, literal_number, statement};

    #[test]
    pub fn test_funcs() {
        let node = statement(
            identifier("foo").to_boxed_assignable_expr(),
            literal_number(32).to_boxed_expr(),
        );

        //assert_eq!(emit(node.into()), "foo = 32;");
    }
}
