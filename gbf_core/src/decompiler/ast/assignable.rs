#![deny(missing_docs)]

use gbf_macros::AstNodeTransform;
use serde::{Deserialize, Serialize};

use super::{
    emit, expr::ExprKind, identifier::IdentifierNode, member_access::MemberAccessNode,
    ssa::SsaVersion, visitors::AstVisitor, AstKind, AstVisitable,
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

impl AssignableKind {
    /// Sets the SSA version of the assignable expression.
    pub fn set_ssa_version(&mut self, ssa_version: SsaVersion) {
        match self {
            AssignableKind::MemberAccess(m) => m.ssa_version = Some(ssa_version),
            AssignableKind::Identifier(i) => i.ssa_version = Some(ssa_version),
        }
    }

    /// Returns the SSA string representation of the assignable expression.
    pub fn id_string(&self) -> String {
        match self {
            AssignableKind::MemberAccess(m) => emit(m.clone()),
            AssignableKind::Identifier(i) => emit(i.clone()),
        }
    }

    /// Returns the SSA version of the assignable expression.
    pub fn ssa_version(&self) -> Option<SsaVersion> {
        match self {
            AssignableKind::MemberAccess(m) => m.ssa_version,
            AssignableKind::Identifier(i) => i.ssa_version,
        }
    }

    /// Remove the SSA version from the assignable expression.
    pub fn remove_ssa_version(&mut self) {
        match self {
            AssignableKind::MemberAccess(m) => m.ssa_version = None,
            AssignableKind::Identifier(i) => i.ssa_version = None,
        }
    }
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
