#![deny(missing_docs)]

use serde::{Deserialize, Serialize};

use super::{
    bin_op::BinaryOperationNode,
    emit::{EmitContext, EmitError},
    identifier::IdentifierNode,
    literal::LiteralNode,
    member_access::MemberAccessNode,
    unary_op::UnaryOperationNode,
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
    fn emit(&self, ctx: &EmitContext) -> Result<String, EmitError> {
        let expr_root = ctx.expr_root;
        let ctx = &ctx.with_expr_root(false);
        Ok(match self {
            ExprNode::Literal(literal) => literal.emit(ctx)?,
            ExprNode::MemberAccess(mem) => mem.emit(ctx)?,
            ExprNode::Identifier(s) => s.emit(ctx)?,
            ExprNode::BinOp(bin_op) => {
                let emitted = bin_op.emit(ctx)?;
                if !expr_root {
                    format!("({})", emitted)
                } else {
                    emitted
                }
            }
            ExprNode::UnaryOp(unary_op) => {
                let emitted = unary_op.emit(ctx)?;
                if !expr_root {
                    format!("({})", emitted)
                } else {
                    emitted
                }
            }
        })
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

    #[test]
    fn test_expr_node_emit() {
        let expr = ExprNode::Literal(LiteralNode::String("Hello, world!".to_string()));
        let ctx = EmitContext::default();
        assert_eq!(expr.emit(&ctx).unwrap(), "\"Hello, world!\"");

        let expr = ExprNode::MemberAccess(
            MemberAccessNode::new(
                ExprNode::Identifier(IdentifierNode::new("temp".to_string())).clone_box(),
                ExprNode::Identifier(IdentifierNode::new("field".to_string())).clone_box(),
            )
            .unwrap(),
        );
        assert_eq!(expr.emit(&ctx).unwrap(), "temp.field");

        let expr = ExprNode::Identifier(IdentifierNode::new("temp".to_string()).clone_box());
        assert_eq!(expr.emit(&ctx).unwrap(), "temp");
    }
}
