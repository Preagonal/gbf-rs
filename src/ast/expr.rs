#![deny(missing_docs)]

use serde::{Deserialize, Serialize};

use super::{
    emit::{EmitContext, EmitError},
    literal::LiteralNode,
    member_access::MemberAccessNode,
    AstNodeTrait,
};

/// Represents an expression node in the AST.
#[derive(Debug, Clone, Serialize, Deserialize, Eq)]
pub enum ExprNode {
    /// Represents a literal node in the AST.
    Literal(LiteralNode),
    /// Represents a member access node in the AST.
    MemberAccess(MemberAccessNode),
    /// Represents an identifier node in the AST.
    Identifier(String),
}

impl AstNodeTrait for ExprNode {
    fn emit(&self, ctx: &EmitContext) -> Result<String, EmitError> {
        Ok(match self {
            ExprNode::Literal(literal) => literal.emit(ctx)?,
            ExprNode::MemberAccess(mem) => mem.emit(ctx)?,
            ExprNode::Identifier(s) => s.to_string(),
        })
    }
}

// == Other implementations for literal ==

impl PartialEq for ExprNode {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ExprNode::Literal(l1), ExprNode::Literal(l2)) => l1 == l2,
            (ExprNode::MemberAccess(m1), ExprNode::MemberAccess(m2)) => m1 == m2,
            (ExprNode::Identifier(id1), ExprNode::Identifier(id2)) => id1 == id2,
            _ => false,
        }
    }
}
