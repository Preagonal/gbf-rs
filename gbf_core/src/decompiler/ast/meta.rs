#![deny(missing_docs)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a metadata node in the AST
#[derive(Debug, Clone, Serialize, Deserialize, Eq, Default)]
pub struct Metadata {
    comments: Vec<String>,
    properties: HashMap<String, String>,
}

impl Metadata {
    /// Returns the comments.
    pub fn comments(&self) -> &Vec<String> {
        &self.comments
    }

    /// Adds a new comment
    pub fn add_comment(&mut self, comment: String) {
        self.comments.push(comment);
    }

    /// Adds a new property
    pub fn add_property(&mut self, key: String, value: String) {
        self.properties.insert(key, value);
    }

    /// Gets a property
    pub fn get_property(&self, key: &str) -> Option<&String> {
        self.properties.get(key)
    }

    /// Returns the properties.
    pub fn properties(&self) -> &HashMap<String, String> {
        &self.properties
    }
}

impl PartialEq for Metadata {
    fn eq(&self, other: &Self) -> bool {
        self.comments == other.comments && self.properties == other.properties
    }
}

#[cfg(test)]
mod tests {
    use crate::decompiler::ast::{
        assignment::AssignmentNode, emit, new_assignment, new_id, new_if, ptr::P,
    };

    use super::*;

    #[test]
    fn test_metadata() {
        let mut metadata = Metadata::default();
        metadata.add_comment("This is a comment".to_string());
        metadata.add_property("key".to_string(), "value".to_string());

        assert_eq!(metadata.comments(), &vec!["This is a comment".to_string()]);
        assert_eq!(metadata.get_property("key"), Some(&"value".to_string()));
    }

    #[test]
    fn test_attach_metadata() {
        let mut stmt: P<AssignmentNode> = new_assignment(new_id("foo"), new_id("bar")).into();
        stmt.metadata_mut()
            .add_comment("This is a comment".to_string());

        // Create if stmt
        let if_node = new_if(new_id("test"), vec![stmt]);

        let emitted = emit(if_node);
        assert_eq!(
            emitted,
            "if (test) \n{\n    // This is a comment\n    foo = bar;\n}"
        );
    }
}
