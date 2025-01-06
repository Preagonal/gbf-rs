#![deny(missing_docs)]

use std::fmt::Display;

use super::ast::{expr::ExprNode, AstNode};

/// Represents the state of execution for the decompiler.
#[derive(Debug, PartialEq, Clone)]
pub enum ExecutionFrame {
    /// The decompiler is currently building a standalone node.
    StandaloneNode(AstNode),
    /// The decompiler is currently building an array.
    BuildingArray(Vec<ExprNode>),
    /// The decompiler is not building any partial construct.
    None,
}

impl Display for ExecutionFrame {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExecutionFrame::BuildingArray(_) => write!(f, "BuildingArray"),
            ExecutionFrame::None => write!(f, "None"),
            ExecutionFrame::StandaloneNode(_) => write!(f, "StandaloneNode"),
        }
    }
}
