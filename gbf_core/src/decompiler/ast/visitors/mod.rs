#![deny(missing_docs)]

use super::{
    assignable::AssignableKind, assignment::AssignmentNode, bin_op::BinaryOperationNode,
    expr::ExprKind, identifier::IdentifierNode, literal::LiteralNode,
    member_access::MemberAccessNode, meta::MetaNode, unary_op::UnaryOperationNode, AstKind,
};

/// Represents a visitor for the AST.
pub mod emit_context;
/// An emitter for the AST.
pub mod emitter;

/// Represents a visitor for the AST.
pub trait AstVisitor {
    /// The output type of the visitor.
    type Output;

    /// Visits an AstNode
    fn visit_node(&mut self, node: &AstKind) -> Self::Output;
    /// Visits a statement node.
    fn visit_statement(&mut self, node: &super::statement::StatementKind) -> Self::Output;
    /// Visits a assignment node.
    fn visit_assignment(&mut self, node: &AssignmentNode) -> Self::Output;
    /// Visits an expression node.
    fn visit_expr(&mut self, node: &ExprKind) -> Self::Output;
    /// Visits an assignable expression node.
    fn visit_assignable_expr(&mut self, node: &AssignableKind) -> Self::Output;
    /// Visits a binary operation node.
    fn visit_bin_op(&mut self, node: &BinaryOperationNode) -> Self::Output;
    /// Visits a unary operation node.
    fn visit_unary_op(&mut self, node: &UnaryOperationNode) -> Self::Output;
    /// Visits an identifier node.
    fn visit_identifier(&mut self, node: &IdentifierNode) -> Self::Output;
    /// Visits a literal node.
    fn visit_literal(&mut self, node: &LiteralNode) -> Self::Output;
    /// Visits a member access node.
    fn visit_member_access(&mut self, node: &MemberAccessNode) -> Self::Output;
    /// Visits a meta node.
    fn visit_meta(&mut self, node: &MetaNode) -> Self::Output;
    /// Visits a function call node.
    fn visit_function_call(&mut self, node: &super::func_call::FunctionCallNode) -> Self::Output;
    /// Visits an array node.
    fn visit_array(&mut self, node: &super::array::ArrayNode) -> Self::Output;
    /// Visits an array access node.
    fn visit_array_access(&mut self, node: &super::array_access::ArrayAccessNode) -> Self::Output;
    /// Visits a function node.
    fn visit_function(&mut self, node: &super::function::FunctionNode) -> Self::Output;
    /// Visits a return node.
    fn visit_return(&mut self, node: &super::ret::ReturnNode) -> Self::Output;
    /// Visits a block node.
    fn visit_block(&mut self, node: &super::block::BlockNode) -> Self::Output;
    /// Visits a control flow node
    fn visit_control_flow(&mut self, node: &super::control_flow::ControlFlowNode) -> Self::Output;
}
