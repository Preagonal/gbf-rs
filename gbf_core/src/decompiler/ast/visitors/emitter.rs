#![deny(missing_docs)]

use super::{
    emit_context::{EmitContext, EmitVerbosity, IndentStyle},
    AstVisitor,
};
use crate::decompiler::ast::{
    assignable::AssignableKind, control_flow::ControlFlowType, expr::ExprKind,
};
use crate::decompiler::ast::{assignment::AssignmentNode, statement::StatementKind};
use crate::decompiler::ast::{
    bin_op::{BinOpType, BinaryOperationNode},
    func_call::FunctionCallNode,
};
use crate::decompiler::ast::{block::BlockNode, meta::MetaNode};
use crate::decompiler::ast::{control_flow::ControlFlowNode, unary_op::UnaryOperationNode};
use crate::decompiler::ast::{function::FunctionNode, literal::LiteralNode};
use crate::decompiler::ast::{member_access::MemberAccessNode, ret::ReturnNode};
use crate::decompiler::ast::{AstKind, AstVisitable};
use crate::{decompiler::ast::identifier::IdentifierNode, utils::escape_string};

/// An emitter for the AST.
///
/// This emitter now builds up and returns a `String` for every AST node rather
/// than writing to a shared output buffer.
pub struct Gs2Emitter {
    /// The context of the emitter.
    context: EmitContext,
}

impl Gs2Emitter {
    /// Creates a new `Gs2Emitter` with the given `context`.
    pub fn new(context: EmitContext) -> Self {
        Self { context }
    }
}

impl AstVisitor for Gs2Emitter {
    type Output = String;

    /// Visits an AST node.
    fn visit_node(&mut self, node: &AstKind) -> String {
        match node {
            AstKind::Expression(expr) => expr.accept(self),
            AstKind::Meta(meta) => meta.accept(self),
            AstKind::Statement(stmt) => stmt.accept(self),
            AstKind::Function(func) => func.accept(self),
            AstKind::Block(block) => block.accept(self),
            AstKind::ControlFlow(control_flow) => control_flow.accept(self),
        }
    }

    /// Visits a statement node.
    fn visit_statement(&mut self, node: &StatementKind) -> String {
        self.context = self.context.with_expr_root(true);
        let stmt_str = match node {
            StatementKind::Assignment(assignment) => assignment.accept(self),
            StatementKind::Return(ret) => ret.accept(self),
        };
        format!("{};", stmt_str)
    }

    /// Visits an assignment node.
    fn visit_assignment(&mut self, stmt_node: &AssignmentNode) -> String {
        // Step 1: Visit and emit the LHS.
        let lhs_str = stmt_node.lhs.accept(self);

        // Step 2: Check for binary operations that use the LHS.
        if let ExprKind::BinOp(bin_op_node) = stmt_node.rhs.clone() {
            let lhs_in_rhs = bin_op_node.lhs == ExprKind::Assignable(stmt_node.lhs.clone());
            if lhs_in_rhs {
                match bin_op_node.op_type {
                    BinOpType::Add => {
                        if let ExprKind::Literal(lit) = bin_op_node.rhs.clone() {
                            if let LiteralNode::Number(num) = lit.as_ref() {
                                if *num == 1 {
                                    return format!("{}++", lhs_str);
                                } else {
                                    let rhs_str = bin_op_node.rhs.accept(self);
                                    return format!("{} += {}", lhs_str, rhs_str);
                                }
                            }
                        }
                    }
                    BinOpType::Sub => {
                        if let ExprKind::Literal(lit) = bin_op_node.rhs.clone() {
                            if let LiteralNode::Number(num) = lit.as_ref() {
                                if *num == 1 {
                                    return format!("{}--", lhs_str);
                                } else {
                                    let rhs_str = bin_op_node.rhs.accept(self);
                                    return format!("{} -= {}", lhs_str, rhs_str);
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        // Step 3: Default assignment.
        let prev_context = self.context;
        self.context = self.context.with_expr_root(true);
        let rhs_str = stmt_node.rhs.accept(self);
        self.context = prev_context;
        format!("{} = {}", lhs_str, rhs_str)
    }

    /// Visits an expression node.
    fn visit_expr(&mut self, node: &ExprKind) -> String {
        match node {
            ExprKind::Literal(literal) => literal.accept(self),
            ExprKind::Assignable(assignable) => self.visit_assignable_expr(assignable),
            ExprKind::BinOp(bin_op) => bin_op.accept(self),
            ExprKind::UnaryOp(unary_op) => unary_op.accept(self),
            ExprKind::FunctionCall(func_call) => func_call.accept(self),
            ExprKind::Array(array) => array.accept(self),
        }
    }

    /// Visits an assignable expression node.
    fn visit_assignable_expr(&mut self, node: &AssignableKind) -> String {
        match node {
            AssignableKind::MemberAccess(member_access) => {
                let mut s = String::new();
                if member_access.ssa_version.is_some() && self.context.include_ssa_versions {
                    s.push('<');
                }
                s.push_str(&member_access.accept(self));
                if self.context.include_ssa_versions {
                    if let Some(ssa_version) = member_access.ssa_version {
                        s.push_str(&format!(">#{}", ssa_version));
                    }
                }
                s
            }
            AssignableKind::Identifier(identifier) => {
                let mut s = identifier.accept(self);
                if self.context.include_ssa_versions {
                    if let Some(ssa_version) = identifier.ssa_version {
                        s.push_str(&format!("#{}", ssa_version));
                    }
                }
                s
            }
            AssignableKind::ArrayAccess(array_access) => array_access.accept(self),
        }
    }

    /// Visits an array node.
    fn visit_array(&mut self, node: &crate::decompiler::ast::array::ArrayNode) -> String {
        let mut s = String::new();
        s.push('{');
        for (i, elem) in node.elements.iter().enumerate() {
            s.push_str(&elem.accept(self));
            if i < node.elements.len() - 1 {
                s.push_str(", ");
            }
        }
        s.push('}');
        s
    }

    /// Visits an array access node.
    fn visit_array_access(
        &mut self,
        node: &crate::decompiler::ast::array_access::ArrayAccessNode,
    ) -> String {
        let array_str = node.arr.accept(self);
        let index_str = node.index.accept(self);
        format!("{}[{}]", array_str, index_str)
    }

    /// Visits a binary operation node.
    fn visit_bin_op(&mut self, node: &BinaryOperationNode) -> String {
        let prev_context = self.context;
        self.context = self.context.with_expr_root(false);
        let lhs_str = node.lhs.accept(self);
        let rhs_str = node.rhs.accept(self);
        self.context = prev_context;
        let op_str = node.op_type.to_string();
        if self.context.expr_root {
            format!("{} {} {}", lhs_str, op_str, rhs_str)
        } else {
            format!("({} {} {})", lhs_str, op_str, rhs_str)
        }
    }

    /// Visits a unary operation node.
    fn visit_unary_op(&mut self, node: &UnaryOperationNode) -> String {
        let prev_context = self.context;
        self.context = self.context.with_expr_root(false);
        let operand_str = node.operand.accept(self);
        self.context = prev_context;
        let op_str = node.op_type.to_string();
        if self.context.expr_root {
            format!("{}{}", op_str, operand_str)
        } else {
            format!("({}{})", op_str, operand_str)
        }
    }

    /// Visits an identifier node.
    fn visit_identifier(&mut self, node: &IdentifierNode) -> String {
        node.id().to_string()
    }

    /// Visits a literal node.
    fn visit_literal(&mut self, node: &LiteralNode) -> String {
        match node {
            LiteralNode::String(s) => format!("\"{}\"", escape_string(s)),
            LiteralNode::Number(n) => {
                if self.context.format_number_hex {
                    format!("0x{:X}", n)
                } else {
                    n.to_string()
                }
            }
            LiteralNode::Float(f) => f.clone(),
            LiteralNode::Boolean(b) => b.to_string(),
            LiteralNode::Null => "null".to_string(),
        }
    }

    /// Visits a member access node.
    fn visit_member_access(&mut self, node: &MemberAccessNode) -> String {
        let lhs_str = node.lhs.accept(self);
        let rhs_str = node.rhs.accept(self);
        format!("{}.{}", lhs_str, rhs_str)
    }

    /// Visits a meta node.
    fn visit_meta(&mut self, node: &MetaNode) -> String {
        if self.context.verbosity == EmitVerbosity::Minified {
            return node.node().accept(self);
        }
        let mut result = String::new();
        if let Some(comment) = &node.comment() {
            result.push_str(&format!("// {}\n", comment));
        }
        result.push_str(&node.node().accept(self));
        result
    }

    /// Visits a function call node.
    fn visit_function_call(&mut self, node: &FunctionCallNode) -> String {
        let mut s = String::new();
        s.push_str(node.name.id_string().as_str());
        s.push('(');
        for (i, arg) in node.arguments.iter().enumerate() {
            s.push_str(&arg.accept(self));
            if i < node.arguments.len() - 1 {
                s.push_str(", ");
            }
        }
        s.push(')');
        s
    }

    /// Visits a function node.
    fn visit_function(&mut self, node: &FunctionNode) -> String {
        if node.name().is_none() {
            let mut s = String::new();
            for stmt in node.body().instructions.iter() {
                s.push_str(&stmt.accept(self));
                s.push('\n');
            }
            return s;
        }
        let name = node.name().as_ref().unwrap();
        let mut s = String::new();
        s.push_str(&format!("function {}(", name));
        for (i, param) in node.params().iter().enumerate() {
            s.push_str(&param.accept(self));
            if i < node.params().len() - 1 {
                s.push_str(", ");
            }
        }
        s.push(')');
        s.push_str(&node.body().accept(self));
        s
    }

    /// Visits a return node.
    fn visit_return(&mut self, node: &ReturnNode) -> String {
        let mut s = String::new();
        s.push_str("return ");
        s.push_str(&node.ret.accept(self));
        s
    }

    /// Visits a block node.
    fn visit_block(&mut self, node: &BlockNode) -> String {
        let mut s = String::new();
        if self.context.indent_style == IndentStyle::Allman {
            s.push('\n');
            s.push_str(&self.emit_indent());
            s.push_str("{\n");
        } else {
            s.push_str(" {\n");
        }
        let old_context = self.context;
        self.context = self.context.with_indent();
        if node.instructions.is_empty() {
            s.push_str(&self.emit_indent());
        } else {
            for stmt in node.instructions.iter() {
                s.push_str(&self.emit_indent());
                s.push_str(&stmt.accept(self));
                s.push('\n');
            }
        }
        self.context = old_context;
        s.push_str(&self.emit_indent());
        s.push('}');
        s
    }

    /// Visits a control flow node.
    fn visit_control_flow(&mut self, node: &ControlFlowNode) -> String {
        let mut s = String::new();
        let name = match node.ty() {
            ControlFlowType::If => "if",
            ControlFlowType::Else => "else",
            ControlFlowType::ElseIf => "else if",
            ControlFlowType::With => "with",
        };
        s.push_str(name);
        if let Some(condition) = node.condition() {
            s.push_str(" (");
            s.push_str(&condition.accept(self));
            s.push_str(") ");
        }
        s.push_str(&node.body().accept(self));
        s
    }
}

impl Gs2Emitter {
    /// Returns a string containing spaces corresponding to the current indentation level.
    fn emit_indent(&self) -> String {
        " ".repeat(self.context.indent)
    }
}
