#![deny(missing_docs)]

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::utils::Gs2BytecodeAddress;

use super::{
    emit::{EmitContext, EmitVerbosity},
    AstNode, AstNodeTrait,
};

/// Represents a metadata node in the AST
#[derive(Debug, Clone, Serialize, Deserialize, Eq)]
pub struct MetaNode {
    /// The child node of the metadata node.
    node: Box<AstNode>,
    comment: Option<String>,
    source_location: Option<Gs2BytecodeAddress>,
    properties: HashMap<String, String>,
}

// == Other implementations for literal ==
impl AstNodeTrait for MetaNode {
    fn emit(&self, ctx: &EmitContext) -> String {
        if ctx.verbosity == EmitVerbosity::Minified {
            return self.node.emit(ctx);
        }

        let mut result = String::new();
        if let Some(comment) = &self.comment {
            result.push_str(&format!("// {}\n", comment));
        }
        result.push_str(&self.node.emit(ctx));
        result
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
    use crate::{
        ast::literal::{LiteralNode, LiteralType},
        utils::Gs2BytecodeAddress,
    };

    fn create_test_node() -> MetaNode {
        let node: AstNode = AstNode::Literal(LiteralNode {
            value: LiteralType::String("inner_node".to_string()),
        });
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
    fn test_emit_with_pretty_verbosity() {
        let context = EmitContext::builder()
            .verbosity(EmitVerbosity::Pretty)
            .build();
        let node = create_test_node();

        let output = node.emit(&context);
        assert_eq!(output, "// This is a comment\n\"inner_node\"");
    }

    #[test]
    fn test_emit_with_minified_verbosity() {
        let context = EmitContext::builder()
            .verbosity(EmitVerbosity::Minified)
            .build();
        let node = create_test_node();

        let output = node.emit(&context);
        assert_eq!(output, "\"inner_node\"");
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
}
