#![deny(missing_docs)]

use gbf_macros::AstNodeTransform;
use serde::{Deserialize, Serialize};

use super::{
    assignable::AssignableKind, bin_op::BinaryOperationNode, func_call::FunctionCallNode,
    literal::LiteralNode, unary_op::UnaryOperationNode, visitors::AstVisitor, AstKind,
    AstVisitable,
};

/// Represents an expression node in the AST.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, AstNodeTransform)]
#[convert_to(AstKind::Expression)]
pub enum ExprKind {
    /// Represents a literal node in the AST.
    Literal(LiteralNode),
    /// Represents an assignable expression node in the AST.
    Assignable(AssignableKind),
    /// Represents a binary operation node in the AST.
    BinOp(BinaryOperationNode),
    /// Represents a unary operation node in the AST.
    UnaryOp(UnaryOperationNode),
    /// Represents a function call node in the AST.
    FunctionCall(FunctionCallNode),
}

impl AstVisitable for ExprKind {
    fn accept(&self, visitor: &mut dyn AstVisitor) {
        visitor.visit_expr(self);
    }
}

// == Other implementations for literal ==

impl PartialEq for ExprKind {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ExprKind::Literal(l1), ExprKind::Literal(l2)) => l1 == l2,
            (ExprKind::Assignable(a1), ExprKind::Assignable(a2)) => a1 == a2,
            (ExprKind::BinOp(b1), ExprKind::BinOp(b2)) => b1 == b2,
            (ExprKind::UnaryOp(u1), ExprKind::UnaryOp(u2)) => u1 == u2,
            _ => false,
        }
    }
}
