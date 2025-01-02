#![deny(missing_docs)]

use std::fmt::Display;

use emit::{EmitContext, EmitError};
use expr::ExprNode;
use meta::MetaNode;
use serde::{Deserialize, Serialize};
use statement::StatementNode;
use thiserror::Error;

/// Contains the emitting context for the AST.
pub mod emit;
/// Contains the specifications for any AstNodes that are expressions
pub mod expr;
/// Contains the specifications for any AstNodes that are identifiers.
pub mod identifier;
/// Contains the specifications for any AstNodes that are literals.
pub mod literal;
/// Contains the specifications for any AstNodes that are member accesses.
pub mod member_access;
/// Contains the specifications for any AstNodes that are metadata.
pub mod meta;
/// Contains the specifications for any AstNodes that are statements
pub mod statement;

/// Represents an error that occurred while converting an AST node.
#[derive(Debug, Error)]
pub enum AstNodeError {
    /// Invalid conversion from AstNode to another type.
    #[error("Expected {0}, found {1}")]
    InvalidConversion(String, String),

    /// Invalid operand for an AST node.
    #[error("Invalid {0} operand for {1}. Expected types in {2:?}, found {3}")]
    InvalidOperand(String, String, Vec<String>, String),
}

/// Trait for all AST nodes.
pub trait AstNodeTrait {
    /// Emits the AST node as a string.
    ///
    /// # Arguments
    /// - `ctx` - The context in which to emit the AST node.
    ///
    /// # Returns
    /// The AST node as a string.
    fn emit(&self, ctx: &EmitContext) -> Result<String, EmitError>;
}

/// Represents an AST node.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AstNode {
    /// Represents a statement node in the AST, such as `variable = value;`.
    Statement(StatementNode),
    // BinaryOperation(BinaryOperationNode),
    // UnaryOperation(UnaryOperationNode),
    // ControlFlow(ControlFlowNode),
    /// Represents a literal node in the AST.
    Expression(ExprNode),
    // Identifier(IdentifierNode),
    // MemberAccess(MemberAccessNode),
    // FunctionCall(FunctionCallNode),
    // Allocation(AllocationNode),
    // Array(ArrayNode),
    // Return(ReturnNode),
    // Phi(PhiNode),
    /// Represents a metadata node in the AST.
    Meta(MetaNode), // Covers comments or annotations
}

impl AstNodeTrait for AstNode {
    fn emit(&self, ctx: &EmitContext) -> Result<String, EmitError> {
        match self {
            AstNode::Expression(expr) => expr.emit(ctx),
            AstNode::Meta(meta) => meta.emit(ctx),
            AstNode::Statement(stmt) => stmt.emit(ctx),
        }
    }
}

// == Other Implementations for AstNode ==
impl Display for AstNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // TODO: Implement EmitContext for AstNode
        write!(f, "{}", self.emit(&EmitContext::default()).unwrap())
    }
}

#[cfg(test)]
mod tests {
    use literal::LiteralNode;

    use super::*;

    fn create_number(x: i32) -> ExprNode {
        ExprNode::Literal(LiteralNode::Number(x))
    }

    fn create_identifier(x: &str) -> ExprNode {
        ExprNode::Identifier(x.to_string())
    }

    fn create_member_access(lhs: ExprNode, rhs: ExprNode) -> ExprNode {
        ExprNode::MemberAccess(member_access::MemberAccessNode::new(lhs, rhs).unwrap())
    }

    fn create_comment(comment: &str, node: AstNode) -> AstNode {
        AstNode::Meta(meta::MetaNode::new(
            node,
            Some(comment.to_string()),
            None,
            Default::default(),
        ))
    }

    #[test]
    fn test_emit() {
        let expr = create_number(42);
        let ast_node = AstNode::Expression(expr);
        assert_eq!(ast_node.emit(&EmitContext::default()).unwrap(), "42");

        let expr = create_identifier("variable");
        let ast_node = AstNode::Expression(expr);
        assert_eq!(ast_node.emit(&EmitContext::default()).unwrap(), "variable");

        let lhs = create_identifier("variable");
        let rhs = create_number(42);
        let stmt = StatementNode::new(lhs, rhs).unwrap();
        let ast_node = AstNode::Statement(stmt);
        assert_eq!(
            ast_node.emit(&EmitContext::default()).unwrap(),
            "variable = 42;"
        );

        // add meta comment
        let node = create_comment("Hello", ast_node);
        assert_eq!(
            node.emit(&EmitContext::default()).unwrap(),
            "// Hello\nvariable = 42;"
        );
    }

    #[test]
    fn test_stack() {
        // Create a stack of AST nodes
        let mut ast_stack = vec![
            AstNode::Expression(create_member_access(
                create_identifier("temp"),
                create_identifier("foo"),
            )),
            AstNode::Expression(create_number(42)),
        ];

        // assume we currently have a statement instruction. pop two values
        let stack_values_to_pop = 2;
        assert_eq!(stack_values_to_pop, 2);

        // pop the last two values from the stack
        let val1 = ast_stack.pop().unwrap();
        let val2 = ast_stack.pop().unwrap();

        // create expression node from the popped values and feed them into statement node
        let expr1 = match val1 {
            AstNode::Expression(expr) => expr,
            _ => unreachable!("Expected an expression node"),
        };

        let expr2 = match val2 {
            AstNode::Expression(expr) => expr,
            _ => unreachable!("Expected an expression node"),
        };

        let stmt = StatementNode::new(expr2, expr1).unwrap();
        let ast_node = AstNode::Statement(stmt);
        assert_eq!(
            ast_node.emit(&EmitContext::default()).unwrap(),
            "temp.foo = 42;"
        );
    }

    #[test]
    fn test_stack_failure() {
        // this time, add meta nodes to the stack and try to pop them
        let mut ast_stack = vec![
            AstNode::Meta(meta::MetaNode::new(
                AstNode::Expression(create_member_access(
                    create_identifier("temp"),
                    create_identifier("foo"),
                )),
                Some("This is a comment".to_string()),
                None,
                Default::default(),
            )),
            AstNode::Expression(create_number(42)),
        ];

        // pop the last two values from the stack
        let val1 = ast_stack.pop().unwrap();
        let val2 = ast_stack.pop().unwrap();

        // This is fine; we know the last value is an expression node
        let expr1 = match val1 {
            AstNode::Expression(expr) => expr,
            _ => unreachable!("Expected an expression node"),
        };

        // Assert that expr1 is a literal
        assert!(
            matches!(expr1, ExprNode::Literal(_)),
            "Expected a literal node"
        );

        // The second node is a meta node, not an expression node
        assert!(matches!(val2, AstNode::Meta(_)), "Expected a meta node");
    }

    #[test]
    fn test_display() {
        let expr = create_number(42);
        let ast_node = AstNode::Expression(expr);
        assert_eq!(format!("{}", ast_node), "42");

        let expr = create_identifier("variable");
        let ast_node = AstNode::Expression(expr);
        assert_eq!(format!("{}", ast_node), "variable");

        let lhs = create_identifier("variable");
        let rhs = create_number(42);
        let stmt = StatementNode::new(lhs, rhs).unwrap();
        let ast_node = AstNode::Statement(stmt);
        assert_eq!(format!("{}", ast_node), "variable = 42;");

        // add meta comment
        let node = create_comment("Hello", ast_node);
        assert_eq!(format!("{}", node), "// Hello\nvariable = 42;");
    }
}
