#![deny(missing_docs)]

use gbf_macros::AstNodeTransform;
use serde::{Deserialize, Serialize};

use crate::decompiler::structure_analysis::region::RegionId;

use super::{ptr::P, statement::StatementKind, visitors::AstVisitor, AstKind, AstVisitable};

/// Represents a unary operation node in the AST, such as `-a` or `!b`.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, AstNodeTransform)]
#[convert_to(StatementKind::VirtualBranch, AstKind::Statement)]
pub struct VirtualBranchNode {
    /// The branch to jump to.
    pub branch: RegionId,
}

impl VirtualBranchNode {
    /// Creates a new `VirtualBranchNode` with the provided branch.
    pub fn new(branch: RegionId) -> Self {
        Self { branch }
    }

    /// Returns the branch to jump to.
    pub fn branch(&self) -> RegionId {
        self.branch
    }
}

impl AstVisitable for P<VirtualBranchNode> {
    fn accept<V: AstVisitor>(&self, visitor: &mut V) -> V::Output {
        visitor.visit_virtual_branch(self)
    }
}

// == Other implementations for unary operations ==
impl PartialEq for VirtualBranchNode {
    fn eq(&self, other: &Self) -> bool {
        self.branch == other.branch
    }
}
