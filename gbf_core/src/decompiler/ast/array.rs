#![deny(missing_docs)]

use gbf_macros::AstNodeTransform;
use serde::{Deserialize, Serialize};

use super::{expr::ExprKind, ptr::P, visitors::AstVisitor, AstKind, AstVisitable};

/// Represents a function call
#[derive(Debug, Clone, Serialize, Deserialize, Eq, AstNodeTransform)]
#[convert_to(ExprKind::Array, AstKind::Expression)]
pub struct ArrayNode {
    /// The arguments to the function.
    pub elements: Vec<ExprKind>,
}

impl ArrayNode {
    /// Creates a new array.
    ///
    /// # Arguments
    /// - `elements`: The elements of the array.
    pub fn new(elements: Vec<ExprKind>) -> Self {
        Self { elements }
    }
}

impl AstVisitable for ArrayNode {
    fn accept<V: AstVisitor>(&self, visitor: &mut V) -> V::Output {
        visitor.visit_array(self)
    }
}

// == Other implementations for unary operations ==
impl PartialEq for ArrayNode {
    fn eq(&self, other: &Self) -> bool {
        self.elements == other.elements
    }
}

#[cfg(test)]
mod tests {
    use crate::decompiler::ast::{new_array, new_num, new_str};

    #[test]
    fn test_array_node() {
        let array = new_array(vec![new_num(5), new_num(6)]);
        let array_two = new_array(vec![new_num(5), new_num(6)]);
        assert_eq!(array, array_two);
    }

    #[test]
    fn test_array_node_emit() {
        let array = new_array(vec![new_num(5), new_num(6)]);
        assert_eq!(crate::decompiler::ast::emit(array), "{5, 6}");
    }

    #[test]
    fn test_array_node_emit_str() {
        let array = new_array(vec![new_str("hello"), new_str("world")]);
        assert_eq!(crate::decompiler::ast::emit(array), r#"{"hello", "world"}"#);
    }
}
