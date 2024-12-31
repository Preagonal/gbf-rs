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
