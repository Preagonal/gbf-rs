#![deny(missing_docs)]

use serde::{Deserialize, Serialize};

use super::{visitors::AstVisitor, AstNodeTrait};

/// Represents a type of literal.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct IdentifierNode {
    id: String,
}

impl IdentifierNode {
    /// Creates a new `IdentifierNode` from any type that can be converted into a `String`.
    ///
    /// # Arguments
    /// - `s`: The input string-like type.
    ///
    /// # Returns
    /// - An `IdentifierNode` instance containing the provided identifier.
    pub fn new<S: Into<String>>(s: S) -> Self {
        Self { id: s.into() }
    }

    /// Returns the identifier as a reference to a `String`.
    pub fn id(&self) -> &String {
        &self.id
    }

    /// Returns the identifier as a `&str`.
    pub fn as_str(&self) -> &str {
        &self.id
    }
}

// == Other implementations for literal ==

impl AstNodeTrait for IdentifierNode {
    fn accept(&self, visitor: &mut dyn AstVisitor) {
        visitor.visit_identifier(self);
    }
}
