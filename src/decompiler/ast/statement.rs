#![deny(missing_docs)]

use serde::{Deserialize, Serialize};

use super::{
    expr::{AssignableExpr, ExprNode},
    visitors::AstVisitor,
    AstNodeError,
};
use crate::decompiler::ast::AstNodeTrait;

/// Represents a statement node in the AST, such as `variable = value`.
#[derive(Debug, Clone, Serialize, Deserialize, Eq)]
pub struct StatementNode {
    /// The left-hand side of the statement, usually a variable.
    pub lhs: Box<AssignableExpr>,
    /// The right-hand side of the statement, the value to assign.
    pub rhs: Box<ExprNode>,
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
    pub fn new(lhs: Box<AssignableExpr>, rhs: Box<ExprNode>) -> Result<Self, AstNodeError> {
        Ok(Self { lhs, rhs })
    }
}

impl AstNodeTrait for StatementNode {
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
    use super::*;
    use crate::decompiler::ast::expr::AssignableExpr;
    use crate::decompiler::ast::{expr::ExprNode, identifier::IdentifierNode};

    #[test]
    fn test_assignment_node_eq() {
        let lhs1 = Box::new(AssignableExpr::Identifier(IdentifierNode::new(
            "variable".to_string(),
        )));
        let rhs1 = Box::new(ExprNode::Assignable(AssignableExpr::Identifier(
            IdentifierNode::new("value".to_string()),
        )));
        let lhs2 = Box::new(AssignableExpr::Identifier(IdentifierNode::new(
            "variable".to_string(),
        )));
        let rhs2 = Box::new(ExprNode::Assignable(AssignableExpr::Identifier(
            IdentifierNode::new("value".to_string()),
        )));
        let lhs3 = Box::new(AssignableExpr::Identifier(IdentifierNode::new(
            "variable".to_string(),
        )));
        let rhs3 = Box::new(ExprNode::Assignable(AssignableExpr::Identifier(
            IdentifierNode::new("different_value".to_string()),
        )));

        let node1 = StatementNode::new(lhs1, rhs1).unwrap();
        let node2 = StatementNode::new(lhs2, rhs2).unwrap();
        let node3 = StatementNode::new(lhs3, rhs3).unwrap();

        assert_eq!(node1, node2);
        assert_ne!(node1, node3);
    }
}
