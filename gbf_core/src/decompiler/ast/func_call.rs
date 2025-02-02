#![deny(missing_docs)]

use gbf_macros::AstNodeTransform;
use serde::{Deserialize, Serialize};

use super::{
    assignable::AssignableKind, expr::ExprKind, ptr::P, visitors::AstVisitor, AstKind, AstVisitable,
};

/// Represents a function call
#[derive(Debug, Clone, Serialize, Deserialize, Eq, AstNodeTransform)]
#[convert_to(ExprKind::FunctionCall, AstKind::Expression)]
pub struct FunctionCallNode {
    /// The name of the function being called.
    pub name: AssignableKind,
    /// The arguments to the function.
    pub arguments: Vec<ExprKind>,
}

impl FunctionCallNode {
    /// Creates a new function call node.
    ///
    /// # Arguments
    /// - `name`: The name of the function being called.
    /// - `arguments`: The arguments to the function.
    /// - `base`: The base of the function call, if it's a method call.
    pub fn new(name: AssignableKind, arguments: Vec<ExprKind>) -> Self {
        Self { name, arguments }
    }
}

impl AstVisitable for FunctionCallNode {
    fn accept<V: AstVisitor>(&self, visitor: &mut V) -> V::Output {
        visitor.visit_function_call(self)
    }
}

// == Other implementations for unary operations ==
impl PartialEq for FunctionCallNode {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.arguments == other.arguments
    }
}

#[cfg(test)]
mod tests {
    use crate::decompiler::ast::{emit, new_fn_call, new_id, new_member_access, AstNodeError};

    #[test]
    fn test_call_emit() -> Result<(), AstNodeError> {
        let call = new_fn_call(new_id("echo"), vec![new_id("hello").into()]);
        assert_eq!(emit(call), "echo(hello)");

        // foo.bar(baz)
        let ma = new_member_access(new_id("foo"), new_id("bar"))?;
        let call = new_fn_call(ma, vec![new_id("baz").into()]);
        assert_eq!(emit(call), "foo.bar(baz)");
        Ok(())
    }

    #[test]
    fn test_call_equality() {
        let call1 = new_fn_call(new_id("echo"), vec![new_id("hello").into()]);
        let call2 = new_fn_call(new_id("echo"), vec![new_id("hello").into()]);
        assert_eq!(call1, call2);

        let call3 = new_fn_call(new_id("echo"), vec![new_id("world").into()]);
        assert_ne!(call1, call3);
    }

    #[test]
    fn test_nested_call_emit() {
        let call = new_fn_call(
            new_id("foo"),
            vec![new_fn_call(new_id("bar"), vec![new_id("baz").into()]).into()],
        );
        assert_eq!(emit(call), "foo(bar(baz))");
    }
}
