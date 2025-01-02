#![deny(missing_docs)]

use serde::{Deserialize, Serialize};

use super::{
    emit::{EmitContext, EmitError},
    expr::ExprNode,
    AstNodeError,
};
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
    pub fn new(lhs: ExprNode, rhs: ExprNode) -> Result<Self, AstNodeError> {
        Self::validate_operand(&lhs)?;
        Self::validate_operand(&rhs)?;

        Ok(Self {
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        })
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

    /// Emits the member access node as a string.
    ///
    /// # Arguments
    /// - `ctx` - The emitting context.
    ///
    /// # Returns
    /// The emitted string representation of the member access.
    pub fn emit(&self, ctx: &EmitContext) -> Result<String, EmitError> {
        let lhs_str = self.lhs.emit(ctx)?;
        let rhs_str = self.rhs.emit(ctx)?;
        Ok(format!("{}.{}", lhs_str, rhs_str))
    }

    /// Returns the number of stack values to pop for the member access node.
    ///
    /// # Returns
    /// `2`, as member access always involves `lhs` and `rhs`.
    pub fn stack_values_to_pop(&self) -> usize {
        2
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
    use crate::ast::expr::ExprNode;

    #[test]
    fn test_member_access_node_eq() {
        let lhs1 = ExprNode::Identifier("object".to_string());
        let rhs1 = ExprNode::Identifier("property".to_string());
        let lhs2 = ExprNode::Identifier("object".to_string());
        let rhs2 = ExprNode::Identifier("property".to_string());
        let lhs3 = ExprNode::Identifier("object".to_string());
        let rhs3 = ExprNode::Identifier("different_property".to_string());

        let node1 = MemberAccessNode::new(lhs1.clone(), rhs1.clone()).unwrap();
        let node2 = MemberAccessNode::new(lhs2.clone(), rhs2.clone()).unwrap();
        let node3 = MemberAccessNode::new(lhs3.clone(), rhs3.clone()).unwrap();

        assert_eq!(node1, node2);
        assert_ne!(node1, node3);
    }

    #[test]
    fn test_member_access_node_emit() {
        let lhs = ExprNode::Identifier("temp".to_string());
        let rhs = ExprNode::Identifier("foo".to_string());
        let node = MemberAccessNode::new(lhs.clone(), rhs.clone()).unwrap();

        let ctx = EmitContext::default();
        let emitted = node.emit(&ctx).unwrap();
        assert_eq!(emitted, "temp.foo");

        // now test case where left is temp and right is another member access
        let lhs = ExprNode::Identifier("temp".to_string());
        let rhs = ExprNode::MemberAccess(
            MemberAccessNode::new(
                ExprNode::Identifier("foo".to_string()),
                ExprNode::Identifier("bar".to_string()),
            )
            .unwrap(),
        );
        let node = MemberAccessNode::new(lhs.clone(), rhs.clone()).unwrap();

        let ctx = EmitContext::default();
        let emitted = node.emit(&ctx).unwrap();
        assert_eq!(emitted, "temp.foo.bar");
    }

    #[test]
    fn test_member_access_node_stack_values_to_pop() {
        let lhs = ExprNode::Identifier("object".to_string());
        let rhs = ExprNode::Identifier("property".to_string());
        let node = MemberAccessNode::new(lhs.clone(), rhs.clone()).unwrap();

        assert_eq!(node.stack_values_to_pop(), 2);
    }
}
