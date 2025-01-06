#![deny(missing_docs)]

use super::{
    emit_context::{EmitContext, EmitVerbosity},
    AstVisitor,
};
use crate::decompiler::ast::expr::ExprNode;
use crate::decompiler::ast::identifier::IdentifierNode;
use crate::decompiler::ast::literal::LiteralNode;
use crate::decompiler::ast::member_access::MemberAccessNode;
use crate::decompiler::ast::meta::MetaNode;
use crate::decompiler::ast::statement::StatementNode;
use crate::decompiler::ast::unary_op::{UnaryOpType, UnaryOperationNode};
use crate::decompiler::ast::{
    bin_op::{BinOpType, BinaryOperationNode},
    func_call::FunctionCallNode,
};
use crate::decompiler::ast::{AstNode, AstNodeTrait};

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
    fn visit_node(&mut self, node: &AstNode) {
        match node {
            AstNode::Expression(expr) => expr.accept(self),
            AstNode::Meta(meta) => meta.accept(self),
            AstNode::Statement(stmt) => stmt.accept(self),
            AstNode::Empty => {}
        }
    }
    fn visit_statement(&mut self, node: &StatementNode) {
        // Step 1: Visit and emit the LHS
        node.lhs.accept(self);
        let lhs_str = self.output.clone(); // Retrieve the emitted LHS
        self.output.clear();

        // Step 2: Handle RHS
        if let ExprNode::BinOp(bin_op_node) = node.rhs.as_ref() {
            // Check if the binary operation directly involves the LHS
            let lhs_in_rhs = bin_op_node.lhs.as_ref() == node.lhs.as_ref();

            if lhs_in_rhs {
                match bin_op_node.op_type {
                    BinOpType::Add => {
                        // Handle increment (++), compound assignment (+=), or fall back to addition
                        if let ExprNode::Literal(LiteralNode::Number(num)) =
                            bin_op_node.rhs.as_ref()
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
                    BinOpType::Sub => {
                        // Handle decrement (--), compound assignment (-=), or fall back to subtraction
                        if let ExprNode::Literal(LiteralNode::Number(num)) =
                            bin_op_node.rhs.as_ref()
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

    fn visit_expr(&mut self, node: &ExprNode) {
        match node {
            ExprNode::Literal(literal) => literal.accept(self),
            ExprNode::MemberAccess(member_access) => member_access.accept(self),
            ExprNode::Identifier(identifier) => identifier.accept(self),
            ExprNode::BinOp(bin_op) => bin_op.accept(self),
            ExprNode::UnaryOp(unary_op) => unary_op.accept(self),
            ExprNode::FunctionCall(func_call) => func_call.accept(self),
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
        let op_str = match node.op_type {
            BinOpType::Add => "+",
            BinOpType::Sub => "-",
            BinOpType::Mul => "*",
            BinOpType::Div => "/",
            BinOpType::Mod => "%",
            BinOpType::And => "&",
            BinOpType::Or => "|",
            BinOpType::Xor => "xor",
            BinOpType::LogicalAnd => "&&",
            BinOpType::LogicalOr => "||",
            BinOpType::Greater => ">",
            BinOpType::Less => "<",
            BinOpType::GreaterOrEqual => ">=",
            BinOpType::LessOrEqual => "<=",
            BinOpType::ShiftLeft => "<<",
            BinOpType::ShiftRight => ">>",
            BinOpType::Equal => "==",
            BinOpType::NotEqual => "!=",
            BinOpType::In => "in",
            BinOpType::Join => "@",
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
        let op_str = match node.op_type {
            UnaryOpType::Negate => "-",
            UnaryOpType::LogicalNot => "!",
            UnaryOpType::BitwiseNot => "~",
        };

        // Combine the emitted parts into the final unary operation string
        // if self.context.expr_root {
        //     self.output.push_str(&format!("{}{}", op_str, operand_str));
        // } else {
        //     self.output.push_str(&format!("{}({})", op_str, operand_str));
        // }
        self.output.push_str(&format!("{}{}", op_str, operand_str));
    }

    fn visit_identifier(&mut self, node: &IdentifierNode) {
        // Append the identifier's ID directly to the output
        self.output.push_str(node.id());
    }

    fn visit_literal(&mut self, node: &LiteralNode) {
        let emitted_literal = match node {
            LiteralNode::String(s) => format!("\"{}\"", s),
            LiteralNode::Number(n) => {
                if self.context.format_number_hex {
                    format!("0x{:X}", n)
                } else {
                    n.to_string()
                }
            }
            LiteralNode::Float(f) => f.clone(),
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
}

#[cfg(test)]
mod tests {
    use crate::decompiler::ast::bin_op::{BinOpType, BinaryOperationNode};
    use crate::decompiler::ast::expr::ExprNode;
    use crate::decompiler::ast::func_call::FunctionCallNode;
    use crate::decompiler::ast::identifier::IdentifierNode;
    use crate::decompiler::ast::literal::LiteralNode;
    use crate::decompiler::ast::member_access::MemberAccessNode;
    use crate::decompiler::ast::meta::MetaNode;
    use crate::decompiler::ast::statement::StatementNode;
    use crate::decompiler::ast::unary_op::{UnaryOpType, UnaryOperationNode};
    use crate::decompiler::ast::visitors::emit_context::EmitContextBuilder;
    use crate::decompiler::ast::visitors::emitter::Gs2Emitter;
    use crate::decompiler::ast::{AstNode, AstNodeTrait};

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
            BinaryOperationNode::new(lhs, rhs, BinOpType::Add).unwrap(),
        ))
    }

    fn create_unary_op(operand: Box<ExprNode>, op: UnaryOpType) -> Box<ExprNode> {
        Box::new(ExprNode::UnaryOp(
            UnaryOperationNode::new(operand, op).unwrap(),
        ))
    }

    fn create_subtraction(lhs: Box<ExprNode>, rhs: Box<ExprNode>) -> Box<ExprNode> {
        Box::new(ExprNode::BinOp(
            BinaryOperationNode::new(lhs, rhs, BinOpType::Sub).unwrap(),
        ))
    }

    fn create_statement(lhs: Box<ExprNode>, rhs: Box<ExprNode>) -> StatementNode {
        StatementNode::new(lhs, rhs).unwrap()
    }

    fn create_method_call(
        name: &str,
        args: Vec<ExprNode>,
        base: Box<ExprNode>,
    ) -> FunctionCallNode {
        FunctionCallNode::new(IdentifierNode::new(name.to_string()), args, Some(base))
    }

    fn create_member_access(lhs: Box<ExprNode>, rhs: Box<ExprNode>) -> MemberAccessNode {
        MemberAccessNode::new(lhs, rhs).unwrap()
    }

    fn create_function_call(name: &str, args: Vec<ExprNode>) -> FunctionCallNode {
        FunctionCallNode::new(IdentifierNode::new(name.to_string()), args, None)
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
        let ast_node = Box::new(AstNode::Statement(statement_node));

        let context = EmitContextBuilder::default().build();
        let mut visitor = Gs2Emitter::new(context);

        let meta_node = MetaNode::new(
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
        let member_access_node = MemberAccessNode::new(lhs, rhs).unwrap();

        let context = EmitContextBuilder::default().build();
        let mut visitor = Gs2Emitter::new(context);
        member_access_node.accept(&mut visitor);
        assert_eq!(visitor.output(), "temp.property");

        // test three-level member access
        let lhs = create_identifier("temp");
        let rhs = create_identifier("property");
        let member_access_node = MemberAccessNode::new(lhs, rhs).unwrap();
        let lhs = Box::new(ExprNode::MemberAccess(member_access_node));
        let rhs = create_identifier("property2");
        let member_access_node = MemberAccessNode::new(lhs, rhs).unwrap();

        let mut visitor = Gs2Emitter::new(context);
        member_access_node.accept(&mut visitor);
        assert_eq!(visitor.output(), "temp.property.property2");
    }

    #[test]
    fn test_literal_string() {
        // temp.asdf = "Hello, world!";
        let lhs = create_identifier("temp");
        let rhs = create_identifier("asdf");
        let member_access_node = MemberAccessNode::new(lhs, rhs).unwrap();
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

    #[test]
    fn test_all_bin_op_types() {
        use BinOpType;
        use ExprNode;
        use LiteralNode;

        let lhs = Box::new(ExprNode::Literal(LiteralNode::Number(1)));
        let rhs = Box::new(ExprNode::Literal(LiteralNode::Number(2)));

        let op_variants = vec![
            (BinOpType::Add, "1 + 2"),
            (BinOpType::Sub, "1 - 2"),
            (BinOpType::Mul, "1 * 2"),
            (BinOpType::Div, "1 / 2"),
            (BinOpType::Mod, "1 % 2"),
            (BinOpType::And, "1 & 2"),
            (BinOpType::Or, "1 | 2"),
            (BinOpType::Xor, "1 xor 2"),
            (BinOpType::LogicalAnd, "1 && 2"),
            (BinOpType::LogicalOr, "1 || 2"),
            (BinOpType::Greater, "1 > 2"),
            (BinOpType::Less, "1 < 2"),
            (BinOpType::GreaterOrEqual, "1 >= 2"),
            (BinOpType::LessOrEqual, "1 <= 2"),
            (BinOpType::ShiftLeft, "1 << 2"),
            (BinOpType::ShiftRight, "1 >> 2"),
            (BinOpType::Equal, "1 == 2"),
            (BinOpType::NotEqual, "1 != 2"),
            (BinOpType::In, "1 in 2"),
            (BinOpType::Join, "1 @ 2"),
        ];

        for (op, expected_output) in op_variants {
            let bin_op_node = BinaryOperationNode::new(lhs.clone(), rhs.clone(), op).unwrap();

            let context = EmitContextBuilder::default().build();
            let mut visitor = Gs2Emitter::new(context);

            bin_op_node.accept(&mut visitor);
            assert_eq!(visitor.output(), expected_output);
        }
    }

    #[test]
    fn test_all_unary_op_types() {
        use ExprNode;
        use LiteralNode;
        use UnaryOpType;

        let operand = Box::new(ExprNode::Literal(LiteralNode::Number(1)));

        let op_variants = vec![
            (UnaryOpType::Negate, "-1"),
            (UnaryOpType::LogicalNot, "!1"),
            (UnaryOpType::BitwiseNot, "~1"),
        ];

        for (op, expected_output) in op_variants {
            let unary_op_node = UnaryOperationNode::new(operand.clone(), op).unwrap();

            let context = EmitContextBuilder::default().build();
            let mut visitor = Gs2Emitter::new(context);

            unary_op_node.accept(&mut visitor);
            assert_eq!(visitor.output(), expected_output);
        }
    }

    #[test]
    fn test_hex_output() {
        let context = EmitContextBuilder::default()
            .format_number_hex(true)
            .build();
        let mut visitor = Gs2Emitter::new(context);

        let literal = LiteralNode::Number(42);
        let expr = ExprNode::Literal(literal);
        expr.accept(&mut visitor);
        assert_eq!(visitor.output(), "0x2A");

        let context = EmitContextBuilder::default()
            .format_number_hex(false)
            .build();
        let mut visitor = Gs2Emitter::new(context);

        let literal = LiteralNode::Number(42);
        let expr = ExprNode::Literal(literal);
        expr.accept(&mut visitor);
        assert_eq!(visitor.output(), "42");
    }

    #[test]
    fn test_float_output() {
        let context = EmitContextBuilder::default().build();
        let mut visitor = Gs2Emitter::new(context);

        let literal = LiteralNode::Float("3.14".to_string());
        let expr = ExprNode::Literal(literal);
        expr.accept(&mut visitor);
        assert_eq!(visitor.output(), "3.14");
    }

    #[test]
    fn test_function_call() {
        let context = EmitContextBuilder::default().build();
        let mut visitor = Gs2Emitter::new(context);

        let function_call = create_function_call("print", vec![*create_string_literal("Hello")]);
        function_call.accept(&mut visitor);
        assert_eq!(visitor.output(), "print(\"Hello\")");

        // two arguments 1, 2
        let mut visitor = Gs2Emitter::new(context);
        let function_call = create_function_call(
            "print",
            vec![*create_integer_literal(1), *create_integer_literal(2)],
        );
        function_call.accept(&mut visitor);
        assert_eq!(visitor.output(), "print(1, 2)");

        // method call
        let mut visitor = Gs2Emitter::new(context);
        let method_call = create_method_call(
            "print",
            vec![*create_integer_literal(1), *create_integer_literal(2)],
            create_identifier("console"),
        );
        method_call.accept(&mut visitor);
        assert_eq!(visitor.output(), "console.print(1, 2)");

        // method call temp.foo.bar(1, 2, 3)
        let mut visitor = Gs2Emitter::new(context);
        let method_call = create_method_call(
            "bar",
            vec![
                *create_integer_literal(1),
                *create_integer_literal(2),
                *create_integer_literal(3),
            ],
            Box::new(ExprNode::MemberAccess(create_member_access(
                create_identifier("temp"),
                create_identifier("foo"),
            ))),
        );
        method_call.accept(&mut visitor);
        assert_eq!(visitor.output(), "temp.foo.bar(1, 2, 3)");
    }
}
