#![deny(missing_docs)]

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::utils::Gs2BytecodeAddress;

use super::{AstNode, AstNodeTrait};

/// Represents a metadata node in the AST
#[derive(Debug, Clone, Serialize, Deserialize, Eq)]
pub struct MetaNode {
    node: Box<AstNode>,
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
        node: Box<AstNode>,
        comment: Option<String>,
        source_location: Option<Gs2BytecodeAddress>,
        properties: HashMap<String, String>,
    ) -> Box<Self> {
        Box::new(Self {
            node,
            comment,
            source_location,
            properties,
        })
    }

    /// Returns the inner node.
    pub fn node(&self) -> &AstNode {
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
impl AstNodeTrait for MetaNode {
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
    use super::*;
    use crate::decompiler::ast::expr::ExprNode;
    use crate::decompiler::ast::literal::LiteralNode;

    fn create_test_node() -> MetaNode {
        let node: AstNode = AstNode::Expression(Box::new(ExprNode::Literal(LiteralNode::String(
            "inner_node".to_string(),
        ))));
        MetaNode {
            node: Box::new(node),
            comment: Some("This is a comment".to_string()),
            source_location: Some(0x1234 as Gs2BytecodeAddress),
            properties: HashMap::from([
                ("key1".to_string(), "value1".to_string()),
                ("key2".to_string(), "value2".to_string()),
            ]),
        }
    }

    #[test]
    fn test_partial_eq_same_node() {
        let node1 = create_test_node();
        let node2 = create_test_node();

        assert_eq!(node1, node2);
    }

    #[test]
    fn test_partial_eq_different_nodes() {
        let mut node1 = create_test_node();
        let mut node2 = create_test_node();

        // Modify `node2`'s properties
        node2
            .properties
            .insert("key3".to_string(), "value3".to_string());

        assert_ne!(node1, node2);

        // Modify `node1`'s comment
        node1.comment = Some("Different comment".to_string());
        assert_ne!(node1, node2);
    }

    #[test]
    fn test_source_location_preservation() {
        let node = create_test_node();

        assert_eq!(node.source_location, Some(0x1234 as Gs2BytecodeAddress));
    }

    #[test]
    fn test_properties_access() {
        let node = create_test_node();

        assert_eq!(node.properties.get("key1"), Some(&"value1".to_string()));
        assert_eq!(node.properties.get("key2"), Some(&"value2".to_string()));
        assert!(!node.properties.contains_key("key3"));
    }

    #[test]
    fn test_getters() {
        let node = create_test_node();

        assert_eq!(
            node.node(),
            &AstNode::Expression(Box::new(ExprNode::Literal(LiteralNode::String(
                "inner_node".to_string()
            ))))
        );
        assert_eq!(node.comment(), Some(&"This is a comment".to_string()));
        assert_eq!(node.source_location(), Some(0x1234 as Gs2BytecodeAddress));
        assert_eq!(
            node.properties(),
            &HashMap::from([
                ("key1".to_string(), "value1".to_string()),
                ("key2".to_string(), "value2".to_string()),
            ])
        );
    }
}
