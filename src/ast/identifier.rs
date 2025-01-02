#![deny(missing_docs)]

use serde::{Deserialize, Serialize};

use super::{
    emit::{EmitContext, EmitError},
    AstNodeTrait,
};

/// Represents a type of literal.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct IdentifierNode(String);

// == Other implementations for literal ==

impl AstNodeTrait for IdentifierNode {
    fn emit(&self, _: &EmitContext) -> Result<String, EmitError> {
        Ok(self.0.clone())
    }
}
