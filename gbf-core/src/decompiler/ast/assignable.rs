#![deny(missing_docs)]

use gbf_macros::AstNodeTransform;
use serde::{Deserialize, Serialize};

use super::{
    expr::ExprKind, identifier::IdentifierNode, member_access::MemberAccessNode,
    visitors::AstVisitor, AstKind, AstVisitable,
};

/// Represents an assignable expression node in the AST.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, AstNodeTransform)]
#[convert_to(ExprKind::Assignable, AstKind::Expression)]
pub enum AssignableKind {
    /// Represents a member access node in the AST.
    MemberAccess(MemberAccessNode),
    /// Represents an identifier node in the AST.
    Identifier(IdentifierNode),
}

impl AstVisitable for AssignableKind {
    fn accept(&self, visitor: &mut dyn AstVisitor) {
        visitor.visit_assignable_expr(self);
    }
}

impl PartialEq for AssignableKind {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (AssignableKind::MemberAccess(m1), AssignableKind::MemberAccess(m2)) => m1 == m2,
            (AssignableKind::Identifier(i1), AssignableKind::Identifier(i2)) => i1 == i2,
            _ => false,
        }
    }
}
