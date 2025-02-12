#![deny(missing_docs)]

use super::{
    array::ArrayNode, array_access::ArrayAccessNode, assignment::AssignmentNode,
    bin_op::BinaryOperationNode, block::BlockNode, control_flow::ControlFlowNode, expr::ExprKind,
    func_call::FunctionCallNode, function::FunctionNode, identifier::IdentifierNode,
    literal::LiteralNode, member_access::MemberAccessNode, phi::PhiNode, ptr::P, ret::ReturnNode,
    statement::StatementKind, unary_op::UnaryOperationNode, AstKind,
};

/// Represents a visitor for the AST.
pub mod emit_context;
/// An emitter for the AST.
pub mod emitter;

/// Represents a visitor for the AST.
pub trait AstVisitor {
    /// The output type of the visitor.
    type Output;

    /// Visits an AST node.
    fn visit_node(&mut self, node: &AstKind) -> Self::Output;
    /// Visits a statement node.
    fn visit_statement(&mut self, node: &StatementKind) -> Self::Output;
    /// Visits an assignment node.
    fn visit_assignment(&mut self, node: &P<AssignmentNode>) -> Self::Output;
    /// Visits an expression node.
    fn visit_expr(&mut self, node: &ExprKind) -> Self::Output;
    /// Visits a binary operation node.
    fn visit_bin_op(&mut self, node: &P<BinaryOperationNode>) -> Self::Output;
    /// Visits a unary operation node.
    fn visit_unary_op(&mut self, node: &P<UnaryOperationNode>) -> Self::Output;
    /// Visits an identifier node.
    fn visit_identifier(&mut self, node: &P<IdentifierNode>) -> Self::Output;
    /// Visits a literal node.
    fn visit_literal(&mut self, node: &P<LiteralNode>) -> Self::Output;
    /// Visits a member access node.
    fn visit_member_access(&mut self, node: &P<MemberAccessNode>) -> Self::Output;
    /// Visits a function call node.
    fn visit_function_call(&mut self, node: &P<FunctionCallNode>) -> Self::Output;
    /// Visits an array node.
    fn visit_array(&mut self, node: &P<ArrayNode>) -> Self::Output;
    /// Visits an array access node.
    fn visit_array_access(&mut self, node: &P<ArrayAccessNode>) -> Self::Output;
    /// Visits a function node.
    fn visit_function(&mut self, node: &P<FunctionNode>) -> Self::Output;
    /// Visits a return node.
    fn visit_return(&mut self, node: &P<ReturnNode>) -> Self::Output;
    /// Visits a block node.
    fn visit_block(&mut self, node: &P<BlockNode>) -> Self::Output;
    /// Visits a control flow node.
    fn visit_control_flow(&mut self, node: &P<ControlFlowNode>) -> Self::Output;
    /// Visits a phi node.
    fn visit_phi(&mut self, node: &P<PhiNode>) -> Self::Output;
    /// Visits a `new` node.
    fn visit_new(&mut self, node: &P<crate::decompiler::ast::new::NewNode>) -> Self::Output;
    /// Visits a virtual branch node.
    fn visit_virtual_branch(
        &mut self,
        node: &P<crate::decompiler::ast::vbranch::VirtualBranchNode>,
    ) -> Self::Output;
}
