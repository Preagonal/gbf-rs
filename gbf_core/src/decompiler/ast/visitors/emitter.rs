#![deny(missing_docs)]

use super::{
    emit_context::{EmitContext, EmitVerbosity, IndentStyle},
    AstVisitor,
};
use crate::decompiler::ast::member_access::MemberAccessNode;
use crate::decompiler::ast::meta::MetaNode;
use crate::decompiler::ast::statement::StatementNode;
use crate::decompiler::ast::unary_op::UnaryOperationNode;
use crate::decompiler::ast::{assignable::AssignableKind, expr::ExprKind};
use crate::decompiler::ast::{
    bin_op::{BinOpType, BinaryOperationNode},
    func_call::FunctionCallNode,
};
use crate::decompiler::ast::{function::FunctionNode, literal::LiteralNode};
use crate::decompiler::ast::{AstKind, AstVisitable};
use crate::{decompiler::ast::identifier::IdentifierNode, utils::escape_string};

/// An emitter for the AST.
pub struct Gs2Emitter {
    /// The output of the emitter.
    output: String,
    /// The context of the emitter.
    context: EmitContext,
}

impl Gs2Emitter {
    /// Creates a new `Gs2Emitter` with the given `context`.
    pub fn new(context: EmitContext) -> Self {
        Self {
            output: String::new(),
            context,
        }
    }

    /// Returns the output of the emitter.
    pub fn output(&self) -> &str {
        &self.output
    }
}

impl AstVisitor for Gs2Emitter {
    fn visit_node(&mut self, node: &AstKind) {
        match node {
            AstKind::Expression(expr) => expr.accept(self),
            AstKind::Meta(meta) => meta.accept(self),
            AstKind::Statement(stmt) => stmt.accept(self),
            AstKind::Function(func) => func.accept(self),
            AstKind::Empty => {}
        }
    }
    fn visit_statement(&mut self, stmt_node: &StatementNode) {
        // Step 0: Print the whitespace
        let indent_level = self.context.indent;
        for _ in 0..indent_level {
            self.output.push(' ');
        }
        // Step 1: Visit and emit the LHS
        stmt_node.lhs.accept(self);
        let lhs_str = self.output.clone();
        self.output.clear();

        // Step 2: Handle RHS
        if let ExprKind::BinOp(bin_op_node) = stmt_node.rhs.as_ref() {
            // Check if the binary operation directly involves the LHS
            let lhs_in_rhs =
                bin_op_node.lhs.as_ref() == &ExprKind::Assignable(*stmt_node.lhs.clone());

            if lhs_in_rhs {
                match bin_op_node.op_type {
                    BinOpType::Add => {
                        // Handle increment (++), compound assignment (+=), or fall back to addition
                        if let ExprKind::Literal(LiteralNode::Number(num)) =
                            bin_op_node.rhs.as_ref()
                        {
                            if *num == 1 {
                                // Emit increment (++)
                                self.output.push_str(&format!("{}++;", lhs_str));
                                return;
                            } else {
                                // Emit compound assignment (+=)
                                bin_op_node.rhs.accept(self); // Visit the RHS to get the formatted number
                                let rhs_str = self.output.clone();
                                self.output.clear();
                                self.output
                                    .push_str(&format!("{} += {};", lhs_str, rhs_str));
                                return;
                            }
                        }
                    }
                    BinOpType::Sub => {
                        // Handle decrement (--), compound assignment (-=), or fall back to subtraction
                        if let ExprKind::Literal(LiteralNode::Number(num)) =
                            bin_op_node.rhs.as_ref()
                        {
                            if *num == 1 {
                                // Emit decrement (--)
                                self.output.push_str(&format!("{}--;", lhs_str));
                                return;
                            } else {
                                // Emit compound assignment (-=)
                                bin_op_node.rhs.accept(self); // Visit the RHS to get the formatted number
                                let rhs_str = self.output.clone();
                                self.output.clear();
                                self.output
                                    .push_str(&format!("{} -= {};", lhs_str, rhs_str));
                                return;
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        // Step 3: Handle default assignment
        let prev_context = self.context;
        self.context = self.context.with_expr_root(true);

        stmt_node.rhs.accept(self); // Visit the RHS
        let rhs_str = self.output.clone();
        self.output.clear();

        self.context = prev_context; // Restore the context
        self.output.push_str(&format!("{} = {};", lhs_str, rhs_str));
    }

    fn visit_expr(&mut self, node: &ExprKind) {
        match node {
            ExprKind::Literal(literal) => literal.accept(self),
            ExprKind::Assignable(assignable) => self.visit_assignable_expr(assignable),
            ExprKind::BinOp(bin_op) => bin_op.accept(self),
            ExprKind::UnaryOp(unary_op) => unary_op.accept(self),
            ExprKind::FunctionCall(func_call) => func_call.accept(self),
        }
    }

    fn visit_assignable_expr(&mut self, node: &AssignableKind) {
        match node {
            AssignableKind::MemberAccess(member_access) => {
                if member_access.ssa_version.is_some() && self.context.include_ssa_versions {
                    self.output.push('{');
                }
                member_access.accept(self);
                if self.context.include_ssa_versions {
                    if let Some(ssa_version) = member_access.ssa_version {
                        self.output.push_str(&format!("}}@{}", ssa_version));
                    }
                }
            }
            AssignableKind::Identifier(identifier) => {
                identifier.accept(self);
                if self.context.include_ssa_versions {
                    if let Some(ssa_version) = identifier.ssa_version {
                        self.output.push_str(&format!("@{}", ssa_version));
                    }
                }
            }
        }
    }

    fn visit_bin_op(&mut self, node: &BinaryOperationNode) {
        // Save the current context and set expr_root to false for nested operations
        let prev_context = self.context;
        self.context = self.context.with_expr_root(false);

        // Visit and emit the left-hand side
        node.lhs.accept(self);
        let lhs_str = self.output.clone(); // Capture emitted LHS
        self.output.clear();

        // Visit and emit the right-hand side
        node.rhs.accept(self);
        let rhs_str = self.output.clone(); // Capture emitted RHS
        self.output.clear();

        // Restore the previous context
        self.context = prev_context;

        // Determine the operator string
        let op_str = node.op_type.to_string();

        // Combine the emitted parts into the final binary operation string
        if self.context.expr_root {
            // Emit without parentheses for root expressions
            self.output
                .push_str(&format!("{} {} {}", lhs_str, op_str, rhs_str));
        } else {
            // Emit with parentheses for nested expressions
            self.output
                .push_str(&format!("({} {} {})", lhs_str, op_str, rhs_str));
        }
    }

    fn visit_unary_op(&mut self, node: &UnaryOperationNode) {
        // Save the current context and set expr_root to false for the operand
        let prev_context = self.context;
        self.context = self.context.with_expr_root(false);

        // Visit and emit the operand
        node.operand.accept(self);
        let operand_str = self.output.clone(); // Capture emitted operand
        self.output.clear();

        // Restore the previous context
        self.context = prev_context;

        // Determine the operator string
        let op_str = node.op_type.to_string();

        // Combine the emitted parts into the final unary operation string
        if self.context.expr_root {
            self.output.push_str(&format!("{}{}", op_str, operand_str));
        } else {
            self.output
                .push_str(&format!("({}{})", op_str, operand_str));
        }
        // self.output.push_str(&format!("{}{}", op_str, operand_str));
    }

    fn visit_identifier(&mut self, node: &IdentifierNode) {
        // Append the identifier's ID directly to the output
        self.output.push_str(node.id());
    }

    fn visit_literal(&mut self, node: &LiteralNode) {
        let emitted_literal = match node {
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
        };

        self.output.push_str(&emitted_literal);
    }

    fn visit_member_access(&mut self, node: &MemberAccessNode) {
        // Visit and emit the LHS
        node.lhs.accept(self);
        let lhs_str = self.output.clone(); // Capture emitted LHS
        self.output.clear();

        // Visit and emit the RHS
        node.rhs.accept(self);
        let rhs_str = self.output.clone(); // Capture emitted RHS
        self.output.clear();

        // Combine the LHS and RHS with a dot for member access
        self.output.push_str(&format!("{}.{}", lhs_str, rhs_str));
    }

    fn visit_meta(&mut self, node: &MetaNode) {
        // Handle minified verbosity
        if self.context.verbosity == EmitVerbosity::Minified {
            node.node().accept(self);
            return;
        }

        let mut result = String::new();

        // Add comment if available
        if let Some(comment) = &node.comment() {
            result.push_str(&format!("// {}\n", comment));
        }

        // Visit and emit the inner node
        node.node().accept(self);
        result.push_str(&self.output);
        self.output.clear();

        // Store the result in the visitor's output
        self.output.push_str(&result);
    }

    fn visit_function_call(&mut self, node: &FunctionCallNode) {
        // Visit and emit the base
        if let Some(base) = &node.base {
            base.accept(self);
            let base_str = self.output.clone(); // Capture emitted base
            self.output.clear();

            // Combine the base and the function name with a dot for method calls
            self.output
                .push_str(&format!("{}.{}", base_str, node.name.id()));
        } else {
            // Emit the function name directly for function calls
            self.output.push_str(node.name.id());
        }

        // Emit the arguments
        self.output.push('(');
        for (i, arg) in node.arguments.iter().enumerate() {
            arg.accept(self);
            if i < node.arguments.len() - 1 {
                self.output.push_str(", ");
            }
        }
        self.output.push(')');
    }

    fn visit_function(&mut self, node: &FunctionNode) {
        let name = node.name().clone();
        if name.is_none() {
            // Just emit the function body if there is no name since we're
            // in the entry point function
            for stmt in node.body() {
                stmt.accept(self);
                self.output.push('\n');
            }
            return;
        }

        let name = name.unwrap();

        // Emit the function parameters
        self.output.push_str(format!("function {}(", name).as_str());
        for (i, param) in node.params().iter().enumerate() {
            param.accept(self);
            if i < node.params().len() - 1 {
                self.output.push_str(", ");
            }
        }
        self.output.push(')');

        // Check if we we are using allman or k&r style
        if self.context.indent_style == IndentStyle::Allman {
            self.output.push_str("\n{\n");
        } else {
            self.output.push_str(" {\n");
        }

        // Emit the function body, indented
        let old_context = self.context;
        self.context = self.context.with_indent();
        for stmt in node.body() {
            stmt.accept(self);
            self.output.push('\n');
        }
        self.context = old_context;
        self.output.push_str("}\n");
    }
}
