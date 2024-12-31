#![deny(missing_docs)]

use serde::{Deserialize, Serialize};

use super::{emit::EmitContext, AstNodeTrait};

/// Represents a type of literal.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum LiteralType {
    /// A string literal.
    String(String),
    /// A number literal.
    Number(i32),
    /// A floating point number literal (represented in GS2 as a string).
    Float(String),
}

/// Represents a literal node in the AST.
#[derive(Debug, Clone, Serialize, Deserialize, Eq)]
pub struct LiteralNode {
    /// The value of the literal.
    pub value: LiteralType,
}

// == Other implementations for literal ==

impl PartialEq for LiteralNode {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl AstNodeTrait for LiteralNode {
    fn emit(&self, ctx: &EmitContext) -> String {
        match &self.value {
            LiteralType::String(s) => format!("\"{}\"", s),
            LiteralType::Number(n) => {
                if ctx.format_number_hex {
                    format!("0x{:X}", n)
                } else {
                    n.to_string()
                }
            }
            LiteralType::Float(f) => f.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_literal_node_eq() {
        let literal1 = LiteralNode {
            value: LiteralType::String("Hello, world!".to_string()),
        };
        let literal2 = LiteralNode {
            value: LiteralType::String("Hello, world!".to_string()),
        };
        let literal3 = LiteralNode {
            value: LiteralType::String("Goodbye, world!".to_string()),
        };

        assert_eq!(literal1, literal2);
        assert_ne!(literal1, literal3);

        // test number
        let literal1 = LiteralNode {
            value: LiteralType::Number(42),
        };
        let literal2 = LiteralNode {
            value: LiteralType::Number(42),
        };
        let literal3 = LiteralNode {
            value: LiteralType::Number(43),
        };
        assert!(literal1 == literal2);
        assert!(literal1 != literal3);

        // test float
        let literal1 = LiteralNode {
            value: LiteralType::Float("42.0".to_string()),
        };
        let literal2 = LiteralNode {
            value: LiteralType::Float("42.0".to_string()),
        };
        let literal3 = LiteralNode {
            value: LiteralType::Float("43.0".to_string()),
        };
        assert!(literal1 == literal2);
        assert!(literal1 != literal3);
    }

    #[test]
    fn test_literal_node_ne_different_types() {
        let literal1 = LiteralNode {
            value: LiteralType::String("Hello, world!".to_string()),
        };
        let literal2 = LiteralNode {
            value: LiteralType::Number(42),
        };

        assert_ne!(literal1, literal2);

        let literal1 = LiteralNode {
            value: LiteralType::String("Hello, world!".to_string()),
        };
        let literal2 = LiteralNode {
            value: LiteralType::Float("42.0".to_string()),
        };

        assert_ne!(literal1, literal2);

        let literal1 = LiteralNode {
            value: LiteralType::Float("42.0".to_string()),
        };
        let literal2 = LiteralNode {
            value: LiteralType::String("42.0".to_string()),
        };

        assert_ne!(literal1, literal2);
    }

    #[test]
    fn test_number_emit() {
        let literal = LiteralNode {
            value: LiteralType::Number(42),
        };
        let ctx = EmitContext::default();
        assert_eq!(literal.emit(&ctx), "42");

        let ctx = EmitContext::builder().format_number_hex(true).build();
        assert_eq!(literal.emit(&ctx), "0x2A");
    }

    #[test]
    fn test_string_emit() {
        let literal = LiteralNode {
            value: LiteralType::String("Hello, world!".to_string()),
        };
        let ctx = EmitContext::default();
        assert_eq!(literal.emit(&ctx), "\"Hello, world!\"");
    }

    #[test]
    fn test_float_emit() {
        let literal = LiteralNode {
            value: LiteralType::Float("42.0".to_string()),
        };
        let ctx = EmitContext::default();
        assert_eq!(literal.emit(&ctx), "42.0");
    }
}
