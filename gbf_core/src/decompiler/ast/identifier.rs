#![deny(missing_docs)]

use gbf_macros::AstNodeTransform;
use serde::{Deserialize, Serialize};

use super::{
    assignable::AssignableKind, expr::ExprKind, ptr::P, ssa::SsaVersion, visitors::AstVisitor,
    AstKind, AstVisitable,
};

/// Represents a type of literal.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, AstNodeTransform)]
#[convert_to(ExprKind::Assignable, AstKind::Expression, AssignableKind::Identifier)]
pub struct IdentifierNode {
    id: String,
    /// Represents the SSA version of a variable.
    pub ssa_version: Option<SsaVersion>,
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
        Self {
            id: s.into(),
            ssa_version: None,
        }
    }

    /// Creates a new `IdentifierNode` with an SSA version from any type that can be converted into a `String`.
    /// This is useful for creating an `IdentifierNode` with an SSA version.
    ///
    /// # Arguments
    /// - `s`: The input string-like type.
    /// - `ssa_version`: The SSA version of the identifier.
    ///
    /// # Returns
    /// - An `IdentifierNode` instance containing the provided identifier and SSA version.
    pub fn with_ssa<S: Into<String>>(s: S, ssa_version: SsaVersion) -> Self {
        Self {
            id: s.into(),
            ssa_version: Some(ssa_version),
        }
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

impl AstVisitable for IdentifierNode {
    fn accept(&self, visitor: &mut dyn AstVisitor) {
        visitor.visit_identifier(self);
    }
}

#[cfg(test)]
mod tests {
    use crate::decompiler::ast::{emit, new_id};

    #[test]
    fn test_identifier_emit() {
        let id = new_id("test");
        assert_eq!(emit(id), "test");
    }

    #[test]
    fn test_identifier_equality() {
        let id1 = new_id("test");
        let id2 = new_id("test");
        assert_eq!(id1, id2);

        let id3 = new_id("test2");
        assert_ne!(id1, id3);
    }
}
