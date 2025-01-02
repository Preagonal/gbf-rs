#![deny(missing_docs)]

use serde::{Deserialize, Serialize};

use super::{
    emit::{EmitContext, EmitError},
    expr::ExprNode,
    AstNodeError,
};
use crate::ast::AstNodeTrait;

/// Represents a statement node in the AST, such as `variable = value`.
#[derive(Debug, Clone, Serialize, Deserialize, Eq)]
pub struct StatementNode {
    /// The left-hand side of the statement, usually a variable.
    pub lhs: Box<ExprNode>,
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
    pub fn new(lhs: ExprNode, rhs: ExprNode) -> Result<Self, AstNodeError> {
        Self::validate_lhs(&lhs)?;
        // TODO: Verify all rhs types are valid
        // Self::validate_rhs(&rhs)?;
        Ok(Self {
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        })
    }

    fn validate_lhs(expr: &ExprNode) -> Result<(), AstNodeError> {
        match expr {
            ExprNode::Identifier(_) => Ok(()),
            ExprNode::MemberAccess(_) => Ok(()),
            _ => Err(AstNodeError::InvalidOperand(
                "StatementNode".to_string(),
                "Unsupported left-hand side type".to_string(),
                vec!["IdentifierNode".to_string()],
                format!("{:?}", expr),
            )),
        }
    }

    /// Emits the statement as a string.
    ///
    /// # Arguments
    /// - `ctx` - The emitting context.
    ///
    /// # Returns
    /// The emitted string representation of the statement.
    pub fn emit(&self, ctx: &EmitContext) -> Result<String, EmitError> {
        let lhs_str = self.lhs.emit(ctx)?;
        let rhs_str = self.rhs.emit(ctx)?;
        Ok(format!("{} = {};", lhs_str, rhs_str))
    }

    /// Returns the number of stack values to pop for the statement node.
    ///
    /// # Returns
    /// `1`, as statement pops the last two expressions from the stack.
    pub fn stack_values_to_pop(&self) -> usize {
        2
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
    use crate::ast::expr::ExprNode;

    #[test]
    fn test_assignment_node_eq() {
        let lhs1 = ExprNode::Identifier("variable".to_string());
        let rhs1 = ExprNode::Identifier("value".to_string());
        let lhs2 = ExprNode::Identifier("variable".to_string());
        let rhs2 = ExprNode::Identifier("value".to_string());
        let lhs3 = ExprNode::Identifier("variable".to_string());
        let rhs3 = ExprNode::Identifier("different_value".to_string());

        let node1 = StatementNode::new(lhs1.clone(), rhs1.clone()).unwrap();
        let node2 = StatementNode::new(lhs2.clone(), rhs2.clone()).unwrap();
        let node3 = StatementNode::new(lhs3.clone(), rhs3.clone()).unwrap();

        assert_eq!(node1, node2);
        assert_ne!(node1, node3);
    }

    #[test]
    fn test_assignment_node_emit() {
        let lhs = ExprNode::Identifier("variable".to_string());
        let rhs = ExprNode::Identifier("value".to_string());
        let node = StatementNode::new(lhs.clone(), rhs.clone()).unwrap();

        let ctx = EmitContext::default();
        let emitted = node.emit(&ctx).unwrap();
        assert_eq!(emitted, "variable = value;");
    }

    #[test]
    fn test_assignment_node_stack_values_to_pop() {
        let lhs = ExprNode::Identifier("variable".to_string());
        let rhs = ExprNode::Identifier("value".to_string());
        let node = StatementNode::new(lhs.clone(), rhs.clone()).unwrap();

        assert_eq!(node.stack_values_to_pop(), 2);
    }
}
