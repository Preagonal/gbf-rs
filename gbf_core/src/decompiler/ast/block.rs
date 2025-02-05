#![deny(missing_docs)]

use gbf_macros::AstNodeTransform;
use serde::{Deserialize, Serialize};

use super::{ptr::P, visitors::AstVisitor, AstKind, AstVisitable};

/// Represents a function call
#[derive(Debug, Clone, Serialize, Deserialize, Eq, AstNodeTransform)]
#[convert_to(AstKind::Block)]
pub struct BlockNode {
    /// The instructions in the block.
    pub instructions: Vec<AstKind>,
}

impl BlockNode {
    /// Creates a new block node.
    ///
    /// # Arguments
    /// - `instructions`: The instructions in the block.
    pub fn new<V>(instructions: Vec<V>) -> Self
    where
        V: Into<AstKind>,
    {
        Self {
            instructions: instructions.into_iter().map(Into::into).collect(),
        }
    }
}

impl AstVisitable for P<BlockNode> {
    fn accept<V: AstVisitor>(&self, visitor: &mut V) -> V::Output {
        visitor.visit_block(self)
    }
}

// == Other implementations for unary operations ==
impl PartialEq for BlockNode {
    fn eq(&self, other: &Self) -> bool {
        self.instructions == other.instructions
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decompiler::ast::{emit, new_assignment, new_float, new_id};

    #[test]
    fn test_block_node() {
        let stmt_1 = new_assignment(new_id("foo"), new_id("bar"));
        let stmt_2 = new_assignment(new_id("baz"), new_float("3.14"));
        let block = BlockNode::new(vec![stmt_1, stmt_2]);
        assert_eq!(block.instructions.len(), 2);
    }

    #[test]
    fn test_block_emit() {
        let stmt_1 = new_assignment(new_id("foo"), new_id("bar"));
        let stmt_2 = new_assignment(new_id("baz"), new_float("3.14"));
        let block = BlockNode::new(vec![stmt_1, stmt_2]);
        let output = emit(block);
        assert_eq!(output, "\n{\n    foo = bar;\n    baz = 3.14;\n}");
    }
}
