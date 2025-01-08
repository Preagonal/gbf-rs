#![deny(missing_docs)]

use gbf_macros::AstNodeTransform;
use serde::{Deserialize, Serialize};

use super::{
    expr::ExprKind, identifier::IdentifierNode, visitors::AstVisitor, AstKind, AstVec, AstVisitable,
};

/// Represents a function call
#[derive(Debug, Clone, Serialize, Deserialize, Eq, AstNodeTransform)]
#[convert_to(ExprKind::FunctionCall, AstKind::Expression)]
pub struct FunctionCallNode {
    /// The name of the function being called.
    pub name: IdentifierNode,
    /// The arguments to the function.
    pub arguments: AstVec<ExprKind>,
    /// The base of the function call, if it's a method call.
    pub base: Option<Box<ExprKind>>,
}

impl FunctionCallNode {
    /// Creates a new function call node.
    ///
    /// # Arguments
    /// - `name`: The name of the function being called.
    /// - `arguments`: The arguments to the function.
    /// - `base`: The base of the function call, if it's a method call.
    pub fn new(
        name: IdentifierNode,
        arguments: AstVec<ExprKind>,
        base: Option<Box<ExprKind>>,
    ) -> Self {
        Self {
            base,
            name,
            arguments,
        }
    }
}

impl AstVisitable for FunctionCallNode {
    fn accept(&self, visitor: &mut dyn AstVisitor) {
        visitor.visit_function_call(self);
    }
}

// == Other implementations for unary operations ==
impl PartialEq for FunctionCallNode {
    fn eq(&self, other: &Self) -> bool {
        self.base == other.base && self.name == other.name && self.arguments == other.arguments
    }
}

#[cfg(test)]
mod tests {
    use crate::decompiler::ast::{emit, new_fn_call, new_id, new_m_call};

    #[test]
    fn test_call_emit() {
        let call = new_fn_call(new_id("echo"), vec![new_id("hello")]);
        assert_eq!(emit(call), "echo(hello)");
    }

    #[test]
    fn test_call_equality() {
        let call1 = new_fn_call(new_id("echo"), vec![new_id("hello")]);
        let call2 = new_fn_call(new_id("echo"), vec![new_id("hello")]);
        assert_eq!(call1, call2);

        let call3 = new_fn_call(new_id("echo"), vec![new_id("world")]);
        assert_ne!(call1, call3);
    }

    #[test]
    fn test_method_call_emit() {
        let call = new_m_call(new_id("foo"), new_id("bar"), vec![new_id("baz")]);
        assert_eq!(emit(call), "foo.bar(baz)");
    }

    #[test]
    fn test_method_call_equality() {
        let call1 = new_m_call(new_id("foo"), new_id("bar"), vec![new_id("baz")]);
        let call2 = new_m_call(new_id("foo"), new_id("bar"), vec![new_id("baz")]);
        assert_eq!(call1, call2);

        let call3 = new_m_call(new_id("foo"), new_id("bar"), vec![new_id("qux")]);
        assert_ne!(call1, call3);
    }

    #[test]
    fn test_nested_call_emit() {
        let call = new_fn_call(
            new_id("foo"),
            vec![new_fn_call(new_id("bar"), vec![new_id("baz")])],
        );
        assert_eq!(emit(call), "foo(bar(baz))");
    }
}
