#![deny(missing_docs)]

use serde::{Deserialize, Serialize};

use super::{visitors::AstVisitor, AstNodeTrait};

/// Represents a type of literal.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum LiteralNode {
    /// A string literal.
    String(String),
    /// A number literal.
    Number(i32),
    /// A floating point number literal (represented in GS2 as a string).
    Float(String),
}

impl LiteralNode {
    /// Creates a new `LiteralNode` from a string.
    pub fn new_string<S: Into<String>>(s: S) -> Self {
        Self::String(s.into())
    }

    /// Creates a new `LiteralNode` from a number.
    pub fn new_number(n: i32) -> Self {
        Self::Number(n)
    }

    /// Creates a new `LiteralNode` from a floating point number.
    pub fn new_float<S: Into<String>>(s: S) -> Self {
        Self::Float(s.into())
    }
}

impl AstNodeTrait for LiteralNode {
    fn accept(&self, visitor: &mut dyn AstVisitor) {
        visitor.visit_literal(self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_literal_node_eq() {
        let literal1 = LiteralNode::String("Hello, world!".to_string());
        let literal2 = LiteralNode::String("Hello, world!".to_string());
        let literal3 = LiteralNode::String("Goodbye, world!".to_string());

        assert_eq!(literal1, literal2);
        assert_ne!(literal1, literal3);

        // test number
        let literal1 = LiteralNode::Number(42);
        let literal2 = LiteralNode::Number(42);
        let literal3 = LiteralNode::Number(43);
        assert_eq!(literal1, literal2);
        assert_ne!(literal1, literal3);

        // test float
        let literal1 = LiteralNode::Float("42.0".to_string());
        let literal2 = LiteralNode::Float("42.0".to_string());
        let literal3 = LiteralNode::Float("43.0".to_string());
        assert_eq!(literal1, literal2);
        assert_ne!(literal1, literal3);
    }

    #[test]
    fn test_literal_node_ne_different_types() {
        let literal1 = LiteralNode::String("Hello, world!".to_string());
        let literal2 = LiteralNode::Number(42);

        assert_ne!(literal1, literal2);

        let literal1 = LiteralNode::String("Hello, world!".to_string());
        let literal2 = LiteralNode::Float("42.0".to_string());

        assert_ne!(literal1, literal2);

        let literal1 = LiteralNode::Float("42.0".to_string());
        let literal2 = LiteralNode::String("42.0".to_string());

        assert_ne!(literal1, literal2);
    }
}
