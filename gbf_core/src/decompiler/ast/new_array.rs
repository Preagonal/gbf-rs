#![deny(missing_docs)]

use gbf_macros::AstNodeTransform;
use serde::{Deserialize, Serialize};

use super::{expr::ExprKind, ptr::P, visitors::AstVisitor, AstKind, AstVisitable};

/// Represents a return node in the AST, such as `return 5`.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, AstNodeTransform)]
#[convert_to(ExprKind::NewArray, AstKind::Expression)]
pub struct NewArrayNode {
    /// The number of elements in the array
    pub arg: ExprKind,
}

impl NewArrayNode {
    /// Creates a new `NewNode` with the provided type and arguments.
    ///
    /// # Arguments
    /// - `arg`: The number of elements in the array
    ///
    /// # Returns
    /// - A `NewNode` instance containing the provided type and arguments.
    pub fn new(arg: ExprKind) -> Self {
        Self { arg }
    }
}

impl AstVisitable for P<NewArrayNode> {
    fn accept<V: AstVisitor>(&self, visitor: &mut V) -> V::Output {
        visitor.visit_new_array(self)
    }
}

// == Other implementations for return ==
impl PartialEq for NewArrayNode {
    fn eq(&self, other: &Self) -> bool {
        self.arg == other.arg
    }
}
