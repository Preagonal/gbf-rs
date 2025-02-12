#![deny(missing_docs)]

use super::{
    emit_context::{EmitContext, IndentStyle},
    AstVisitor,
};
use crate::decompiler::ast::{
    array::ArrayNode, array_access::ArrayAccessNode, assignable::AssignableKind,
    control_flow::ControlFlowType, expr::ExprKind, phi::PhiNode,
};
use crate::decompiler::ast::{assignment::AssignmentNode, statement::StatementKind};
use crate::decompiler::ast::{
    bin_op::{BinOpType, BinaryOperationNode},
    func_call::FunctionCallNode,
};
use crate::decompiler::ast::{block::BlockNode, ptr::P};
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

    /// Merges multiple comment vectors into a single one.
    ///
    /// This helper centralizes comment merging so that any future changes (such
    /// as deduplication or filtering) can be applied in one place.
    fn merge_comments(&self, comments: Vec<Vec<String>>) -> Vec<String> {
        comments.into_iter().flatten().collect()
    }

    /// Returns a string containing spaces corresponding to the current indentation level.
    fn emit_indent(&self) -> String {
        " ".repeat(self.context.indent)
    }
}

/// The output of the emitter.
pub struct AstOutput {
    /// The emitted node.
    pub node: String,
    /// The comments associated with the node.
    pub comments: Vec<String>,
}

impl AstVisitor for Gs2Emitter {
    type Output = AstOutput;

    /// Visits an AST node.
    fn visit_node(&mut self, node: &AstKind) -> AstOutput {
        match node {
            AstKind::Expression(expr) => expr.accept(self),
            AstKind::Statement(stmt) => stmt.accept(self),
            AstKind::Function(func) => func.accept(self),
            AstKind::Block(block) => block.accept(self),
            AstKind::ControlFlow(control_flow) => control_flow.accept(self),
        }
    }

    /// Visits a statement node.
    fn visit_statement(&mut self, node: &StatementKind) -> AstOutput {
        self.context = self.context.with_expr_root(true);
        let stmt_str = match node {
            StatementKind::Assignment(assignment) => assignment.accept(self),
            StatementKind::Return(ret) => ret.accept(self),
            StatementKind::VirtualBranch(vbranch) => vbranch.accept(self),
        };
        AstOutput {
            node: format!("{};", stmt_str.node),
            comments: stmt_str.comments,
        }
    }

    /// Visits an assignment node.
    fn visit_assignment(&mut self, stmt_node: &P<AssignmentNode>) -> AstOutput {
        let base_comments = stmt_node.metadata().comments().clone();
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
                                    return AstOutput {
                                        node: format!("{}++", lhs_str.node),
                                        comments: self.merge_comments(vec![
                                            base_comments.clone(),
                                            lhs_str.comments.clone(),
                                        ]),
                                    };
                                } else {
                                    let rhs_str = bin_op_node.rhs.accept(self);
                                    return AstOutput {
                                        node: format!("{} += {}", lhs_str.node, rhs_str.node),
                                        comments: self.merge_comments(vec![
                                            base_comments.clone(),
                                            lhs_str.comments.clone(),
                                            rhs_str.comments,
                                        ]),
                                    };
                                }
                            }
                        }
                    }
                    BinOpType::Sub => {
                        if let ExprKind::Literal(lit) = bin_op_node.rhs.clone() {
                            if let LiteralNode::Number(num) = lit.as_ref() {
                                if *num == 1 {
                                    return AstOutput {
                                        node: format!("{}--", lhs_str.node),
                                        comments: self.merge_comments(vec![
                                            base_comments.clone(),
                                            lhs_str.comments.clone(),
                                        ]),
                                    };
                                } else {
                                    let rhs_str = bin_op_node.rhs.accept(self);
                                    return AstOutput {
                                        node: format!("{} -= {}", lhs_str.node, rhs_str.node),
                                        comments: self.merge_comments(vec![
                                            base_comments.clone(),
                                            lhs_str.comments.clone(),
                                            rhs_str.comments,
                                        ]),
                                    };
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
        AstOutput {
            node: format!("{} = {}", lhs_str.node, rhs_str.node),
            comments: self.merge_comments(vec![base_comments, lhs_str.comments, rhs_str.comments]),
        }
    }

    /// Visits a virtual branch node.
    fn visit_virtual_branch(
        &mut self,
        node: &P<crate::decompiler::ast::vbranch::VirtualBranchNode>,
    ) -> Self::Output {
        // Put out `goto` statement
        let mut s = String::new();
        s.push_str("goto ");
        s.push_str(&node.branch().to_string());

        AstOutput {
            node: s,
            comments: node.metadata().comments().clone(),
        }
    }

    /// Visits an expression node.
    fn visit_expr(&mut self, node: &ExprKind) -> AstOutput {
        match node {
            ExprKind::Literal(literal) => literal.accept(self),
            ExprKind::Assignable(assignable) => self.visit_assignable_expr(assignable),
            ExprKind::BinOp(bin_op) => bin_op.accept(self),
            ExprKind::UnaryOp(unary_op) => unary_op.accept(self),
            ExprKind::FunctionCall(func_call) => func_call.accept(self),
            ExprKind::Array(array) => array.accept(self),
            ExprKind::New(new_node) => new_node.accept(self),
        }
    }

    /// Visits an assignable expression node.
    fn visit_assignable_expr(&mut self, node: &AssignableKind) -> AstOutput {
        match node {
            AssignableKind::MemberAccess(member_access) => {
                let mut s = String::new();
                if member_access.ssa_version.is_some() && self.context.include_ssa_versions {
                    s.push('<');
                }
                let member_access_out = member_access.accept(self);
                s.push_str(member_access_out.node.as_str());
                if self.context.include_ssa_versions {
                    if let Some(ssa_version) = member_access.ssa_version {
                        s.push_str(&format!(">#{}", ssa_version));
                    }
                }
                AstOutput {
                    node: s,
                    comments: member_access_out.comments.clone(),
                }
            }
            AssignableKind::Identifier(identifier) => {
                let out = identifier.accept(self);
                let mut s = out.node;
                if self.context.include_ssa_versions {
                    if let Some(ssa_version) = identifier.ssa_version {
                        s.push_str(&format!("#{}", ssa_version));
                    }
                }
                AstOutput {
                    node: s,
                    comments: out.comments,
                }
            }
            AssignableKind::ArrayAccess(array_access) => array_access.accept(self),
            AssignableKind::Phi(phi) => phi.accept(self),
        }
    }

    /// Visits an array node.
    fn visit_array(&mut self, node: &P<ArrayNode>) -> AstOutput {
        let mut s = String::new();
        let mut comments = node.metadata().comments().clone();
        s.push('{');
        for (i, elem) in node.elements.iter().enumerate() {
            let elem_out = elem.accept(self);
            s.push_str(&elem_out.node);
            comments.extend(elem_out.comments);
            if i < node.elements.len() - 1 {
                s.push_str(", ");
            }
        }
        s.push('}');
        AstOutput { node: s, comments }
    }

    /// Visits an array access node.
    fn visit_array_access(&mut self, node: &P<ArrayAccessNode>) -> AstOutput {
        let array_str = node.arr.accept(self);
        let index_str = node.index.accept(self);
        AstOutput {
            node: format!("{}[{}]", array_str.node, index_str.node),
            comments: self.merge_comments(vec![
                node.metadata().comments().clone(),
                array_str.comments,
                index_str.comments,
            ]),
        }
    }

    /// Visits a binary operation node.
    fn visit_bin_op(&mut self, node: &P<BinaryOperationNode>) -> AstOutput {
        let base_comments = node.metadata().comments().clone();
        let prev_context = self.context;
        self.context = self.context.with_expr_root(false);
        let lhs_str = node.lhs.accept(self);
        let rhs_str = node.rhs.accept(self);
        self.context = prev_context;
        let op_str = node.op_type.to_string();
        if self.context.expr_root {
            AstOutput {
                node: format!("{} {} {}", lhs_str.node, op_str, rhs_str.node),
                comments: self.merge_comments(vec![
                    base_comments,
                    lhs_str.comments,
                    rhs_str.comments,
                ]),
            }
        } else {
            AstOutput {
                node: format!("({} {} {})", lhs_str.node, op_str, rhs_str.node),
                comments: self.merge_comments(vec![
                    base_comments,
                    lhs_str.comments,
                    rhs_str.comments,
                ]),
            }
        }
    }

    /// Visits a unary operation node.
    fn visit_unary_op(&mut self, node: &P<UnaryOperationNode>) -> AstOutput {
        let base_comments = node.metadata().comments().clone();
        let prev_context = self.context;
        self.context = self.context.with_expr_root(false);
        let operand_str = node.operand.accept(self);
        self.context = prev_context;
        let op_str = node.op_type.to_string();
        if self.context.expr_root {
            AstOutput {
                node: format!("{}{}", op_str, operand_str.node),
                comments: self.merge_comments(vec![
                    base_comments,
                    node.metadata().comments().clone(),
                    operand_str.comments,
                ]),
            }
        } else {
            AstOutput {
                node: format!("({}{})", op_str, operand_str.node),
                comments: self.merge_comments(vec![
                    base_comments,
                    node.metadata().comments().clone(),
                    operand_str.comments,
                ]),
            }
        }
    }

    /// Visits an identifier node.
    fn visit_identifier(&mut self, node: &P<IdentifierNode>) -> AstOutput {
        AstOutput {
            node: node.id().to_string(),
            comments: node.metadata().comments().clone(),
        }
    }

    /// Visits a literal node.
    fn visit_literal(&mut self, node: &P<LiteralNode>) -> AstOutput {
        match node.as_ref() {
            LiteralNode::String(s) => {
                let escaped = escape_string(s);
                AstOutput {
                    node: format!("\"{}\"", escaped),
                    comments: node.metadata().comments().clone(),
                }
            }
            LiteralNode::Number(n) => {
                if self.context.format_number_hex {
                    AstOutput {
                        node: format!("0x{:x}", n),
                        comments: node.metadata().comments().clone(),
                    }
                } else {
                    AstOutput {
                        node: n.to_string(),
                        comments: node.metadata().comments().clone(),
                    }
                }
            }
            LiteralNode::Float(f) => AstOutput {
                node: f.to_string(),
                comments: node.metadata().comments().clone(),
            },
            LiteralNode::Boolean(b) => AstOutput {
                node: b.to_string(),
                comments: node.metadata().comments().clone(),
            },
            LiteralNode::Null => AstOutput {
                node: "null".to_string(),
                comments: node.metadata().comments().clone(),
            },
        }
    }

    /// Visits a member access node.
    fn visit_member_access(&mut self, node: &P<MemberAccessNode>) -> AstOutput {
        let lhs_str = node.lhs.accept(self);
        let rhs_str = node.rhs.accept(self);
        AstOutput {
            node: format!("{}.{}", lhs_str.node, rhs_str.node),
            comments: self.merge_comments(vec![
                node.metadata().comments().clone(),
                lhs_str.comments,
                rhs_str.comments,
            ]),
        }
    }

    /// Visits a function call node.
    fn visit_function_call(&mut self, node: &P<FunctionCallNode>) -> AstOutput {
        let mut s = String::new();
        let mut arg_comments = Vec::new();
        let name_out = node.name.accept(self);
        s.push_str(name_out.node.as_str());
        s.push('(');
        for (i, arg) in node.arguments.iter().enumerate() {
            let arg_out = arg.accept(self);
            s.push_str(&arg_out.node);
            arg_comments.extend(arg_out.comments);
            if i < node.arguments.len() - 1 {
                s.push_str(", ");
            }
        }
        s.push(')');
        AstOutput {
            node: s,
            comments: self.merge_comments(vec![node.metadata().comments().clone(), arg_comments]),
        }
    }

    /// Visits a function node.
    fn visit_function(&mut self, node: &P<FunctionNode>) -> AstOutput {
        let mut comments = node.metadata().comments().clone();
        if node.name().is_none() {
            let mut s = String::new();
            for stmt in node.body().instructions.iter() {
                let stmt_out = stmt.accept(self);
                // First emit any comments.
                for comment in stmt_out.comments.iter() {
                    s.push_str(&self.emit_indent());
                    s.push_str("// ");
                    s.push_str(comment);
                    s.push('\n');
                }

                s.push_str(&stmt_out.node);
                s.push('\n');
            }
            return AstOutput { node: s, comments };
        }
        let name = node.name().as_ref().unwrap();
        let mut s = String::new();
        s.push_str(&format!("function {}(", name));
        for (i, param) in node.params().iter().enumerate() {
            let param_out = param.accept(self);
            comments.extend(param_out.comments);
            s.push_str(&param_out.node);
            if i < node.params().len() - 1 {
                s.push_str(", ");
            }
        }
        let block_output = node.body().accept(self);
        s.push(')');
        s.push_str(&block_output.node);
        AstOutput {
            node: s,
            comments: self.merge_comments(vec![comments, block_output.comments]),
        }
    }

    /// Visits a return node.
    fn visit_return(&mut self, node: &P<ReturnNode>) -> AstOutput {
        let child = node.ret.accept(self);
        let mut s = String::new();
        s.push_str("return ");
        s.push_str(&child.node);
        AstOutput {
            node: s,
            comments: self.merge_comments(vec![node.metadata().comments().clone(), child.comments]),
        }
    }

    /// Visits a block node.
    fn visit_block(&mut self, node: &P<BlockNode>) -> AstOutput {
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
        if !node.instructions.is_empty() {
            for stmt in node.instructions.iter() {
                let stmt_out = stmt.accept(self);
                // First emit any comments.
                for comment in stmt_out.comments.iter() {
                    s.push_str(&self.emit_indent());
                    s.push_str("// ");
                    s.push_str(comment);
                    s.push('\n');
                }
                // Then emit the statement.
                s.push_str(&self.emit_indent());
                s.push_str(&stmt_out.node);
                s.push('\n');
            }
        }
        self.context = old_context;
        s.push_str(&self.emit_indent());
        s.push('}');
        AstOutput {
            node: s,
            comments: node.metadata().comments().clone(),
        }
    }

    /// Visits a control flow node.
    fn visit_control_flow(&mut self, node: &P<ControlFlowNode>) -> AstOutput {
        let mut s = String::new();
        let mut base_comments = node.metadata().comments().clone();
        let name = match node.ty() {
            ControlFlowType::If => "if",
            ControlFlowType::Else => "else",
            ControlFlowType::ElseIf => "else if",
            ControlFlowType::With => "with",
        };
        s.push_str(name);
        if let Some(condition) = node.condition() {
            let condition_out = condition.accept(self);
            s.push_str(" (");
            s.push_str(&condition_out.node);
            s.push_str(") ");
            base_comments.extend(condition_out.comments.clone());
        }
        let body_out = node.body().accept(self);
        s.push_str(&body_out.node);
        AstOutput {
            node: s,
            comments: self.merge_comments(vec![base_comments, body_out.comments]),
        }
    }

    /// Visits a phi node.
    fn visit_phi(&mut self, node: &P<PhiNode>) -> AstOutput {
        let mut s = String::new();
        s.push_str("phi<idx=");
        s.push_str(&node.index.to_string());
        s.push_str(", regions=(");
        for (i, region) in node.regions().iter().enumerate() {
            s.push_str(&region.0.to_string());
            if i < node.regions().len() - 1 {
                s.push_str(", ");
            }
        }
        s.push_str(")>");
        AstOutput {
            node: s,
            comments: node.metadata().comments().clone(),
        }
    }

    /// Visits a new node
    fn visit_new(&mut self, node: &P<crate::decompiler::ast::new::NewNode>) -> AstOutput {
        let type_out = node.new_type.accept(self);
        let arg_out = node.arg.accept(self);
        // TODO: if type_out is a string literal, we shouldn't put out the quotes.
        AstOutput {
            node: format!("new {}({})", type_out.node, arg_out.node),
            comments: self.merge_comments(vec![
                node.metadata().comments().clone(),
                type_out.comments,
                arg_out.comments,
            ]),
        }
    }
}
