#![deny(missing_docs)]

use gbf_macros::AstNodeTransform;
use serde::{Deserialize, Serialize};

use super::{
    assignment::AssignmentNode, ptr::P, ret::ReturnNode, visitors::AstVisitor, AstKind,
    AstVisitable,
};

/// Represents an expression node in the AST.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, AstNodeTransform)]
#[convert_to(AstKind::Statement)]
pub enum StatementKind {
    /// Assignment
    Assignment(P<AssignmentNode>),
    /// Return
    Return(P<ReturnNode>),
    // Goto
    // Goto(GotoNode),
}

impl AstVisitable for StatementKind {
    fn accept<V: AstVisitor>(&self, visitor: &mut V) -> V::Output {
        visitor.visit_statement(self)
    }
}

// == Other implementations for literal ==

impl PartialEq for StatementKind {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (StatementKind::Assignment(a1), StatementKind::Assignment(a2)) => a1 == a2,
            (StatementKind::Return(r1), StatementKind::Return(r2)) => r1 == r2,
            _ => false,
        }
    }
}
