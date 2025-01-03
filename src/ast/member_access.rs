#![deny(missing_docs)]

use serde::{Deserialize, Serialize};

use super::{expr::ExprNode, visitors::AstVisitor, AstNodeError};
use crate::ast::AstNodeTrait;

/// Represents a member access node in the AST, such as `object.field`.
#[derive(Debug, Clone, Serialize, Deserialize, Eq)]
pub struct MemberAccessNode {
    /// The left-hand side of the member access, such as `object`.
    pub lhs: Box<ExprNode>,
    /// The right-hand side of the member access, such as `field`.
    pub rhs: Box<ExprNode>,
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
    pub fn new(lhs: Box<ExprNode>, rhs: Box<ExprNode>) -> Result<Box<Self>, AstNodeError> {
        Self::validate_operand(&lhs)?;
        Self::validate_operand(&rhs)?;

        Ok(Box::new(Self { lhs, rhs }))
    }

    fn validate_operand(expr: &ExprNode) -> Result<(), AstNodeError> {
        match expr {
            ExprNode::Identifier(_) | ExprNode::MemberAccess(_) => Ok(()),
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
    use crate::ast::{expr::ExprNode, identifier::IdentifierNode, literal::LiteralNode};

    #[test]
    fn test_member_access_node_eq() {
        let lhs1 = ExprNode::Identifier(IdentifierNode::new("object".to_string()));
        let rhs1 = ExprNode::Identifier(IdentifierNode::new("property".to_string()));
        let lhs2 = ExprNode::Identifier(IdentifierNode::new("object".to_string()));
        let rhs2 = ExprNode::Identifier(IdentifierNode::new("property".to_string()));
        let lhs3 = ExprNode::Identifier(IdentifierNode::new("object".to_string()));
        let rhs3 = ExprNode::Identifier(IdentifierNode::new("different_property".to_string()));

        let node1 = MemberAccessNode::new(Box::new(lhs1), Box::new(rhs1)).unwrap();
        let node2 = MemberAccessNode::new(Box::new(lhs2), Box::new(rhs2)).unwrap();
        let node3 = MemberAccessNode::new(Box::new(lhs3), Box::new(rhs3)).unwrap();

        assert_eq!(node1, node2);
        assert_ne!(node1, node3);
    }

    #[test]
    fn test_member_access_node_new_invalid_operand() {
        // literals are not allowed; 1.temp or temp.1 is invalid
        let lhs = ExprNode::Literal(LiteralNode::String("Hello, world!".to_string()));
        let rhs = ExprNode::Literal(LiteralNode::String("Hello, world!".to_string()));

        let result = MemberAccessNode::new(Box::new(lhs), Box::new(rhs));
        assert!(result.is_err());

        // test left side with invalid operand number
        let lhs = ExprNode::Literal(LiteralNode::Number(42));
        let rhs = ExprNode::Identifier(IdentifierNode::new("property".to_string()));

        let result = MemberAccessNode::new(Box::new(lhs), Box::new(rhs));
        assert!(result.is_err());

        // test right side with invalid operand number
        let lhs = ExprNode::Identifier(IdentifierNode::new("object".to_string()));
        let rhs = ExprNode::Literal(LiteralNode::Number(42));

        let result = MemberAccessNode::new(Box::new(lhs), Box::new(rhs));
        assert!(result.is_err());
    }
}
