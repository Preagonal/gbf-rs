#![deny(missing_docs)]

use serde::{Deserialize, Serialize};

use super::{
    bin_op::BinaryOperationNode, identifier::IdentifierNode, literal::LiteralNode,
    member_access::MemberAccessNode, unary_op::UnaryOperationNode, visitors::AstVisitor,
    AstNodeTrait,
};

/// Represents an expression node in the AST.
#[derive(Debug, Clone, Serialize, Deserialize, Eq)]
pub enum ExprNode {
    /// Represents a literal node in the AST.
    Literal(LiteralNode),
    /// Represents a member access node in the AST.
    MemberAccess(Box<MemberAccessNode>),
    /// Represents an identifier node in the AST.
    Identifier(Box<IdentifierNode>),
    /// Represents a binary operation node in the AST.
    BinOp(Box<BinaryOperationNode>),
    /// Represents a unary operation node in the AST.
    UnaryOp(Box<UnaryOperationNode>),
}

impl AstNodeTrait for ExprNode {
    fn accept(&self, visitor: &mut dyn AstVisitor) {
        visitor.visit_expr(self);
    }
}

// == Other implementations for literal ==

impl PartialEq for ExprNode {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ExprNode::Literal(l1), ExprNode::Literal(l2)) => l1 == l2,
            (ExprNode::MemberAccess(m1), ExprNode::MemberAccess(m2)) => m1 == m2,
            (ExprNode::Identifier(id1), ExprNode::Identifier(id2)) => id1 == id2,
            (ExprNode::BinOp(b1), ExprNode::BinOp(b2)) => b1 == b2,
            (ExprNode::UnaryOp(u1), ExprNode::UnaryOp(u2)) => u1 == u2,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expr_node_eq() {
        let expr1 = ExprNode::Literal(LiteralNode::String("Hello, world!".to_string()));
        let expr2 = ExprNode::Literal(LiteralNode::String("Hello, world!".to_string()));
        let expr3 = ExprNode::Literal(LiteralNode::String("Goodbye, world!".to_string()));

        assert_eq!(expr1, expr2);
        assert_ne!(expr1, expr3);

        // test member access
        let expr1 = ExprNode::MemberAccess(
            MemberAccessNode::new(
                ExprNode::Identifier(IdentifierNode::new("object".to_string())).clone_box(),
                ExprNode::Identifier(IdentifierNode::new("field".to_string())).clone_box(),
            )
            .unwrap(),
        );
        let expr2 = ExprNode::MemberAccess(
            MemberAccessNode::new(
                ExprNode::Identifier(IdentifierNode::new("object".to_string())).clone_box(),
                ExprNode::Identifier(IdentifierNode::new("field".to_string())).clone_box(),
            )
            .unwrap(),
        );
        let expr3 = ExprNode::MemberAccess(
            MemberAccessNode::new(
                ExprNode::Identifier(IdentifierNode::new("object".to_string())).clone_box(),
                ExprNode::Identifier(IdentifierNode::new("field2".to_string())).clone_box(),
            )
            .unwrap(),
        );
        assert_eq!(expr1, expr2);
        assert_ne!(expr1, expr3);

        // test identifier
        let expr1 = ExprNode::Identifier(IdentifierNode::new("object".to_string()).clone_box());
        let expr2 = ExprNode::Identifier(IdentifierNode::new("object".to_string()).clone_box());

        assert_eq!(expr1, expr2);
        assert_ne!(expr1, expr3);

        // test binary operation
        let expr1 = ExprNode::BinOp(
            BinaryOperationNode::new(
                ExprNode::Identifier(IdentifierNode::new("x".to_string())).clone_box(),
                ExprNode::Literal(LiteralNode::Number(1)).clone_box(),
                crate::ast::bin_op::BinOpType::Add,
            )
            .unwrap(),
        );
        let expr2 = ExprNode::BinOp(
            BinaryOperationNode::new(
                ExprNode::Identifier(IdentifierNode::new("x".to_string())).clone_box(),
                ExprNode::Literal(LiteralNode::Number(1)).clone_box(),
                crate::ast::bin_op::BinOpType::Add,
            )
            .unwrap(),
        );
        let expr3 = ExprNode::BinOp(
            BinaryOperationNode::new(
                ExprNode::Identifier(IdentifierNode::new("x".to_string())).clone_box(),
                ExprNode::Literal(LiteralNode::Number(2)).clone_box(),
                crate::ast::bin_op::BinOpType::Add,
            )
            .unwrap(),
        );

        assert_eq!(expr1, expr2);
        assert_ne!(expr1, expr3);

        // test unary operation
        let expr1 = ExprNode::UnaryOp(
            UnaryOperationNode::new(
                ExprNode::Identifier(IdentifierNode::new("x".to_string())).clone_box(),
                crate::ast::unary_op::UnaryOpType::Negate,
            )
            .unwrap(),
        );
        let expr2 = ExprNode::UnaryOp(
            UnaryOperationNode::new(
                ExprNode::Identifier(IdentifierNode::new("x".to_string())).clone_box(),
                crate::ast::unary_op::UnaryOpType::Negate,
            )
            .unwrap(),
        );
        let expr3 = ExprNode::UnaryOp(
            UnaryOperationNode::new(
                ExprNode::Identifier(IdentifierNode::new("x".to_string())).clone_box(),
                crate::ast::unary_op::UnaryOpType::LogicalNot,
            )
            .unwrap(),
        );

        assert_eq!(expr1, expr2);
        assert_ne!(expr1, expr3);
    }

    #[test]
    fn test_expr_node_ne_different_types() {
        let expr1 = ExprNode::Literal(LiteralNode::String("Hello, world!".to_string()));
        let expr2 = ExprNode::MemberAccess(
            MemberAccessNode::new(
                ExprNode::Identifier(IdentifierNode::new("object".to_string())).clone_box(),
                ExprNode::Identifier(IdentifierNode::new("field".to_string())).clone_box(),
            )
            .unwrap(),
        );

        assert_ne!(expr1, expr2);

        let expr1 = ExprNode::Literal(LiteralNode::String("Hello, world!".to_string()));
        let expr2 = ExprNode::Identifier(IdentifierNode::new("object".to_string()).clone_box());

        assert_ne!(expr1, expr2);

        let expr1 = ExprNode::Identifier(IdentifierNode::new("object".to_string()).clone_box());
        let expr2 = ExprNode::Literal(LiteralNode::String("object".to_string()));

        assert_ne!(expr1, expr2);
    }
}
