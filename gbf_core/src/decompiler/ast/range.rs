#![deny(missing_docs)]

use gbf_macros::AstNodeTransform;
use serde::{Deserialize, Serialize};

use super::{expr::ExprKind, ptr::P, visitors::AstVisitor, AstKind, AstVisitable};

/// Represents a range node in the AST, such as <1, 5>.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, AstNodeTransform)]
#[convert_to(ExprKind::Range, AstKind::Expression)]
pub struct RangeNode {
    /// The start of the range
    pub start: ExprKind,
    /// The end of the range
    pub end: ExprKind,
}

impl RangeNode {
    /// Creates a new `RangeNode` with the provided start and end expressions.
    ///
    /// # Arguments
    /// - `start`: The start of the range
    /// - `end`: The end of the range
    ///
    /// # Returns
    /// - A `RangeNode` instance containing the provided start and end expressions.
    pub fn new(start: ExprKind, end: ExprKind) -> Self {
        Self { start, end }
    }
}

impl AstVisitable for P<RangeNode> {
    fn accept<V: AstVisitor>(&self, visitor: &mut V) -> V::Output {
        visitor.visit_range(self)
    }
}

// == Other implementations for range operations ==
impl PartialEq for RangeNode {
    fn eq(&self, other: &Self) -> bool {
        self.start == other.start && self.end == other.end
    }
}
