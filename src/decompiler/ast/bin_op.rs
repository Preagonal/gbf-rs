#![deny(missing_docs)]

use serde::{Deserialize, Serialize};

use super::{expr::ExprNode, AstNodeError};
use crate::decompiler::ast::literal::LiteralNode;
use crate::decompiler::ast::AstNodeTrait;

/// Represents a binary operation type in the AST.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub enum BinOpType {
    /// Addition operation (`+`)
    Add,
    /// Subtraction operation (`-`)
    Sub,
    /// Multiplication operation (`*`)
    Mul,
    /// Division operation (`/`)
    Div,
    /// Modulo operation (`%`)
    Mod,
    /// Bitwise AND (`&`)
    And,
    /// Bitwise OR (`|`)
    Or,
    /// Bitwise XOR (`xor`)
    Xor,
    /// Logical AND (`&&`)
    LogicalAnd,
    /// Logical OR (`||`)
    LogicalOr,
    /// Equality (`==`)
    Equal,
    /// Inequality (`!=`)
    NotEqual,
    /// Greater than (`>`)
    Greater,
    /// Less than (`<`)
    Less,
    /// Greater than or equal (`>=`)
    GreaterOrEqual,
    /// Less than or equal (`<=`)
    LessOrEqual,
    /// Shift left (`<<`)
    ShiftLeft,
    /// Shift right (`>>`)
    ShiftRight,
    /// In (`in`)
    In,
    /// Join (`@`)
    Join,
}

/// Represents a binary operation node in the AST, such as `a + b`.
#[derive(Debug, Clone, Serialize, Deserialize, Eq)]
pub struct BinaryOperationNode {
    /// The left-hand side of the binary operation.
    pub lhs: Box<ExprNode>,
    /// The right-hand side of the binary operation.
    pub rhs: Box<ExprNode>,
    /// The binary operation type.
    pub op_type: BinOpType,
}

impl BinaryOperationNode {
    /// Creates a new `BinaryOperationNode` after validating `lhs` and `rhs`.
    ///
    /// # Arguments
    /// - `lhs` - The left-hand side expression.
    /// - `rhs` - The right-hand side expression.
    /// - `op_type` - The binary operation type.
    ///
    /// # Returns
    /// A new `BinaryOperationNode`.
    ///
    /// # Errors
    /// Returns an `AstNodeError` if `lhs` or `rhs` is of an unsupported type.
    pub fn new(
        lhs: Box<ExprNode>,
        rhs: Box<ExprNode>,
        op_type: BinOpType,
    ) -> Result<Self, AstNodeError> {
        Self::validate_operand(&lhs)?;
        Self::validate_operand(&rhs)?;

        Ok(Self { lhs, rhs, op_type })
    }

    fn validate_operand(expr: &ExprNode) -> Result<(), AstNodeError> {
        // Most expressions are ok except for string literals.
        if let ExprNode::Literal(LiteralNode::String(_)) = expr {
            return Err(AstNodeError::InvalidOperand(
                "BinaryOperationNode".to_string(),
                "Unsupported operand type".to_string(),
                vec!["LiteralNode".to_string()],
                format!("{:?}", expr),
            ));
        }
        Ok(())
    }
}

// == Other implementations for binary operations ==
impl PartialEq for BinaryOperationNode {
    fn eq(&self, other: &Self) -> bool {
        self.lhs == other.lhs && self.rhs == other.rhs && self.op_type == other.op_type
    }
}

impl AstNodeTrait for BinaryOperationNode {
    fn accept(&self, visitor: &mut dyn super::visitors::AstVisitor) {
        visitor.visit_bin_op(self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decompiler::ast::{
        expr::ExprNode, identifier::IdentifierNode, literal::LiteralNode,
    };

    #[test]
    fn test_binary_operation_node_eq() {
        let lhs = ExprNode::Identifier(IdentifierNode::new("x".to_string()));
        let rhs = ExprNode::Identifier(IdentifierNode::new("y".to_string()));
        let node1 =
            BinaryOperationNode::new(lhs.clone_box(), rhs.clone_box(), BinOpType::Add).unwrap();
        let node2 =
            BinaryOperationNode::new(lhs.clone_box(), rhs.clone_box(), BinOpType::Add).unwrap();
        let node3 =
            BinaryOperationNode::new(lhs.clone_box(), rhs.clone_box(), BinOpType::Sub).unwrap();

        assert_eq!(node1, node2);
        assert_ne!(node1, node3);
    }

    #[test]
    fn test_binary_operation_node_new_invalid_operand() {
        let lhs = ExprNode::Literal(LiteralNode::String("invalid".to_string()));
        let rhs = ExprNode::Identifier(IdentifierNode::new("y".to_string()));

        let result = BinaryOperationNode::new(Box::new(lhs), Box::new(rhs), BinOpType::Sub);
        assert!(result.is_err());
    }
}
