#![deny(missing_docs)]

use gbf_macros::AstNodeTransform;
use serde::{Deserialize, Serialize};

use super::{ast_vec::AstVec, block::BlockNode, expr::ExprKind, AstKind, AstVisitable};

/// Represents a metadata node in the AST
#[derive(Debug, Clone, Serialize, Deserialize, Eq, AstNodeTransform)]
#[convert_to(AstKind::Function)]
pub struct FunctionNode {
    name: Option<String>,
    params: AstVec<ExprKind>,
    body: BlockNode,
}

impl FunctionNode {
    /// Creates a new `FunctionNode` with the given `params` and `body`.
    ///
    /// # Arguments
    /// - `name` - The name of the function.
    /// - `params` - The parameters of the function.
    /// - `body` - The body of the function.
    ///
    /// # Returns
    /// A new `FunctionNode`.
    pub fn new<N, V>(name: N, params: AstVec<ExprKind>, body: V) -> Self
    where
        N: Into<Option<String>>,
        V: Into<AstVec<AstKind>>,
    {
        Self {
            name: name.into(),
            params,
            body: BlockNode::new(body),
        }
    }

    /// Returns the parameters of the function.
    pub fn params(&self) -> &Vec<ExprKind> {
        &self.params
    }

    /// Returns the body of the function.
    pub fn body(&self) -> &BlockNode {
        &self.body
    }

    /// Returns the name of the function.
    pub fn name(&self) -> &Option<String> {
        &self.name
    }
}

// == Other implementations for literal ==
impl AstVisitable for FunctionNode {
    fn accept(&self, visitor: &mut dyn super::visitors::AstVisitor) {
        visitor.visit_function(self);
    }
}

impl PartialEq for FunctionNode {
    fn eq(&self, other: &Self) -> bool {
        self.params == other.params && self.body == other.body
    }
}
