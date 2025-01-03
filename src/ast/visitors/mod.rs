#![deny(missing_docs)]

use super::{
    bin_op::BinaryOperationNode, expr::ExprNode, identifier::IdentifierNode, literal::LiteralNode,
    member_access::MemberAccessNode, meta::MetaNode, statement::StatementNode,
    unary_op::UnaryOperationNode,
};

/// An emitter for the AST.
pub mod emitter;

/// Represents a visitor for the AST.
pub trait AstVisitor {
    /// Visits a statement node.
    fn visit_statement(&mut self, node: &StatementNode);
    /// Visits an expression node.
    fn visit_expr(&mut self, node: &ExprNode);
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
}
