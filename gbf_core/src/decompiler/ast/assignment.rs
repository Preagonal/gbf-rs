#![deny(missing_docs)]

use gbf_macros::AstNodeTransform;
use serde::{Deserialize, Serialize};

use super::{
    expr::ExprKind, ptr::P, statement::StatementKind, visitors::AstVisitor, AstKind, AstNodeError,
};
use crate::decompiler::ast::AstVisitable;

/// Represents a statement node in the AST, such as `variable = value`.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, AstNodeTransform)]
#[convert_to(StatementKind::Assignment, AstKind::Statement)]
pub struct AssignmentNode {
    /// The left-hand side of the statement, usually a variable.
    pub lhs: ExprKind,
    /// The right-hand side of the statement, the value to assign.
    pub rhs: ExprKind,
}

impl AssignmentNode {
    /// Creates a new `StatementNode` after validating `lhs` and `rhs` types.
    ///
    /// # Arguments
    /// - `lhs` - The left-hand side of the statement.
    /// - `rhs` - The right-hand side of the statement.
    ///
    /// # Returns
    /// A new `StatementNode`.
    ///
    /// # Errors
    /// Returns an `AstNodeError` if `lhs` or `rhs` is of an unsupported type.
    pub fn new(lhs: ExprKind, rhs: ExprKind) -> Result<Self, AstNodeError> {
        Ok(Self { lhs, rhs })
    }
}

impl AstVisitable for P<AssignmentNode> {
    /// Accepts the visitor and calls the appropriate visit method.
    fn accept<V: AstVisitor>(&self, visitor: &mut V) -> V::Output {
        visitor.visit_assignment(self)
    }
}

// == Other implementations for statement ==
impl PartialEq for AssignmentNode {
    fn eq(&self, other: &Self) -> bool {
        self.lhs == other.lhs && self.rhs == other.rhs
    }
}

#[cfg(test)]
mod tests {
    use crate::decompiler::ast::{
        emit, new_assignment, new_id, new_member_access, new_str, AstNodeError,
    };

    #[test]
    fn test_statement_emit() -> Result<(), AstNodeError> {
        let stmt = new_assignment(new_id("test1"), new_id("test2"));
        // this becomes a statement because of the `into`, so it should end with a semicolon
        assert_eq!(emit(stmt), "test1 = test2;");

        // player.chat = "Hello, world!";
        let stmt = new_assignment(
            new_member_access(new_id("player"), new_id("chat"))?,
            new_str("Hello, world!"),
        );
        assert_eq!(emit(stmt), "player.chat = \"Hello, world!\";");
        Ok(())
    }
}
