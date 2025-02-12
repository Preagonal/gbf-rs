#![deny(missing_docs)]

use gbf_macros::AstNodeTransform;
use serde::{Deserialize, Serialize};

use super::{
    assignable::AssignableKind, expr::ExprKind, literal::LiteralNode, ptr::P, visitors::AstVisitor,
    AstKind, AstNodeError, AstVisitable,
};

/// Represents a return node in the AST, such as `return 5`.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, AstNodeTransform)]
#[convert_to(ExprKind::New, AstKind::Expression)]
pub struct NewNode {
    /// The type of object to create.
    pub new_type: ExprKind,
    /// The arguments to pass to the object.
    pub arg: ExprKind,
}

impl NewNode {
    /// Creates a new `NewNode` with the provided type and arguments.
    ///
    /// # Arguments
    /// - `new_type`: The type of object to create.
    /// - `arg`: The arguments to pass to the object.
    ///
    /// # Returns
    /// - A `NewNode` instance containing the provided type and arguments.
    pub fn new(new_type: ExprKind, arg: ExprKind) -> Result<Self, AstNodeError> {
        Self::validate_operand(&arg)?;
        Ok(Self { new_type, arg })
    }

    fn validate_operand(expr: &ExprKind) -> Result<(), AstNodeError> {
        // String literals and identifiers are ok.
        if let ExprKind::Literal(lit) = expr {
            if let LiteralNode::String(_) = lit.as_ref() {
                return Ok(());
            }
        }
        if let ExprKind::Assignable(AssignableKind::Identifier(_)) = expr {
            return Ok(());
        }

        Err(AstNodeError::InvalidOperand)
    }
}

impl AstVisitable for P<NewNode> {
    fn accept<V: AstVisitor>(&self, visitor: &mut V) -> V::Output {
        visitor.visit_new(self)
    }
}

// == Other implementations for return ==
impl PartialEq for NewNode {
    fn eq(&self, other: &Self) -> bool {
        self.new_type == other.new_type && self.arg == other.arg
    }
}

// #[cfg(test)]
// mod tests {
//     use crate::decompiler::ast::{emit, new_num, new_return};

//     #[test]
//     fn test_return_node() {
//         let ret = new_return(new_num(5));
//         assert_eq!(ret.ret, new_return(new_num(5)).ret);
//     }

//     #[test]
//     fn test_emit() {
//         let ret = new_return(new_num(5));
//         assert_eq!(emit(ret), "return 5;");
//     }

//     #[test]
//     fn test_equality() {
//         let ret = new_return(new_num(5));
//         let ret2 = new_return(new_num(5));
//         let ret3 = new_return(new_num(6));
//         assert_eq!(ret, ret2);
//         assert_ne!(ret, ret3);
//     }
// }
