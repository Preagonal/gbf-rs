#![deny(missing_docs)]

use serde::{Deserialize, Serialize};

use super::{expr::ExprNode, identifier::IdentifierNode, visitors::AstVisitor, AstNodeTrait};

/// Represents a function call
#[derive(Debug, Clone, Serialize, Deserialize, Eq)]
pub struct FunctionCallNode {
    /// The name of the function being called.
    pub name: IdentifierNode,
    /// The arguments to the function.
    pub arguments: Vec<ExprNode>,
    /// The base of the function call, if it's a method call.
    pub base: Option<Box<ExprNode>>,
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
        arguments: Vec<ExprNode>,
        base: Option<Box<ExprNode>>,
    ) -> Self {
        Self {
            base,
            name,
            arguments,
        }
    }
}

impl AstNodeTrait for FunctionCallNode {
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
