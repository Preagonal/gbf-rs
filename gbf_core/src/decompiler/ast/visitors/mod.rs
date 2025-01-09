#![deny(missing_docs)]

use super::{
    assignable::AssignableKind, bin_op::BinaryOperationNode, expr::ExprKind,
    identifier::IdentifierNode, literal::LiteralNode, member_access::MemberAccessNode,
    meta::MetaNode, statement::StatementNode, unary_op::UnaryOperationNode, AstKind,
};

/// Represents a visitor for the AST.
pub mod emit_context;
/// An emitter for the AST.
pub mod emitter;

/// Represents a visitor for the AST.
pub trait AstVisitor {
    /// Visits an AstNode
    fn visit_node(&mut self, node: &AstKind);
    /// Visits a statement node.
    fn visit_statement(&mut self, node: &StatementNode);
    /// Visits an expression node.
    fn visit_expr(&mut self, node: &ExprKind);
    /// Visits an assignable expression node.
    fn visit_assignable_expr(&mut self, node: &AssignableKind);
    /// Visits a binary operation node.
    fn visit_bin_op(&mut self, node: &BinaryOperationNode);
    /// Visits a unary operation node.
    fn visit_unary_op(&mut self, node: &UnaryOperationNode);
    /// Visits an identifier node.
    fn visit_identifier(&mut self, node: &IdentifierNode);
    /// Visits a literal node.
    fn visit_literal(&mut self, node: &LiteralNode);
    /// Visits a member access node.
    fn visit_member_access(&mut self, node: &MemberAccessNode);
    /// Visits a meta node.
    fn visit_meta(&mut self, node: &MetaNode);
    /// Visits a function call node.
    fn visit_function_call(&mut self, node: &super::func_call::FunctionCallNode);
    /// Visits a function node.
    fn visit_function(&mut self, node: &super::function::FunctionNode);
    /// Visits a return node.
    fn visit_return(&mut self, node: &super::ret::ReturnNode);
}
