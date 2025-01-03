#![deny(missing_docs)]

use super::AstVisitor;

/// An emitter for the AST.
pub struct Gs2Emitter {
    /// The output of the emitter.
    output: String,
}

impl AstVisitor for Gs2Emitter {
    fn visit_statement(&mut self, node: &crate::ast::statement::StatementNode) {
        todo!()
    }

    fn visit_expr(&mut self, node: &crate::ast::expr::ExprNode) {
        todo!()
    }

    fn visit_bin_op(&mut self, node: &crate::ast::bin_op::BinaryOperationNode) {
        todo!()
    }

    fn visit_unary_op(&mut self, node: &crate::ast::unary_op::UnaryOperationNode) {
        todo!()
    }

    fn visit_identifier(&mut self, node: &crate::ast::identifier::IdentifierNode) {
        todo!()
    }

    fn visit_literal(&mut self, node: &crate::ast::literal::LiteralNode) {
        todo!()
    }

    fn visit_member_access(&mut self, node: &crate::ast::member_access::MemberAccessNode) {
        todo!()
    }

    fn visit_meta(&mut self, node: &crate::ast::meta::MetaNode) {
        todo!()
    }
}
