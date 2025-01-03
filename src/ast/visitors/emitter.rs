#![deny(missing_docs)]

use crate::ast::AstNodeTrait;

use super::{
    emit_context::{EmitContext, EmitVerbosity},
    AstVisitor,
};

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
    fn visit_statement(&mut self, node: &crate::ast::statement::StatementNode) {
        // Step 1: Visit and emit the LHS
        node.lhs.accept(self);
        let lhs_str = self.output.clone(); // Retrieve the emitted LHS
        self.output.clear();

        // Step 2: Handle RHS
        if let crate::ast::expr::ExprNode::BinOp(bin_op_node) = node.rhs.as_ref() {
            // Check if the binary operation directly involves the LHS
            let lhs_in_rhs = bin_op_node.lhs.as_ref() == node.lhs.as_ref();

            if lhs_in_rhs {
                match bin_op_node.op_type {
                    crate::ast::bin_op::BinOpType::Add => {
                        // Handle increment (++), compound assignment (+=), or fall back to addition
                        if let crate::ast::expr::ExprNode::Literal(
                            crate::ast::literal::LiteralNode::Number(num),
                        ) = bin_op_node.rhs.as_ref()
                        {
                            if *num == 1 {
                                // Emit increment (++)
                                self.output.push_str(&format!("{}++;", lhs_str));
                                return;
                            } else {
                                // Emit compound assignment (+=)
                                self.output.push_str(&format!("{} += {};", lhs_str, num));
                                return;
                            }
                        }
                    }
                    crate::ast::bin_op::BinOpType::Sub => {
                        // Handle decrement (--), compound assignment (-=), or fall back to subtraction
                        if let crate::ast::expr::ExprNode::Literal(
                            crate::ast::literal::LiteralNode::Number(num),
                        ) = bin_op_node.rhs.as_ref()
                        {
                            if *num == 1 {
                                // Emit decrement (--)
                                self.output.push_str(&format!("{}--;", lhs_str));
                                return;
                            } else {
                                // Emit compound assignment (-=)
                                self.output.push_str(&format!("{} -= {};", lhs_str, num));
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

        node.rhs.accept(self); // Visit the RHS
        let rhs_str = self.output.clone();
        self.output.clear();

        self.context = prev_context; // Restore the context
        self.output.push_str(&format!("{} = {};", lhs_str, rhs_str));
    }

    fn visit_expr(&mut self, node: &crate::ast::expr::ExprNode) {
        match node {
            crate::ast::expr::ExprNode::Literal(literal) => literal.accept(self),
            crate::ast::expr::ExprNode::MemberAccess(member_access) => member_access.accept(self),
            crate::ast::expr::ExprNode::Identifier(identifier) => identifier.accept(self),
            crate::ast::expr::ExprNode::BinOp(bin_op) => bin_op.accept(self),
            crate::ast::expr::ExprNode::UnaryOp(unary_op) => unary_op.accept(self),
        }
    }

    fn visit_bin_op(&mut self, node: &crate::ast::bin_op::BinaryOperationNode) {
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
        let op_str = match node.op_type {
            crate::ast::bin_op::BinOpType::Add => "+",
            crate::ast::bin_op::BinOpType::Sub => "-",
            crate::ast::bin_op::BinOpType::Mul => "*",
            crate::ast::bin_op::BinOpType::Div => "/",
            crate::ast::bin_op::BinOpType::Mod => "%",
            crate::ast::bin_op::BinOpType::And => "&",
            crate::ast::bin_op::BinOpType::Or => "|",
            crate::ast::bin_op::BinOpType::Xor => "xor",
            crate::ast::bin_op::BinOpType::LogicalAnd => "&&",
            crate::ast::bin_op::BinOpType::LogicalOr => "||",
            crate::ast::bin_op::BinOpType::Greater => ">",
            crate::ast::bin_op::BinOpType::Less => "<",
            crate::ast::bin_op::BinOpType::GreaterOrEqual => ">=",
            crate::ast::bin_op::BinOpType::LessOrEqual => "<=",
            crate::ast::bin_op::BinOpType::ShiftLeft => "<<",
            crate::ast::bin_op::BinOpType::ShiftRight => ">>",
            crate::ast::bin_op::BinOpType::Equal => "==",
            crate::ast::bin_op::BinOpType::NotEqual => "!=",
            crate::ast::bin_op::BinOpType::In => "in",
            crate::ast::bin_op::BinOpType::Join => "@",
        };

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

    fn visit_unary_op(&mut self, node: &crate::ast::unary_op::UnaryOperationNode) {
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
        let op_str = match node.op_type {
            crate::ast::unary_op::UnaryOpType::Negate => "-",
            crate::ast::unary_op::UnaryOpType::LogicalNot => "!",
            crate::ast::unary_op::UnaryOpType::BitwiseNot => "~",
        };

        // Combine the emitted parts into the final unary operation string
        // if self.context.expr_root {
        //     self.output.push_str(&format!("{}{}", op_str, operand_str));
        // } else {
        //     self.output.push_str(&format!("{}({})", op_str, operand_str));
        // }
        self.output.push_str(&format!("{}{}", op_str, operand_str));
    }

    fn visit_identifier(&mut self, node: &crate::ast::identifier::IdentifierNode) {
        // Append the identifier's ID directly to the output
        self.output.push_str(node.id());
    }

    fn visit_literal(&mut self, node: &crate::ast::literal::LiteralNode) {
        let emitted_literal = match node {
            crate::ast::literal::LiteralNode::String(s) => format!("\"{}\"", s),
            crate::ast::literal::LiteralNode::Number(n) => {
                if self.context.format_number_hex {
                    format!("0x{:X}", n)
                } else {
                    n.to_string()
                }
            }
            crate::ast::literal::LiteralNode::Float(f) => f.clone(),
        };

        self.output.push_str(&emitted_literal);
    }

    fn visit_member_access(&mut self, node: &crate::ast::member_access::MemberAccessNode) {
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

    fn visit_meta(&mut self, node: &crate::ast::meta::MetaNode) {
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
}

#[cfg(test)]
mod tests {
    use crate::ast::unary_op::UnaryOpType;
    use crate::ast::{
        expr::ExprNode,
        identifier::IdentifierNode,
        literal::LiteralNode,
        statement::StatementNode,
        visitors::{emit_context::EmitContextBuilder, emitter::Gs2Emitter},
        AstNode, AstNodeTrait,
    };

    fn create_identifier(id: &str) -> Box<ExprNode> {
        Box::new(ExprNode::Identifier(IdentifierNode::new(id.to_string())))
    }

    fn create_integer_literal(value: i32) -> Box<ExprNode> {
        Box::new(ExprNode::Literal(LiteralNode::Number(value)))
    }

    fn create_string_literal(value: &str) -> Box<ExprNode> {
        Box::new(ExprNode::Literal(LiteralNode::String(value.to_string())))
    }

    fn create_addition(lhs: Box<ExprNode>, rhs: Box<ExprNode>) -> Box<ExprNode> {
        Box::new(ExprNode::BinOp(
            crate::ast::bin_op::BinaryOperationNode::new(
                lhs,
                rhs,
                crate::ast::bin_op::BinOpType::Add,
            )
            .unwrap(),
        ))
    }

    fn create_unary_op(operand: Box<ExprNode>, op: UnaryOpType) -> Box<ExprNode> {
        Box::new(ExprNode::UnaryOp(
            crate::ast::unary_op::UnaryOperationNode::new(operand, op).unwrap(),
        ))
    }

    fn create_subtraction(lhs: Box<ExprNode>, rhs: Box<ExprNode>) -> Box<ExprNode> {
        Box::new(ExprNode::BinOp(
            crate::ast::bin_op::BinaryOperationNode::new(
                lhs,
                rhs,
                crate::ast::bin_op::BinOpType::Sub,
            )
            .unwrap(),
        ))
    }

    fn create_statement(lhs: Box<ExprNode>, rhs: Box<ExprNode>) -> Box<StatementNode> {
        StatementNode::new(lhs, rhs).unwrap()
    }

    #[test]
    fn test_statement_emit() {
        let lhs = create_identifier("variable");
        let rhs = create_integer_literal(42);
        let statement_node = create_statement(lhs, rhs);

        let context = EmitContextBuilder::default().build();
        let mut visitor = Gs2Emitter::new(context);
        statement_node.accept(&mut visitor);
        assert_eq!(visitor.output(), "variable = 42;");
    }

    #[test]
    fn test_statement_nested_bin_op() {
        let lhs = create_identifier("variable");
        let rhs = create_addition(lhs.clone_box(), create_integer_literal(1));
        let statement_node = create_statement(lhs, rhs);

        let context = EmitContextBuilder::default().build();
        let mut visitor = Gs2Emitter::new(context);
        statement_node.accept(&mut visitor);
        assert_eq!(visitor.output(), "variable++;");

        let lhs = create_identifier("variable");
        let rhs = create_addition(lhs.clone_box(), create_integer_literal(2));
        let statement_node = create_statement(lhs, rhs);
        let mut visitor = Gs2Emitter::new(context);
        statement_node.accept(&mut visitor);
        assert_eq!(visitor.output(), "variable += 2;");

        // test -- case
        let lhs = create_identifier("variable");
        let rhs = create_subtraction(lhs.clone_box(), create_integer_literal(1));
        let statement_node = create_statement(lhs, rhs);
        let mut visitor = Gs2Emitter::new(context);
        statement_node.accept(&mut visitor);
        assert_eq!(visitor.output(), "variable--;");

        // test -= case
        let lhs = create_identifier("variable");
        let rhs = create_subtraction(lhs.clone_box(), create_integer_literal(2));
        let statement_node = create_statement(lhs, rhs);
        let mut visitor = Gs2Emitter::new(context);
        statement_node.accept(&mut visitor);
        assert_eq!(visitor.output(), "variable -= 2;");

        // test variable = 1 + (2 + 3)
        let lhs = create_identifier("variable");
        let rhs = create_addition(
            create_integer_literal(1),
            create_addition(create_integer_literal(2), create_integer_literal(3)),
        );
        let statement_node = create_statement(lhs, rhs);
        let mut visitor = Gs2Emitter::new(context);
        statement_node.accept(&mut visitor);
        assert_eq!(visitor.output(), "variable = 1 + (2 + 3);");

        // test variable = (1 + 2) + (3 + 4)
        let lhs = create_identifier("variable");
        let rhs = create_addition(
            create_addition(create_integer_literal(1), create_integer_literal(2)),
            create_addition(create_integer_literal(3), create_integer_literal(4)),
        );
        let statement_node = create_statement(lhs, rhs);
        let mut visitor = Gs2Emitter::new(context);
        statement_node.accept(&mut visitor);
        assert_eq!(visitor.output(), "variable = (1 + 2) + (3 + 4);");
    }

    #[test]
    fn test_meta() {
        let lhs = create_identifier("variable");
        let rhs = create_integer_literal(42);
        let statement_node = create_statement(lhs, rhs);
        let ast_node = Box::new(AstNode::Statement(*statement_node));

        let context = EmitContextBuilder::default().build();
        let mut visitor = Gs2Emitter::new(context);

        let meta_node = crate::ast::meta::MetaNode::new(
            ast_node,
            Some("This is a comment".to_string()),
            None,
            Default::default(),
        );
        meta_node.accept(&mut visitor);
        assert_eq!(visitor.output(), "// This is a comment\nvariable = 42;");
    }

    #[test]
    fn test_member_access() {
        let lhs = create_identifier("temp");
        let rhs = create_identifier("property");
        let member_access_node =
            crate::ast::member_access::MemberAccessNode::new(lhs, rhs).unwrap();

        let context = EmitContextBuilder::default().build();
        let mut visitor = Gs2Emitter::new(context);
        member_access_node.accept(&mut visitor);
        assert_eq!(visitor.output(), "temp.property");

        // test three-level member access
        let lhs = create_identifier("temp");
        let rhs = create_identifier("property");
        let member_access_node =
            crate::ast::member_access::MemberAccessNode::new(lhs, rhs).unwrap();
        let lhs = Box::new(ExprNode::MemberAccess(member_access_node));
        let rhs = create_identifier("property2");
        let member_access_node =
            crate::ast::member_access::MemberAccessNode::new(lhs, rhs).unwrap();

        let mut visitor = Gs2Emitter::new(context);
        member_access_node.accept(&mut visitor);
        assert_eq!(visitor.output(), "temp.property.property2");
    }

    #[test]
    fn test_literal_string() {
        // temp.asdf = "Hello, world!";
        let lhs = create_identifier("temp");
        let rhs = create_identifier("asdf");
        let member_access_node =
            crate::ast::member_access::MemberAccessNode::new(lhs, rhs).unwrap();
        let hello = create_string_literal("Hello, world!");
        let statement_node =
            create_statement(Box::new(ExprNode::MemberAccess(member_access_node)), hello);

        let context = EmitContextBuilder::default().build();
        let mut visitor = Gs2Emitter::new(context);
        statement_node.accept(&mut visitor);
        assert_eq!(visitor.output(), "temp.asdf = \"Hello, world!\";");
    }

    #[test]
    fn test_unary_operation() {
        // i = -(42 + 1);
        let context = EmitContextBuilder::default().build();
        let mut visitor = Gs2Emitter::new(context);

        let operand = create_addition(create_integer_literal(42), create_integer_literal(1));
        let lhs = create_identifier("i");
        let unary_op_node = create_unary_op(operand, UnaryOpType::Negate);
        let statement_node = create_statement(lhs, unary_op_node);
        statement_node.accept(&mut visitor);
        assert_eq!(visitor.output(), "i = -(42 + 1);");

        // i = -42;
        let context = EmitContextBuilder::default().build();
        let mut visitor = Gs2Emitter::new(context);

        let operand = create_integer_literal(42);
        let unary_op_node = create_unary_op(operand, UnaryOpType::Negate);
        let lhs = create_identifier("i");
        let statement_node = create_statement(lhs, unary_op_node);
        statement_node.accept(&mut visitor);
        assert_eq!(visitor.output(), "i = -42;");
    }
}
