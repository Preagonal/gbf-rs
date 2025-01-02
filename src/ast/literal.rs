#![deny(missing_docs)]

use serde::{Deserialize, Serialize};

use super::{
    emit::{EmitContext, EmitError},
    AstNodeTrait,
};

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

// == Other implementations for literal ==

impl AstNodeTrait for LiteralNode {
    fn emit(&self, ctx: &EmitContext) -> Result<String, EmitError> {
        Ok(match self {
            LiteralNode::String(s) => format!("\"{}\"", s),
            LiteralNode::Number(n) => {
                if ctx.format_number_hex {
                    format!("0x{:X}", n)
                } else {
                    n.to_string()
                }
            }
            LiteralNode::Float(f) => f.clone(),
        })
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

    #[test]
    fn test_number_emit() {
        let literal = LiteralNode::Number(42);
        let ctx = EmitContext::default();
        assert_eq!(literal.emit(&ctx).unwrap(), "42");

        let ctx = EmitContext {
            format_number_hex: true,
            ..Default::default()
        };
        assert_eq!(literal.emit(&ctx).unwrap(), "0x2A");
    }

    #[test]
    fn test_string_emit() {
        let literal = LiteralNode::String("Hello, world!".to_string());
        let ctx = EmitContext::default();
        assert_eq!(literal.emit(&ctx).unwrap(), "\"Hello, world!\"");
    }

    #[test]
    fn test_float_emit() {
        let literal = LiteralNode::Float("42.0".to_string());
        let ctx = EmitContext::default();
        assert_eq!(literal.emit(&ctx).unwrap(), "42.0");
    }
}
