#![deny(missing_docs)]

use serde::{Deserialize, Serialize};

use super::{
    bin_op::BinaryOperationNode, func_call::FunctionCallNode, identifier::IdentifierNode,
    literal::LiteralNode, member_access::MemberAccessNode, unary_op::UnaryOperationNode,
    visitors::AstVisitor, AstNodeTrait,
};

/// Represents an expression node in the AST.
#[derive(Debug, Clone, Serialize, Deserialize, Eq)]
pub enum ExprNode {
    /// Represents a literal node in the AST.
    Literal(LiteralNode),
    /// Represents an assignable expression node in the AST.
    Assignable(AssignableExpr),
    /// Represents a binary operation node in the AST.
    BinOp(BinaryOperationNode),
    /// Represents a unary operation node in the AST.
    UnaryOp(UnaryOperationNode),
    /// Represents a function call node in the AST.
    FunctionCall(FunctionCallNode),
}

impl AstNodeTrait for ExprNode {
    fn accept(&self, visitor: &mut dyn AstVisitor) {
        visitor.visit_expr(self);
    }
}

/// Represents an assignable expression node in the AST.
#[derive(Debug, Clone, Serialize, Deserialize, Eq)]
pub enum AssignableExpr {
    /// Represents a member access node in the AST.
    MemberAccess(MemberAccessNode),
    /// Represents an identifier node in the AST.
    Identifier(IdentifierNode),
}

impl AstNodeTrait for AssignableExpr {
    fn accept(&self, visitor: &mut dyn AstVisitor) {
        visitor.visit_assignable_expr(self);
    }
}

// == Other implementations for literal ==

impl PartialEq for ExprNode {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ExprNode::Literal(l1), ExprNode::Literal(l2)) => l1 == l2,
            (ExprNode::Assignable(a1), ExprNode::Assignable(a2)) => a1 == a2,
            (ExprNode::BinOp(b1), ExprNode::BinOp(b2)) => b1 == b2,
            (ExprNode::UnaryOp(u1), ExprNode::UnaryOp(u2)) => u1 == u2,
            _ => false,
        }
    }
}

impl PartialEq for AssignableExpr {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (AssignableExpr::MemberAccess(m1), AssignableExpr::MemberAccess(m2)) => m1 == m2,
            (AssignableExpr::Identifier(i1), AssignableExpr::Identifier(i2)) => i1 == i2,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decompiler::ast::bin_op::BinOpType;
    use crate::decompiler::ast::unary_op::UnaryOpType;

    #[test]
    fn test_expr_node_eq() {
        let expr1 = ExprNode::Literal(LiteralNode::String("Hello, world!".to_string()));
        let expr2 = ExprNode::Literal(LiteralNode::String("Hello, world!".to_string()));
        let expr3 = ExprNode::Literal(LiteralNode::String("Goodbye, world!".to_string()));

        assert_eq!(expr1, expr2);
        assert_ne!(expr1, expr3);

        // test member access
        let expr1 = ExprNode::Assignable(AssignableExpr::MemberAccess(
            MemberAccessNode::new(
                Box::new(AssignableExpr::Identifier(IdentifierNode::new(
                    "object".to_string(),
                ))),
                Box::new(AssignableExpr::Identifier(IdentifierNode::new(
                    "field".to_string(),
                ))),
            )
            .unwrap(),
        ));
        let expr2 = ExprNode::Assignable(AssignableExpr::MemberAccess(
            MemberAccessNode::new(
                Box::new(AssignableExpr::Identifier(IdentifierNode::new(
                    "object".to_string(),
                ))),
                Box::new(AssignableExpr::Identifier(IdentifierNode::new(
                    "field".to_string(),
                ))),
            )
            .unwrap(),
        ));
        let expr3 = ExprNode::Assignable(AssignableExpr::MemberAccess(
            MemberAccessNode::new(
                Box::new(AssignableExpr::Identifier(IdentifierNode::new(
                    "object".to_string(),
                ))),
                Box::new(AssignableExpr::Identifier(IdentifierNode::new(
                    "field3".to_string(),
                ))),
            )
            .unwrap(),
        ));
        assert_eq!(expr1, expr2);
        assert_ne!(expr1, expr3);

        // test identifier
        let expr1 = ExprNode::Assignable(AssignableExpr::Identifier(IdentifierNode::new(
            "object".to_string(),
        )));
        let expr2 = ExprNode::Assignable(AssignableExpr::Identifier(IdentifierNode::new(
            "object".to_string(),
        )));

        assert_eq!(expr1, expr2);
        assert_ne!(expr1, expr3);

        // test binary operation
        let expr1 = ExprNode::BinOp(
            BinaryOperationNode::new(
                Box::new(ExprNode::Assignable(AssignableExpr::Identifier(
                    IdentifierNode::new("x".to_string()),
                ))),
                Box::new(ExprNode::Literal(LiteralNode::Number(1))),
                BinOpType::Add,
            )
            .unwrap(),
        );
        let expr2 = ExprNode::BinOp(
            BinaryOperationNode::new(
                ExprNode::Assignable(AssignableExpr::Identifier(IdentifierNode::new(
                    "x".to_string(),
                )))
                .clone_box(),
                ExprNode::Literal(LiteralNode::Number(1)).clone_box(),
                BinOpType::Add,
            )
            .unwrap(),
        );
        let expr3 = ExprNode::BinOp(
            BinaryOperationNode::new(
                ExprNode::Assignable(AssignableExpr::Identifier(IdentifierNode::new(
                    "x".to_string(),
                )))
                .clone_box(),
                ExprNode::Literal(LiteralNode::Number(2)).clone_box(),
                BinOpType::Add,
            )
            .unwrap(),
        );

        assert_eq!(expr1, expr2);
        assert_ne!(expr1, expr3);

        // test unary operation
        let expr1 = ExprNode::UnaryOp(
            UnaryOperationNode::new(
                ExprNode::Assignable(AssignableExpr::Identifier(IdentifierNode::new(
                    "x".to_string(),
                )))
                .clone_box(),
                UnaryOpType::Negate,
            )
            .unwrap(),
        );
        let expr2 = ExprNode::UnaryOp(
            UnaryOperationNode::new(
                ExprNode::Assignable(AssignableExpr::Identifier(IdentifierNode::new(
                    "x".to_string(),
                )))
                .clone_box(),
                UnaryOpType::Negate,
            )
            .unwrap(),
        );
        let expr3 = ExprNode::UnaryOp(
            UnaryOperationNode::new(
                ExprNode::Assignable(AssignableExpr::Identifier(IdentifierNode::new(
                    "x".to_string(),
                )))
                .clone_box(),
                UnaryOpType::LogicalNot,
            )
            .unwrap(),
        );

        assert_eq!(expr1, expr2);
        assert_ne!(expr1, expr3);
    }

    #[test]
    fn test_expr_node_ne_different_types() {
        let expr1 = ExprNode::Literal(LiteralNode::String("Hello, world!".to_string()));
        let expr2 = ExprNode::Assignable(AssignableExpr::MemberAccess(
            MemberAccessNode::new(
                Box::new(AssignableExpr::Identifier(IdentifierNode::new(
                    "object".to_string(),
                ))),
                Box::new(AssignableExpr::Identifier(IdentifierNode::new(
                    "field".to_string(),
                ))),
            )
            .unwrap(),
        ));

        assert_ne!(expr1, expr2);

        let expr1 = ExprNode::Literal(LiteralNode::String("Hello, world!".to_string()));
        let expr2 = ExprNode::Assignable(AssignableExpr::Identifier(IdentifierNode::new(
            "object".to_string(),
        )));

        assert_ne!(expr1, expr2);

        let expr1 = ExprNode::Assignable(AssignableExpr::Identifier(IdentifierNode::new(
            "object".to_string(),
        )));
        let expr2 = ExprNode::Literal(LiteralNode::String("object".to_string()));

        assert_ne!(expr1, expr2);
    }
}
