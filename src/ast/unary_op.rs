#![deny(missing_docs)]

use serde::{Deserialize, Serialize};

use super::{
    emit::{EmitContext, EmitError},
    expr::ExprNode,
    visitors::AstVisitor,
    AstNodeError,
};
use crate::ast::AstNodeTrait;

/// Represents a unary operation type in the AST.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub enum UnaryOpType {
    /// Negation (`-`)
    Negate,
    /// Logical NOT (`!`)
    LogicalNot,
    /// Bitwise NOT (`~`)
    BitwiseNot,
}

/// Represents a unary operation node in the AST, such as `-a` or `!b`.
#[derive(Debug, Clone, Serialize, Deserialize, Eq)]
pub struct UnaryOperationNode {
    /// The operand of the unary operation.
    pub operand: Box<ExprNode>,
    /// The unary operation type.
    pub op_type: UnaryOpType,
}

impl UnaryOperationNode {
    /// Creates a new `UnaryOperationNode` after validating the operand.
    ///
    /// # Arguments
    /// - `operand` - The operand for the unary operation.
    /// - `op_type` - The unary operation type.
    ///
    /// # Returns
    /// A new `UnaryOperationNode`.
    ///
    /// # Errors
    /// Returns an `AstNodeError` if `operand` is of an unsupported type.
    pub fn new(operand: Box<ExprNode>, op_type: UnaryOpType) -> Result<Box<Self>, AstNodeError> {
        Self::validate_operand(&operand)?;

        Ok(Box::new(Self { operand, op_type }))
    }

    fn validate_operand(expr: &ExprNode) -> Result<(), AstNodeError> {
        // Most expressions are ok except for string literals.
        if let ExprNode::Literal(crate::ast::literal::LiteralNode::String(_)) = expr {
            return Err(AstNodeError::InvalidOperand(
                "BinaryOperationNode".to_string(),
                "Unsupported operand type".to_string(),
                vec!["LiteralNode".to_string()],
                format!("{:?}", expr),
            ));
        }
        Ok(())
    }

    /// Returns the number of stack values to pop for the unary operation node.
    ///
    /// # Returns
    /// `1`, as unary operations involve only one operand.
    pub fn stack_values_to_pop(&self) -> usize {
        1
    }
}

impl AstNodeTrait for UnaryOperationNode {
    /// Emits the unary operation node as a string.
    ///
    /// # Arguments
    /// - `ctx` - The emitting context.
    ///
    /// # Returns
    /// The emitted string representation of the unary operation.
    fn emit(&self, ctx: &EmitContext) -> Result<String, EmitError> {
        let mut operand_str = self.operand.emit(ctx)?;
        if let ExprNode::BinOp(_) = *self.operand {
            operand_str = format!("({})", operand_str);
        }
        let op_str = match self.op_type {
            UnaryOpType::Negate => "-",
            UnaryOpType::LogicalNot => "!",
            UnaryOpType::BitwiseNot => "~",
        };

        Ok(format!("{}{}", op_str, operand_str))
    }

    fn accept(&self, visitor: &mut dyn AstVisitor) {
        visitor.visit_unary_op(self);
    }
}

// == Other implementations for unary operations ==
impl PartialEq for UnaryOperationNode {
    fn eq(&self, other: &Self) -> bool {
        self.operand == other.operand && self.op_type == other.op_type
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{expr::ExprNode, identifier::IdentifierNode};

    #[test]
    fn test_unary_operation_node_eq() {
        let operand = ExprNode::Identifier(IdentifierNode::new("x".to_string()));
        let node1 = UnaryOperationNode::new(operand.clone_box(), UnaryOpType::Negate).unwrap();
        let node2 = UnaryOperationNode::new(operand.clone_box(), UnaryOpType::Negate).unwrap();
        let node3 = UnaryOperationNode::new(operand.clone_box(), UnaryOpType::LogicalNot).unwrap();

        assert_eq!(node1, node2);
        assert_ne!(node1, node3);
    }

    #[test]
    fn test_unary_operation_node_emit() {
        let operand = ExprNode::Identifier(IdentifierNode::new("b".to_string()));
        let node = UnaryOperationNode::new(operand.clone_box(), UnaryOpType::LogicalNot).unwrap();

        let ctx = EmitContext::default();
        let emitted = node.emit(&ctx).unwrap();
        assert_eq!(emitted, "!b");
    }

    #[test]
    fn test_unary_operation_node_stack_values_to_pop() {
        let operand = ExprNode::Identifier(IdentifierNode::new("x".to_string()));
        let node = UnaryOperationNode::new(operand.clone_box(), UnaryOpType::BitwiseNot).unwrap();

        assert_eq!(node.stack_values_to_pop(), 1);
    }

    #[test]
    fn test_unary_operation_node_new_invalid_operand() {
        let operand = ExprNode::Literal(crate::ast::literal::LiteralNode::String(
            "string".to_string(),
        ))
        .clone_box();
        let result = UnaryOperationNode::new(operand, UnaryOpType::Negate);
        assert!(result.is_err());
    }
}
