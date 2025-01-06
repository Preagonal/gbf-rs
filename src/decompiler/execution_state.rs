#![deny(missing_docs)]

use std::fmt::Display;

use super::ast::expr::ExprNode;

/// Represents the state of execution for the decompiler.
#[derive(Debug, PartialEq)]
pub enum ExecutionState {
    /// The decompiler is currently building an array.
    BuildingArray(Vec<ExprNode>),
    /// The decompiler is not building any partial construct.
    None,
}

impl Display for ExecutionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExecutionState::BuildingArray(_) => write!(f, "BuildingArray"),
            ExecutionState::None => write!(f, "None"),
        }
    }
}
