#![deny(missing_docs)]

use gbf_macros::AstNodeTransform;
use serde::{Deserialize, Serialize};

use super::{
    expr::ExprKind, ptr::P, statement::StatementKind, visitors::AstVisitor, AstKind, AstVisitable,
};

/// Represents a return node in the AST, such as `return 5`.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, AstNodeTransform)]
#[convert_to(StatementKind::Return, AstKind::Statement)]
pub struct ReturnNode {
    /// The value to return.
    pub ret: ExprKind,
}

impl ReturnNode {
    /// Creates a new return node.
    ///
    /// # Arguments
    /// - `ret`: The value to return.
    ///
    /// # Returns
    /// The return node.
    pub fn new(ret: ExprKind) -> Self {
        Self { ret }
    }
}

impl AstVisitable for ReturnNode {
    fn accept(&self, visitor: &mut dyn AstVisitor) {
        visitor.visit_return(self);
    }
}

// == Other implementations for return ==
impl PartialEq for ReturnNode {
    fn eq(&self, other: &Self) -> bool {
        self.ret == other.ret
    }
}

#[cfg(test)]
mod tests {
    use crate::decompiler::ast::{emit, new_num, new_return};

    #[test]
    fn test_return_node() {
        let ret = new_return(new_num(5));
        assert_eq!(ret.ret, new_return(new_num(5)).ret);
    }

    #[test]
    fn test_emit() {
        let ret = new_return(new_num(5));
        assert_eq!(emit(ret), "return 5;");
    }

    #[test]
    fn test_equality() {
        let ret = new_return(new_num(5));
        let ret2 = new_return(new_num(5));
        let ret3 = new_return(new_num(6));
        assert_eq!(ret, ret2);
        assert_ne!(ret, ret3);
    }
}
