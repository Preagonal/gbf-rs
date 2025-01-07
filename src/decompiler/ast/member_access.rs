#![deny(missing_docs)]

use serde::{Deserialize, Serialize};

use super::{expr::AssignableExpr, visitors::AstVisitor, AstNodeError};
use crate::decompiler::ast::AstNodeTrait;

/// Represents a member access node in the AST, such as `object.field`.
#[derive(Debug, Clone, Serialize, Deserialize, Eq)]
pub struct MemberAccessNode {
    /// The left-hand side of the member access, such as `object`.
    pub lhs: Box<AssignableExpr>,
    /// The right-hand side of the member access, such as `field`.
    pub rhs: Box<AssignableExpr>,
}

impl MemberAccessNode {
    /// Creates a new `MemberAccessNode` after validating `lhs` and `rhs` types.
    ///
    /// # Arguments
    /// - `lhs` - The left-hand side of the member access.
    /// - `rhs` - The right-hand side of the member access.
    ///
    /// # Returns
    /// A new `MemberAccessNode`.
    ///
    /// # Errors
    /// Returns an `AstNodeError` if `lhs` or `rhs` is of an unsupported type.
    pub fn new(lhs: Box<AssignableExpr>, rhs: Box<AssignableExpr>) -> Result<Self, AstNodeError> {
        Self::validate_operand(&lhs)?;
        Self::validate_operand(&rhs)?;

        Ok(Self { lhs, rhs })
    }

    fn validate_operand(expr: &AssignableExpr) -> Result<(), AstNodeError> {
        match expr {
            AssignableExpr::Identifier(_) | AssignableExpr::MemberAccess(_) => Ok(()),
            _ => Err(AstNodeError::InvalidOperand(
                "MemberAccessNode".to_string(),
                "Unsupported operand type".to_string(),
                vec!["IdentifierNode".to_string(), "MemberAccessNode".to_string()],
                format!("{:?}", expr),
            )),
        }
    }
}

impl AstNodeTrait for MemberAccessNode {
    fn accept(&self, visitor: &mut dyn AstVisitor) {
        visitor.visit_member_access(self);
    }
}

// == Other implementations for member access ==
impl PartialEq for MemberAccessNode {
    fn eq(&self, other: &Self) -> bool {
        self.lhs == other.lhs && self.rhs == other.rhs
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decompiler::ast::{expr::AssignableExpr, identifier::IdentifierNode};

    #[test]
    fn test_member_access_node_eq() {
        let lhs1 = AssignableExpr::Identifier(IdentifierNode::new("object".to_string()));
        let rhs1 = AssignableExpr::Identifier(IdentifierNode::new("property".to_string()));
        let lhs2 = AssignableExpr::Identifier(IdentifierNode::new("object".to_string()));
        let rhs2 = AssignableExpr::Identifier(IdentifierNode::new("property".to_string()));
        let lhs3 = AssignableExpr::Identifier(IdentifierNode::new("object".to_string()));
        let rhs3 =
            AssignableExpr::Identifier(IdentifierNode::new("different_property".to_string()));

        let node1 = MemberAccessNode::new(Box::new(lhs1), Box::new(rhs1)).unwrap();
        let node2 = MemberAccessNode::new(Box::new(lhs2), Box::new(rhs2)).unwrap();
        let node3 = MemberAccessNode::new(Box::new(lhs3), Box::new(rhs3)).unwrap();

        assert_eq!(node1, node2);
        assert_ne!(node1, node3);
    }
}
