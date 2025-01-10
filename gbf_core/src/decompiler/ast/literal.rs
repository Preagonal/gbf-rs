#![deny(missing_docs)]

use gbf_macros::AstNodeTransform;
use serde::{Deserialize, Serialize};

use super::{expr::ExprKind, visitors::AstVisitor, AstKind, AstVisitable};

/// Represents a type of literal.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, AstNodeTransform)]
#[convert_to(ExprKind::Literal, AstKind::Expression)]
pub enum LiteralNode {
    /// A string literal.
    String(String),
    /// A number literal.
    Number(i32),
    /// A floating point number literal (represented in GS2 as a string).
    Float(String),
    /// A boolean literal.
    Boolean(bool),
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

    /// Creates a new `LiteralNode` from a boolean.
    pub fn new_boolean(b: bool) -> Self {
        Self::Boolean(b)
    }
}

impl AstVisitable for LiteralNode {
    fn accept(&self, visitor: &mut dyn AstVisitor) {
        visitor.visit_literal(self);
    }
}

#[cfg(test)]
mod tests {
    use crate::decompiler::ast::{emit, new_bool, new_float, new_num, new_str};

    #[test]
    fn test_literal_emit() {
        let string = new_str("str");
        assert_eq!(emit(string), "\"str\"");

        let number = new_num(42);
        assert_eq!(emit(number), "42");

        let float = new_float("3.14");
        assert_eq!(emit(float), "3.14");

        let boolean = new_bool(true);
        assert_eq!(emit(boolean), "true");
    }

    #[test]
    fn test_literal_equalities() {
        let string = new_str("str");
        let number = new_num(42);
        let float = new_float("3.14");
        let boolean = new_bool(true);

        assert_eq!(string, new_str("str"));
        assert_ne!(string, number);
        assert_ne!(string, float);
        assert_ne!(string, boolean);

        assert_ne!(number, string);
        assert_eq!(number, new_num(42));
        assert_ne!(number, float);
        assert_ne!(number, boolean);

        assert_ne!(float, string);
        assert_ne!(float, number);
        assert_eq!(float, new_float("3.14"));
        assert_ne!(float, boolean);

        assert_ne!(boolean, string);
        assert_ne!(boolean, number);
        assert_ne!(boolean, float);
        assert_eq!(boolean, new_bool(true));
    }
}
