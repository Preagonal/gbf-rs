#![deny(missing_docs)]

use std::fmt::Display;

use serde::{Deserialize, Serialize};

use super::ast::{expr::ExprKind, AstKind};

/// Represents the state of execution for the decompiler.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum ExecutionFrame {
    /// The decompiler is currently building a standalone node.
    StandaloneNode(AstKind),
    /// The decompiler is currently building an array.
    BuildingArray(Vec<ExprKind>),
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
