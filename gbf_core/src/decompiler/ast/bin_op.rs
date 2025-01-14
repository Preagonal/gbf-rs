#![deny(missing_docs)]

use gbf_macros::AstNodeTransform;
use serde::{Deserialize, Serialize};

use super::AstKind;
use super::{expr::ExprKind, AstNodeError};
use crate::decompiler::ast::AstVisitable;
use crate::define_ast_enum_type;

define_ast_enum_type! {
    BinOpType {
        Add => "+",
        Sub => "-",
        Mul => "*",
        Div => "/",
        Mod => "%",
        And => "&",
        Or => "|",
        Xor => "xor",
        LogicalAnd => "&&",
        LogicalOr => "||",
        Equal => "==",
        NotEqual => "!=",
        Greater => ">",
        Less => "<",
        GreaterOrEqual => ">=",
        LessOrEqual => "<=",
        ShiftLeft => "<<",
        ShiftRight => ">>",
        In => "in",
        Join => "@",
    }
}

/// Represents a binary operation node in the AST, such as `a + b`.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, AstNodeTransform)]
#[convert_to(ExprKind::BinOp, AstKind::Expression)]
pub struct BinaryOperationNode {
    /// The left-hand side of the binary operation.
    pub lhs: Box<ExprKind>,
    /// The right-hand side of the binary operation.
    pub rhs: Box<ExprKind>,
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
        lhs: Box<ExprKind>,
        rhs: Box<ExprKind>,
        op_type: BinOpType,
    ) -> Result<Self, AstNodeError> {
        Ok(Self { lhs, rhs, op_type })
    }
}

// == Other implementations for binary operations ==
impl PartialEq for BinaryOperationNode {
    fn eq(&self, other: &Self) -> bool {
        self.lhs == other.lhs && self.rhs == other.rhs && self.op_type == other.op_type
    }
}

impl AstVisitable for BinaryOperationNode {
    fn accept(&self, visitor: &mut dyn super::visitors::AstVisitor) {
        visitor.visit_bin_op(self);
    }
}

#[cfg(test)]
mod tests {
    use crate::decompiler::ast::{emit, new_bin_op, new_id};

    use super::*;

    #[test]
    fn test_bin_op_emit() -> Result<(), AstNodeError> {
        for op_type in BinOpType::all_variants() {
            let expr = new_bin_op(new_id("a"), new_id("b"), op_type.clone())?;
            assert_eq!(emit(expr), format!("a {} b", op_type.as_str()));
        }
        Ok(())
    }

    #[test]
    fn test_nested_bin_op_emit() -> Result<(), AstNodeError> {
        let expr = new_bin_op(
            new_bin_op(new_id("a"), new_id("b"), BinOpType::Add)?,
            new_id("c"),
            BinOpType::Mul,
        )?;
        assert_eq!(emit(expr), "(a + b) * c");
        Ok(())
    }

    #[test]
    fn test_bin_op_eq() -> Result<(), AstNodeError> {
        let a = new_bin_op(new_id("a"), new_id("b"), BinOpType::Add)?;
        let b = new_bin_op(new_id("a"), new_id("b"), BinOpType::Add)?;
        let c = new_bin_op(new_id("a"), new_id("b"), BinOpType::Sub)?;
        let d = new_bin_op(new_id("a"), new_id("c"), BinOpType::Add)?;

        assert_eq!(a, b);
        assert_ne!(a, c);
        assert_ne!(a, d);
        Ok(())
    }
}
