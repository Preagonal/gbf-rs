#![deny(missing_docs)]

use std::sync::atomic::{AtomicUsize, Ordering};

use serde::{Deserialize, Serialize};

/// A unique identifier for an AST node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(usize);

static NEXT_NODE_ID: AtomicUsize = AtomicUsize::new(0);

impl NodeId {
    /// Generates a new, unique NodeId.
    ///
    /// This uses a global atomic counter to ensure that each NodeId is unique.
    pub fn new() -> Self {
        // Using Relaxed ordering is fine here because we only need atomic uniqueness,
        // not synchronization with other operations.
        let id = NEXT_NODE_ID.fetch_add(1, Ordering::Relaxed);
        NodeId(id)
    }

    /// Returns the underlying numeric id.
    pub fn get(self) -> usize {
        self.0
    }
}

impl Default for NodeId {
    fn default() -> Self {
        NodeId::new()
    }
}
