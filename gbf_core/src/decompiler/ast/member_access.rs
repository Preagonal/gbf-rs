#![deny(missing_docs)]

use gbf_macros::AstNodeTransform;
use serde::{Deserialize, Serialize};

use super::{
    assignable::AssignableKind, expr::ExprKind, ssa::SsaVersion, visitors::AstVisitor, AstKind,
    AstNodeError,
};
use crate::decompiler::ast::AstVisitable;

/// Represents a member access node in the AST, such as `object.field`.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, AstNodeTransform)]
#[convert_to(
    ExprKind::Assignable,
    AstKind::Expression,
    AssignableKind::MemberAccess
)]
pub struct MemberAccessNode {
    /// The left-hand side of the member access, such as `object`.
    pub lhs: Box<AssignableKind>,
    /// The right-hand side of the member access, such as `field`.
    pub rhs: Box<AssignableKind>,
    /// Represents the SSA version of a variable.
    pub ssa_version: Option<SsaVersion>,
}

impl MemberAccessNode {
    /// Creates a new `MemberAccessNode` after validating `lhs` and `rhs` types.
    ///
    /// # Arguments
    /// - `lhs` - The left-hand side of the member access.
    /// - `rhs` - The right-hand side of the member access.
    ///
    /// # Returns
    /// A new `MemberAccessNode`.
    ///
    /// # Errors
    /// Returns an `AstNodeError` if `lhs` or `rhs` is of an unsupported type.
    pub fn new(lhs: Box<AssignableKind>, rhs: Box<AssignableKind>) -> Result<Self, AstNodeError> {
        let mut new_lhs = lhs.clone();
        let mut new_rhs = rhs.clone();
        Self::validate_and_strip_operand(&mut new_lhs)?;
        Self::validate_and_strip_operand(&mut new_rhs)?;

        Ok(Self {
            lhs: new_lhs,
            rhs: new_rhs,
            ssa_version: None,
        })
    }

    // This is marked as unreachable because the only two types of operands are Identifier and MemberAccess.
    // In the future, if more types are added, this function will need to be updated, especially for array.
    // This also removes the SsaVersion from the operands.
    #[allow(unreachable_patterns)]
    fn validate_and_strip_operand(expr: &mut AssignableKind) -> Result<(), AstNodeError> {
        expr.remove_ssa_version();
        match expr {
            AssignableKind::Identifier(_) | AssignableKind::MemberAccess(_) => Ok(()),
            _ => Err(AstNodeError::InvalidOperand(
                "MemberAccessNode".to_string(),
                "Unsupported operand type".to_string(),
                vec!["IdentifierNode".to_string(), "MemberAccessNode".to_string()],
                format!("{:?}", expr),
            )),
        }
    }
}

impl AstVisitable for MemberAccessNode {
    fn accept(&self, visitor: &mut dyn AstVisitor) {
        visitor.visit_member_access(self);
    }
}

// == Other implementations for member access ==
impl PartialEq for MemberAccessNode {
    fn eq(&self, other: &Self) -> bool {
        self.lhs == other.lhs && self.rhs == other.rhs
    }
}

#[cfg(test)]
mod tests {
    use crate::decompiler::ast::{emit, new_member_access, new_id, AstNodeError};

    #[test]
    fn test_member_access_emit() -> Result<(), AstNodeError> {
        let member = new_member_access(new_id("object"), new_id("field"))?;
        assert_eq!(emit(member), "object.field");
        Ok(())
    }

    #[test]
    fn test_member_access_nested_emit() -> Result<(), AstNodeError> {
        let member = new_member_access(
            new_member_access(new_id("object"), new_id("field"))?,
            new_id("other"),
        )?;
        assert_eq!(emit(member), "object.field.other");
        Ok(())
    }

    #[test]
    fn test_member_access_equality() -> Result<(), AstNodeError> {
        let member1 = new_member_access(new_id("object"), new_id("field"))?;
        let member2 = new_member_access(new_id("object"), new_id("field"))?;
        assert_eq!(member1, member2);

        let member3 = new_member_access(new_id("object"), new_id("other"))?;
        assert_ne!(member1, member3);
        Ok(())
    }
}
