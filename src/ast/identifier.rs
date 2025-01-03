#![deny(missing_docs)]

use serde::{Deserialize, Serialize};

use super::{visitors::AstVisitor, AstNodeTrait};

/// Represents a type of literal.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
// Identifiers are usually short so we don't need to box them.
pub struct IdentifierNode {
    id: String,
}

impl IdentifierNode {
    /// Creates a new `IdentifierNode` from a string.
    pub fn new(s: String) -> Box<Self> {
        Box::new(Self { id: s })
    }

    /// Returns the identifier.
    pub fn id(&self) -> &String {
        &self.id
    }
}

// == Other implementations for literal ==

impl AstNodeTrait for IdentifierNode {
    fn accept(&self, visitor: &mut dyn AstVisitor) {
        visitor.visit_identifier(self);
    }
}
