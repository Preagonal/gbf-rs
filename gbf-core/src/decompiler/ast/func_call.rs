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
    use crate::decompiler::ast::{call, emit, identifier, method_call};

    #[test]
    fn test_call_emit() {
        let call = call("echo", vec![identifier("hello")]);
        assert_eq!(emit(call), "echo(hello)");
    }

    #[test]
    fn test_call_equality() {
        let call1 = call("echo", vec![identifier("hello")]);
        let call2 = call("echo", vec![identifier("hello")]);
        assert_eq!(call1, call2);

        let call3 = call("echo", vec![identifier("world")]);
        assert_ne!(call1, call3);
    }

    #[test]
    fn test_method_call_emit() {
        let call = method_call(identifier("foo"), "bar", vec![identifier("baz")]);
        assert_eq!(emit(call), "foo.bar(baz)");
    }

    #[test]
    fn test_method_call_equality() {
        let call1 = method_call(identifier("foo"), "bar", vec![identifier("baz")]);
        let call2 = method_call(identifier("foo"), "bar", vec![identifier("baz")]);
        assert_eq!(call1, call2);

        let call3 = method_call(identifier("foo"), "bar", vec![identifier("qux")]);
        assert_ne!(call1, call3);
    }

    #[test]
    fn test_nested_call_emit() {
        let call = call("foo", vec![call("bar", vec![identifier("baz")])]);
        assert_eq!(emit(call), "foo(bar(baz))");
    }
}
