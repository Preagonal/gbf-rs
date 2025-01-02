#![deny(missing_docs)]

use serde::{Deserialize, Serialize};

use super::{
    bin_op::BinOpType,
    emit::{EmitContext, EmitError},
    expr::ExprNode,
    literal::LiteralNode,
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
    pub fn new(lhs: Box<ExprNode>, rhs: Box<ExprNode>) -> Result<Box<Self>, AstNodeError> {
        Self::validate_lhs(&lhs)?;
        Ok(Box::new(Self { lhs, rhs }))
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

        // Check if the RHS is a binary operation
        if let ExprNode::BinOp(bin_op_node) = self.rhs.as_ref() {
            // Check if the binary operation involves the LHS
            let lhs_in_rhs = bin_op_node.lhs.as_ref() == self.lhs.as_ref()
                || bin_op_node.rhs.as_ref() == self.lhs.as_ref();

            if lhs_in_rhs {
                match bin_op_node.op_type {
                    BinOpType::Add => {
                        // Handle += or ++
                        if let ExprNode::Literal(LiteralNode::Number(num)) =
                            bin_op_node.rhs.as_ref()
                        {
                            if *num == 1 {
                                return Ok(format!("{}++;", lhs_str));
                            }
                        }
                        return Ok(format!("{} += {};", lhs_str, bin_op_node.rhs.emit(ctx)?));
                    }
                    BinOpType::Sub => {
                        // Handle -= or --
                        if let ExprNode::Literal(LiteralNode::Number(num)) =
                            bin_op_node.rhs.as_ref()
                        {
                            if *num == 1 {
                                return Ok(format!("{}--;", lhs_str));
                            }
                        }
                        return Ok(format!("{} -= {};", lhs_str, bin_op_node.rhs.emit(ctx)?));
                    }
                    _ => {}
                }
            }
        }

        // At this point, we can change the emit context for the RHS as the expr root
        let ctx = &ctx.with_expr_root(true);

        // Default to simple assignment
        Ok(format!("{} = {};", lhs_str, self.rhs.emit(ctx)?))
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
    use crate::ast::{bin_op::BinaryOperationNode, expr::ExprNode, identifier::IdentifierNode};

    #[test]
    fn test_assignment_node_eq() {
        let lhs1 = Box::new(ExprNode::Identifier(IdentifierNode::new(
            "variable".to_string(),
        )));
        let rhs1 = Box::new(ExprNode::Identifier(IdentifierNode::new(
            "value".to_string(),
        )));
        let lhs2 = Box::new(ExprNode::Identifier(IdentifierNode::new(
            "variable".to_string(),
        )));
        let rhs2 = Box::new(ExprNode::Identifier(IdentifierNode::new(
            "value".to_string(),
        )));
        let lhs3 = Box::new(ExprNode::Identifier(IdentifierNode::new(
            "variable".to_string(),
        )));
        let rhs3 = Box::new(ExprNode::Identifier(IdentifierNode::new(
            "different_value".to_string(),
        )));

        let node1 = StatementNode::new(lhs1, rhs1).unwrap();
        let node2 = StatementNode::new(lhs2, rhs2).unwrap();
        let node3 = StatementNode::new(lhs3, rhs3).unwrap();

        assert_eq!(node1, node2);
        assert_ne!(node1, node3);
    }

    #[test]
    fn test_emit_postfix_increment() {
        let lhs = Box::new(ExprNode::Identifier(IdentifierNode::new("i".to_string())));
        let rhs = Box::new(ExprNode::BinOp(
            BinaryOperationNode::new(
                Box::new(ExprNode::Identifier(IdentifierNode::new("i".to_string()))),
                Box::new(ExprNode::Literal(LiteralNode::Number(1))),
                BinOpType::Add,
            )
            .unwrap(),
        ));

        let assignment = StatementNode { lhs, rhs };
        let ctx = EmitContext::default();
        let emitted = assignment.emit(&ctx).unwrap();

        assert_eq!(emitted, "i++;");
    }

    #[test]
    fn test_emit_postfix_decrement() {
        let lhs = Box::new(ExprNode::Identifier(IdentifierNode::new(
            "temp.i".to_string(),
        )));
        let rhs = Box::new(ExprNode::BinOp(
            BinaryOperationNode::new(
                Box::new(ExprNode::Identifier(IdentifierNode::new(
                    "temp.i".to_string(),
                ))),
                Box::new(ExprNode::Literal(LiteralNode::Number(1))),
                BinOpType::Sub,
            )
            .unwrap(),
        ));

        let assignment = StatementNode { lhs, rhs };
        let ctx = EmitContext::default();
        let emitted = assignment.emit(&ctx).unwrap();

        assert_eq!(emitted, "temp.i--;");
    }

    #[test]
    fn test_emit_compound_addition() {
        let lhs = Box::new(ExprNode::Identifier(IdentifierNode::new(
            "temp.i".to_string(),
        )));
        let rhs = Box::new(ExprNode::BinOp(
            BinaryOperationNode::new(
                Box::new(ExprNode::Identifier(IdentifierNode::new(
                    "temp.i".to_string(),
                ))),
                Box::new(ExprNode::Literal(LiteralNode::Number(5))),
                BinOpType::Add,
            )
            .unwrap(),
        ));

        let assignment = StatementNode { lhs, rhs };
        let ctx = EmitContext::default();
        let emitted = assignment.emit(&ctx).unwrap();

        assert_eq!(emitted, "temp.i += 5;");
    }

    #[test]
    fn test_emit_compound_subtraction() {
        let lhs = Box::new(ExprNode::Identifier(IdentifierNode::new(
            "temp.i".to_string(),
        )));
        let rhs = Box::new(ExprNode::BinOp(
            BinaryOperationNode::new(
                Box::new(ExprNode::Identifier(IdentifierNode::new(
                    "temp.i".to_string(),
                ))),
                Box::new(ExprNode::Literal(LiteralNode::Number(3))),
                BinOpType::Sub,
            )
            .unwrap(),
        ));

        let assignment = StatementNode { lhs, rhs };
        let ctx = EmitContext::default();
        let emitted = assignment.emit(&ctx).unwrap();

        assert_eq!(emitted, "temp.i -= 3;");
    }

    #[test]
    fn test_emit_simple_assignment() {
        let lhs = Box::new(ExprNode::Identifier(IdentifierNode::new(
            "temp.i".to_string(),
        )));
        let rhs = Box::new(ExprNode::Literal(LiteralNode::Number(42)));

        let assignment = StatementNode { lhs, rhs };
        let ctx = EmitContext::default();
        let emitted = assignment.emit(&ctx).unwrap();

        assert_eq!(emitted, "temp.i = 42;");
    }

    #[test]
    fn test_emit_unrelated_binop() {
        let lhs = Box::new(ExprNode::Identifier(IdentifierNode::new(
            "temp.i".to_string(),
        )));
        let rhs = Box::new(ExprNode::BinOp(
            BinaryOperationNode::new(
                Box::new(ExprNode::Identifier(IdentifierNode::new(
                    "other".to_string(),
                ))),
                Box::new(ExprNode::Literal(LiteralNode::Number(10))),
                BinOpType::Add,
            )
            .unwrap(),
        ));

        let assignment = StatementNode { lhs, rhs };
        let ctx = EmitContext::default();
        let emitted = assignment.emit(&ctx).unwrap();

        assert_eq!(emitted, "temp.i = other + 10;");
    }

    #[test]
    fn test_parentheses() {
        // test case like i = 1 + (2 + 3);
        let lhs = Box::new(ExprNode::Identifier(IdentifierNode::new("i".to_string())));
        let rhs = Box::new(ExprNode::BinOp(
            BinaryOperationNode::new(
                Box::new(ExprNode::Literal(LiteralNode::Number(1))),
                Box::new(ExprNode::BinOp(
                    BinaryOperationNode::new(
                        Box::new(ExprNode::Literal(LiteralNode::Number(2))),
                        Box::new(ExprNode::Literal(LiteralNode::Number(3))),
                        BinOpType::Add,
                    )
                    .unwrap(),
                )),
                BinOpType::Add,
            )
            .unwrap(),
        ));

        let assignment = StatementNode { lhs, rhs };
        let ctx = EmitContext::default();
        let emitted = assignment.emit(&ctx).unwrap();

        assert_eq!(emitted, "i = 1 + (2 + 3);");

        // test case like i = (1 + 2) + (3 + 4);
        let lhs = Box::new(ExprNode::Identifier(IdentifierNode::new("i".to_string())));
        let rhs = Box::new(ExprNode::BinOp(
            BinaryOperationNode::new(
                Box::new(ExprNode::BinOp(
                    BinaryOperationNode::new(
                        Box::new(ExprNode::Literal(LiteralNode::Number(1))),
                        Box::new(ExprNode::Literal(LiteralNode::Number(2))),
                        BinOpType::Add,
                    )
                    .unwrap(),
                )),
                Box::new(ExprNode::BinOp(
                    BinaryOperationNode::new(
                        Box::new(ExprNode::Literal(LiteralNode::Number(3))),
                        Box::new(ExprNode::Literal(LiteralNode::Number(4))),
                        BinOpType::Add,
                    )
                    .unwrap(),
                )),
                BinOpType::Add,
            )
            .unwrap(),
        ));

        let assignment = StatementNode { lhs, rhs };
        let ctx = EmitContext::default();
        let emitted = assignment.emit(&ctx).unwrap();

        assert_eq!(emitted, "i = (1 + 2) + (3 + 4);");

        // test case like i = 1 + (2 + (3 + 4));
        let lhs = Box::new(ExprNode::Identifier(IdentifierNode::new("i".to_string())));
        let rhs = Box::new(ExprNode::BinOp(
            BinaryOperationNode::new(
                Box::new(ExprNode::Literal(LiteralNode::Number(1))),
                Box::new(ExprNode::BinOp(
                    BinaryOperationNode::new(
                        Box::new(ExprNode::Literal(LiteralNode::Number(2))),
                        Box::new(ExprNode::BinOp(
                            BinaryOperationNode::new(
                                Box::new(ExprNode::Literal(LiteralNode::Number(3))),
                                Box::new(ExprNode::Literal(LiteralNode::Number(4))),
                                BinOpType::Add,
                            )
                            .unwrap(),
                        )),
                        BinOpType::Add,
                    )
                    .unwrap(),
                )),
                BinOpType::Add,
            )
            .unwrap(),
        ));

        let assignment = StatementNode { lhs, rhs };
        let ctx = EmitContext::default();
        let emitted = assignment.emit(&ctx).unwrap();

        assert_eq!(emitted, "i = 1 + (2 + (3 + 4));");
    }
}
