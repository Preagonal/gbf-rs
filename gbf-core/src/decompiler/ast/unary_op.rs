#![deny(missing_docs)]

use gbf_macros::AstNodeTransform;
use serde::{Deserialize, Serialize};

use crate::define_ast_enum_type;

use super::{expr::ExprKind, visitors::AstVisitor, AstKind, AstNodeError, AstVisitable};

define_ast_enum_type!(
    UnaryOpType {
        LogicalNot => "!",
        BitwiseNot => "~",
        Negate => "-",
    }
);

/// Represents a unary operation node in the AST, such as `-a` or `!b`.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, AstNodeTransform)]
#[convert_to(ExprKind::UnaryOp, AstKind::Expression)]
pub struct UnaryOperationNode {
    /// The operand of the unary operation.
    pub operand: Box<ExprKind>,
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
    pub fn new(operand: Box<ExprKind>, op_type: UnaryOpType) -> Result<Self, AstNodeError> {
        Self::validate_operand(&operand)?;

        Ok(Self { operand, op_type })
    }

    fn validate_operand(expr: &ExprKind) -> Result<(), AstNodeError> {
        // Most expressions are ok except for string literals.
        if let ExprKind::Literal(crate::decompiler::ast::literal::LiteralNode::String(_)) = expr {
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

impl AstVisitable for UnaryOperationNode {
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
    use crate::decompiler::ast::{
        bin_op, bin_op::BinOpType, emit, identifier, literal_string, unary_op, AstNodeError,
    };

    use super::UnaryOpType;

    #[test]
    fn test_unary_op_emit() -> Result<(), AstNodeError> {
        for op_type in UnaryOpType::all_variants() {
            let expr = unary_op(identifier("a"), op_type.clone())?;
            assert_eq!(emit(expr), format!("{}a", op_type.as_str()));
        }
        Ok(())
    }

    #[test]
    fn test_nested_unary_op_emit() -> Result<(), AstNodeError> {
        for op_type in UnaryOpType::all_variants() {
            let expr = unary_op(unary_op(identifier("a"), op_type.clone())?, op_type.clone())?;
            assert_eq!(
                emit(expr),
                format!("{}({}a)", op_type.as_str(), op_type.as_str())
            );
        }
        Ok(())
    }

    #[test]
    fn test_unary_op_binary_operand() -> Result<(), AstNodeError> {
        let result = unary_op(
            bin_op(identifier("a"), identifier("b"), BinOpType::Add)?,
            UnaryOpType::Negate,
        )?;

        assert_eq!(emit(result), "-(a + b)");

        Ok(())
    }

    #[test]
    fn test_unary_op_invalid_operand() {
        let result = unary_op(literal_string("a"), UnaryOpType::Negate);
        assert!(result.is_err());
    }

    #[test]
    fn test_unary_op_equality() -> Result<(), AstNodeError> {
        let unary1 = unary_op(identifier("a"), UnaryOpType::Negate)?;
        let unary2 = unary_op(identifier("a"), UnaryOpType::Negate)?;
        assert_eq!(unary1, unary2);

        let unary3 = unary_op(identifier("b"), UnaryOpType::Negate)?;
        assert_ne!(unary1, unary3);
        Ok(())
    }
}
