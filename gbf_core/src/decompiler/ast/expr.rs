#![deny(missing_docs)]

use gbf_macros::AstNodeTransform;
use serde::{Deserialize, Serialize};

use super::{
    array::ArrayNode, array_access::ArrayAccessNode, bin_op::BinaryOperationNode,
    func_call::FunctionCallNode, identifier::IdentifierNode, literal::LiteralNode,
    member_access::MemberAccessNode, new::NewNode, phi::PhiNode, ptr::P,
    unary_op::UnaryOperationNode, visitors::AstVisitor, AstKind, AstVisitable,
};

/// Represents an expression node in the AST.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, AstNodeTransform)]
#[convert_to(AstKind::Expression)]
pub enum ExprKind {
    /// Represents a literal node in the AST.
    Literal(P<LiteralNode>),
    /// Represents a binary operation node in the AST.
    BinOp(P<BinaryOperationNode>),
    /// Represents a unary operation node in the AST.
    UnaryOp(P<UnaryOperationNode>),
    /// Represents a function call node in the AST.
    FunctionCall(P<FunctionCallNode>),
    /// Represents an array node in the AST.
    Array(P<ArrayNode>),
    /// Represents a new node in the AST.
    New(P<NewNode>),
    /// Represents a member access node in the AST.
    MemberAccess(P<MemberAccessNode>),
    /// Represents an identifier node in the AST.
    Identifier(P<IdentifierNode>),
    /// Represents an array access node in the AST.
    ArrayAccess(P<ArrayAccessNode>),
    /// Represents a phi node in the AST.
    Phi(P<PhiNode>),
}

impl AstVisitable for ExprKind {
    fn accept<V: AstVisitor>(&self, visitor: &mut V) -> V::Output {
        visitor.visit_expr(self)
    }
}

// == Other implementations for literal ==

impl PartialEq for ExprKind {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ExprKind::Literal(l1), ExprKind::Literal(l2)) => l1 == l2,
            (ExprKind::BinOp(b1), ExprKind::BinOp(b2)) => b1 == b2,
            (ExprKind::UnaryOp(u1), ExprKind::UnaryOp(u2)) => u1 == u2,
            (ExprKind::FunctionCall(f1), ExprKind::FunctionCall(f2)) => f1 == f2,
            (ExprKind::Array(a1), ExprKind::Array(a2)) => a1 == a2,
            (ExprKind::New(n1), ExprKind::New(n2)) => n1 == n2,
            (ExprKind::MemberAccess(m1), ExprKind::MemberAccess(m2)) => m1 == m2,
            (ExprKind::Identifier(i1), ExprKind::Identifier(i2)) => i1 == i2,
            (ExprKind::ArrayAccess(a1), ExprKind::ArrayAccess(a2)) => a1 == a2,
            (ExprKind::Phi(p1), ExprKind::Phi(p2)) => p1 == p2,
            _ => false,
        }
    }
}
