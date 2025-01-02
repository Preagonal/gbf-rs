#![deny(missing_docs)]

use serde::{Deserialize, Serialize};

use super::{
    emit::{EmitContext, EmitError},
    AstNodeTrait,
};

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
}

// == Other implementations for literal ==

impl AstNodeTrait for IdentifierNode {
    fn emit(&self, _: &EmitContext) -> Result<String, EmitError> {
        Ok(self.id.clone())
    }
}
