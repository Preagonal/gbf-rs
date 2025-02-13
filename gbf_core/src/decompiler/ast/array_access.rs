#![deny(missing_docs)]

use gbf_macros::AstNodeTransform;
use serde::{Deserialize, Serialize};

use super::{expr::ExprKind, ptr::P, ssa::SsaVersion, visitors::AstVisitor, AstKind, AstVisitable};

/// Represents a function call
#[derive(Debug, Clone, Serialize, Deserialize, Eq, AstNodeTransform)]
#[convert_to(ExprKind::ArrayAccess, AstKind::Expression)]
pub struct ArrayAccessNode {
    /// The array to access.
    pub arr: ExprKind,

    /// The index to access.
    pub index: ExprKind,

    /// Represents the SSA version of a variable.
    pub ssa_version: Option<SsaVersion>,
}

impl ArrayAccessNode {
    /// Creates a new array access.
    ///
    /// # Arguments
    /// - `arr`: The array to access.
    /// - `index`: The index to access.
    pub fn new(arr: ExprKind, index: ExprKind) -> Self {
        Self {
            arr,
            index,
            ssa_version: None,
        }
    }
}

impl AstVisitable for P<ArrayAccessNode> {
    fn accept<V: AstVisitor>(&self, visitor: &mut V) -> V::Output {
        visitor.visit_array_access(self)
    }
}

// == Other implementations for unary operations ==
impl PartialEq for ArrayAccessNode {
    fn eq(&self, other: &Self) -> bool {
        self.arr == other.arr && self.index == other.index
    }
}

#[cfg(test)]
mod tests {
    use crate::decompiler::ast::{new_array_access, new_id, new_num};

    #[test]
    fn test_array_access_node() {
        let arr = new_id("arr");
        let index = new_num(5);
        let array_access = new_array_access(arr.clone(), index.clone());
        let array_access_two = new_array_access(arr, index);
        assert_eq!(array_access, array_access_two);
    }

    #[test]
    fn test_array_access_node_emit() {
        let arr = new_id("arr");
        let index = new_num(5);
        let array_access = new_array_access(arr, index);
        assert_eq!(crate::decompiler::ast::emit(array_access), "arr[5]");
    }
}
