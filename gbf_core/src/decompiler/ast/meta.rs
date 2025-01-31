#![deny(missing_docs)]

use std::collections::HashMap;

use gbf_macros::AstNodeTransform;
use serde::{Deserialize, Serialize};

use crate::utils::Gs2BytecodeAddress;

use super::{ptr::P, AstKind, AstVisitable};

/// Represents a metadata node in the AST
#[derive(Debug, Clone, Serialize, Deserialize, Eq, AstNodeTransform)]
#[convert_to(AstKind::Meta)]
pub struct MetaNode {
    node: P<AstKind>,
    comment: Option<String>,
    source_location: Option<Gs2BytecodeAddress>,
    properties: HashMap<String, String>,
}

impl MetaNode {
    /// Creates a new `MetaNode` with the given `node`, `comment`, `source_location`, and `properties`.
    ///
    /// # Arguments
    /// - `node` - The inner node.
    /// - `comment` - The comment for the node.
    /// - `source_location` - The source location of the node.
    /// - `properties` - The properties of the node.
    ///
    /// # Returns
    /// A new `MetaNode`.
    pub fn new(
        node: P<AstKind>,
        comment: Option<String>,
        source_location: Option<Gs2BytecodeAddress>,
        properties: HashMap<String, String>,
    ) -> Self {
        Self {
            node,
            comment,
            source_location,
            properties,
        }
    }

    /// Returns the inner node.
    pub fn node(&self) -> &AstKind {
        &self.node
    }

    /// Returns the comment.
    pub fn comment(&self) -> Option<&String> {
        self.comment.as_ref()
    }

    /// Returns the source location.
    pub fn source_location(&self) -> Option<Gs2BytecodeAddress> {
        self.source_location
    }

    /// Returns the properties.
    pub fn properties(&self) -> &HashMap<String, String> {
        &self.properties
    }
}

// == Other implementations for literal ==
impl AstVisitable for MetaNode {
    fn accept(&self, visitor: &mut dyn super::visitors::AstVisitor) {
        visitor.visit_meta(self);
    }
}

impl PartialEq for MetaNode {
    fn eq(&self, other: &Self) -> bool {
        self.node == other.node
            && self.comment == other.comment
            && self.source_location == other.source_location
            && self.properties == other.properties
    }
}

#[cfg(test)]
mod tests {
    use crate::decompiler::ast::{emit, new_assignment, new_comment, new_id};

    #[test]
    fn test_comment_emit() {
        let statement = new_assignment(new_id("foo"), new_id("bar"));
        let comment = new_comment(statement, "Sets foo to bar");
        assert_eq!(emit(comment), "// Sets foo to bar\nfoo = bar;");
    }

    #[test]
    fn test_comment_equality() {
        let statement = new_assignment(new_id("foo"), new_id("bar"));
        let comment1 = new_comment(statement.clone(), "Sets foo to bar");
        let comment2 = new_comment(statement.clone(), "Sets foo to bar");
        let comment3 = new_comment(statement.clone(), "Sets foo to baz");

        assert_eq!(comment1, comment2);
        assert_ne!(comment1, comment3);
    }
}
