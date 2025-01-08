#![deny(missing_docs)]

use gbf_macros::AstNodeTransform;
use serde::{Deserialize, Serialize};

use super::{
    assignable::AssignableKind, expr::ExprKind, visitors::AstVisitor, AstKind, AstNodeError,
};
use crate::decompiler::ast::AstVisitable;

/// Represents a statement node in the AST, such as `variable = value`.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, AstNodeTransform)]
#[convert_to(AstKind::Statement)]
pub struct StatementNode {
    /// The left-hand side of the statement, usually a variable.
    pub lhs: Box<AssignableKind>,
    /// The right-hand side of the statement, the value to assign.
    pub rhs: Box<ExprKind>,
}

impl StatementNode {
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
    pub fn new(lhs: Box<AssignableKind>, rhs: Box<ExprKind>) -> Result<Self, AstNodeError> {
        Ok(Self { lhs, rhs })
    }
}

impl AstVisitable for StatementNode {
    /// Accepts the visitor and calls the appropriate visit method.
    fn accept(&self, visitor: &mut dyn AstVisitor) {
        visitor.visit_statement(self);
    }
}

// == Other implementations for statement ==
impl PartialEq for StatementNode {
    fn eq(&self, other: &Self) -> bool {
        self.lhs == other.lhs && self.rhs == other.rhs
    }
}

#[cfg(test)]
mod tests {
    use crate::decompiler::ast::{emit, member_access, new_id, new_str, statement, AstNodeError};

    #[test]
    fn test_statement_emit() -> Result<(), AstNodeError> {
        let var = new_id("test1");
        let var2 = new_id("test2");
        let stmt = statement(var, var2);
        assert_eq!(emit(stmt), "test1 = test2;");

        // player.chat = "Hello, world!";
        let player = new_id("player");
        let chat = new_id("chat");
        let chat_str = new_str("Hello, world!");
        let ma = member_access(player, chat)?;
        let stmt = statement(ma, chat_str);
        assert_eq!(emit(stmt), "player.chat = \"Hello, world!\";");
        Ok(())
    }
}
