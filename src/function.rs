#![deny(missing_docs)]

use std::fmt;
use std::{collections::HashMap, hash::Hash};

use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::Direction;
use thiserror::Error;

use crate::basic_block::{BasicBlock, BasicBlockId, BasicBlockType};

/// Represents an error that can occur when working with functions.
#[derive(Error, Debug)]
pub enum FunctionError {
    /// The requested `BasicBlock` was not found.
    #[error("BasicBlock not found: {0}")]
    BasicBlockNotFound(BasicBlockId),

    /// The requested `BasicBlock` does not have a `NodeId`.
    #[error("BasicBlock with id {0} does not have a NodeId")]
    BasicBlockNodeIdNotFound(BasicBlockId),

    /// The function already has an entry block.
    #[error("Function already has an entry block")]
    EntryBlockAlreadyExists,
}

/// Represents the identifier of a function.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FunctionId {
    index: usize,
    name: Option<String>,
    offset: u64,
}

impl fmt::Display for FunctionId {
    /// Display the `Function` as `Function{index}`.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Function{}", self.index)
    }
}

impl FunctionId {
    /// Create a new `FunctionId`.
    ///
    /// # Arguments
    /// - `index`: The index of the function in the module.
    /// - `name`: The name of the function, if it is not the entry point.
    /// - `offset`: The offset of the function in the module.
    ///
    /// # Returns
    /// - A new `FunctionId` instance.
    ///
    /// # Example
    /// ```
    /// use gbf_rs::function::FunctionId;
    ///
    /// let entry = FunctionId::new(0, None, 0);
    /// let add = FunctionId::new(1, Some("add".to_string()), 0x100);
    /// ```
    pub fn new(index: usize, name: Option<String>, offset: u64) -> Self {
        Self {
            index,
            name,
            offset,
        }
    }

    /// If the function has a name.
    ///
    /// # Returns
    /// - `true` if the function has a name.
    /// - `false` if the function does not have a name.
    ///
    /// # Example
    /// ```
    /// use gbf_rs::function::FunctionId;
    ///
    /// let entry = FunctionId::new(0, None, 0);
    ///
    /// assert!(entry.is_named());
    /// ```
    pub fn is_named(&self) -> bool {
        self.name.is_none()
    }
}

/// Represents a function in a module.
#[derive(Debug)]
pub struct Function {
    /// The identifier of the function.
    pub id: FunctionId,
    entry_block: BasicBlockId,
    blocks: HashMap<BasicBlockId, BasicBlock>,

    // Our petgraph-based control-flow graph
    cfg: DiGraph<(), ()>,

    // used to convert NodeIndex to BasicBlockId
    node_to_block: HashMap<NodeIndex, BasicBlockId>,

    // used to convert BasicBlockId to NodeIndex
    block_to_node: HashMap<BasicBlockId, NodeIndex>,
}

impl Function {
    /// Create a new `Function`. Automatically creates an entry block.
    ///
    /// # Arguments
    /// - `id`: The `FunctionId` of the function.
    ///
    /// # Returns
    /// - A new `Function` instance.
    pub fn new(id: FunctionId) -> Self {
        let mut blocks = HashMap::new();
        let mut node_to_block: HashMap<NodeIndex, BasicBlockId> = HashMap::new();
        let mut block_to_node: HashMap<BasicBlockId, NodeIndex> = HashMap::new();
        let mut cfg = DiGraph::new();

        // Initialize entry block
        let entry_block = BasicBlockId::new(blocks.len(), BasicBlockType::Entry);
        blocks.insert(entry_block, BasicBlock::new(entry_block));

        // Add an empty node in the graph to represent this BasicBlock
        let entry_node_id = cfg.add_node(());

        node_to_block.insert(entry_node_id, entry_block);
        block_to_node.insert(entry_block, entry_node_id);

        Self {
            id,
            entry_block,
            blocks,
            cfg,
            node_to_block,
            block_to_node,
        }
    }

    /// Create a new `BasicBlock` and add it to the function.
    ///
    /// # Arguments
    /// - `block_type`: The type of the block.
    ///
    /// # Returns
    /// - A `BasicBlockId` for the new block.
    ///
    /// # Example
    /// ```
    /// use gbf_rs::function::{Function, FunctionId};
    /// use gbf_rs::basic_block::BasicBlockType;
    ///
    /// let mut function = Function::new(FunctionId::new(0, None, 0));
    /// let block = function.create_block(BasicBlockType::Normal);
    /// ```
    pub fn create_block(
        &mut self,
        block_type: BasicBlockType,
    ) -> Result<BasicBlockId, FunctionError> {
        // do not allow entry block to be created more than once
        if block_type == BasicBlockType::Entry {
            return Err(FunctionError::EntryBlockAlreadyExists);
        }

        let id = BasicBlockId::new(self.blocks.len(), block_type);
        self.blocks.insert(id, BasicBlock::new(id));

        // Insert a node in the petgraph to represent this BasicBlock
        let node_id = self.cfg.add_node(());

        self.block_to_node.insert(id, node_id);
        self.node_to_block.insert(node_id, id);

        Ok(id)
    }

    /// Convert a `NodeIndex` to a `BasicBlockId`.
    ///
    /// # Arguments
    /// - `node_id`: The `NodeIndex` to convert.
    ///
    /// # Returns
    /// - The `BasicBlockId` of the block with the corresponding `NodeId`.
    fn node_id_to_block_id(&self, node_id: NodeIndex) -> Option<BasicBlockId> {
        self.node_to_block.get(&node_id).cloned()
    }

    /// Convert a `BasicBlockId` to a `NodeIndex`.
    ///
    /// # Arguments
    /// - `block_id`: The `BasicBlockId` to convert.
    ///
    /// # Returns
    /// - The `NodeId` of the block with the corresponding `BasicBlockId`.
    fn block_id_to_node_id(&self, block_id: BasicBlockId) -> Option<NodeIndex> {
        self.block_to_node.get(&block_id).cloned()
    }

    /// Get a reference to a `BasicBlock` by its `BasicBlockId`.
    ///
    /// # Arguments
    /// - `id`: The `BasicBlockId` of the block.
    ///
    /// # Returns
    /// - A reference to the `BasicBlock`.
    ///
    /// # Errors
    /// - `FunctionError::BasicBlockNotFound` if the block does not exist.
    ///
    /// # Example
    /// ```
    /// use gbf_rs::function::{Function, FunctionId};
    /// use gbf_rs::basic_block::BasicBlockType;
    ///
    /// let mut function = Function::new(FunctionId::new(0, None, 0));
    /// let block_id = function.create_block(BasicBlockType::Normal).unwrap();
    /// let block_ref = function.get_block(block_id).unwrap();
    /// ```
    pub fn get_block(&self, id: BasicBlockId) -> Result<&BasicBlock, FunctionError> {
        self.blocks
            .get(&id)
            .ok_or(FunctionError::BasicBlockNotFound(id))
    }

    /// Get a mutable reference to a `BasicBlock` by its `BasicBlockId`.
    ///
    /// # Arguments
    /// - `id`: The `BasicBlockId` of the block.
    ///
    /// # Returns
    /// - A mutable reference to the `BasicBlock`.
    ///
    /// # Errors
    /// - `FunctionError::BasicBlockNotFound` if the block does not exist.
    ///
    /// # Example
    /// ```
    /// use gbf_rs::function::{Function, FunctionId};
    /// use gbf_rs::basic_block::BasicBlockType;
    ///
    /// let mut function = Function::new(FunctionId::new(0, None, 0));
    /// let block_id = function.create_block(BasicBlockType::Normal).unwrap();
    /// let block_ref = function.get_block_mut(block_id).unwrap();
    /// ```
    pub fn get_block_mut(&mut self, id: BasicBlockId) -> Result<&mut BasicBlock, FunctionError> {
        self.blocks
            .get_mut(&id)
            .ok_or(FunctionError::BasicBlockNotFound(id))
    }

    /// Get the entry block of the function.
    ///
    /// # Returns
    /// - A reference to the entry block.
    ///
    /// # Example
    /// ```
    /// use gbf_rs::function::{Function, FunctionId};
    ///
    /// let mut function = Function::new(FunctionId::new(0, None, 0));
    /// let entry = function.get_entry_block();
    /// ```
    pub fn get_entry_block(&self) -> &BasicBlock {
        self.get_block(self.entry_block).unwrap()
    }

    /// Get the entry block of the function.
    ///
    /// # Returns
    /// - A mutable reference to the entry block.
    ///
    /// # Example
    /// ```
    /// use gbf_rs::function::{Function, FunctionId};
    ///
    /// let mut function = Function::new(FunctionId::new(0, None, 0));
    /// let entry = function.get_entry_block_mut();
    /// ```
    pub fn get_entry_block_mut(&mut self) -> &mut BasicBlock {
        self.get_block_mut(self.entry_block).unwrap()
    }

    /// Add an edge between two `BasicBlock`s.
    ///
    /// # Arguments
    /// - `source`: The `BasicBlockId` of the source block.
    /// - `target`: The `BasicBlockId` of the target block.
    ///
    /// # Errors
    /// - `FunctionError::BasicBlockNodeIdNotFound` if either block does not have a `NodeId`.
    /// - `FunctionError::GraphError` if the edge could not be added to the graph.
    ///
    /// # Example
    /// ```
    /// use gbf_rs::function::{Function, FunctionId};
    /// use gbf_rs::basic_block::BasicBlockType;
    ///
    /// let mut function = Function::new(FunctionId::new(0, None, 0));
    /// let block1 = function.create_block(BasicBlockType::Normal).unwrap();
    /// let block2 = function.create_block(BasicBlockType::Normal).unwrap();
    /// function.add_edge(block1, block2);
    /// ```
    pub fn add_edge(
        &mut self,
        source: BasicBlockId,
        target: BasicBlockId,
    ) -> Result<(), FunctionError> {
        let source_node_id = self
            .block_id_to_node_id(source)
            .ok_or(FunctionError::BasicBlockNodeIdNotFound(source))?;
        let target_node_id = self
            .block_id_to_node_id(target)
            .ok_or(FunctionError::BasicBlockNodeIdNotFound(target))?;

        // With petgraph, this does not fail, so we simply do it:
        // It can panic if the node does not exist, but we have already checked that.
        self.cfg.add_edge(source_node_id, target_node_id, ());
        Ok(())
    }

    /// Get the predecessors of a `BasicBlock`.
    ///
    /// # Arguments
    /// - `id`: The `BasicBlockId` of the block.
    ///
    /// # Returns
    /// - A vector of `BasicBlockId`s that are predecessors of the block.
    ///
    /// # Errors
    /// - `FunctionError::BasicBlockNodeIdNotFound` if the block does not exist.
    /// - `FunctionError::GraphError` if the predecessors could not be retrieved from the graph.
    ///
    /// # Example
    /// ```
    /// use gbf_rs::function::{Function, FunctionId};
    /// use gbf_rs::basic_block::BasicBlockType;
    ///
    /// let mut function = Function::new(FunctionId::new(0, None, 0));
    /// let block1 = function.create_block(BasicBlockType::Normal).unwrap();
    /// let block2 = function.create_block(BasicBlockType::Normal).unwrap();
    ///
    /// function.add_edge(block1, block2);
    /// let preds = function.get_predecessors(block2).unwrap();
    /// ```
    pub fn get_predecessors(&self, id: BasicBlockId) -> Result<Vec<BasicBlockId>, FunctionError> {
        let node_id = self
            .block_id_to_node_id(id)
            .ok_or(FunctionError::BasicBlockNodeIdNotFound(id))?;

        // Collect all incoming neighbors
        let preds = self
            .cfg
            .neighbors_directed(node_id, Direction::Incoming)
            .collect::<Vec<_>>();

        Ok(preds
            .into_iter()
            .filter_map(|pred| self.node_id_to_block_id(pred))
            .collect())
    }

    /// Get the successors of a `BasicBlock`.
    ///
    /// # Arguments
    /// - `id`: The `BasicBlockId` of the block.
    ///
    /// # Returns
    /// - A vector of `BasicBlockId`s that are successors of the block.
    ///
    /// # Errors
    /// - `FunctionError::BasicBlockNodeIdNotFound` if the block does not exist.
    /// - `FunctionError::GraphError` if the successors could not be retrieved from the graph.
    ///
    /// # Example
    /// ```
    /// use gbf_rs::function::{Function, FunctionId};
    /// use gbf_rs::basic_block::BasicBlockType;
    ///
    /// let mut function = Function::new(FunctionId::new(0, None, 0));
    /// let block1 = function.create_block(BasicBlockType::Normal).unwrap();
    /// let block2 = function.create_block(BasicBlockType::Normal).unwrap();
    ///
    /// function.add_edge(block1, block2);
    /// let succs = function.get_successors(block1).unwrap();
    /// ```
    pub fn get_successors(&self, id: BasicBlockId) -> Result<Vec<BasicBlockId>, FunctionError> {
        let node_id = self
            .block_id_to_node_id(id)
            .ok_or(FunctionError::BasicBlockNodeIdNotFound(id))?;

        // Collect all outgoing neighbors
        let succs = self
            .cfg
            .neighbors_directed(node_id, Direction::Outgoing)
            .collect::<Vec<_>>();

        Ok(succs
            .into_iter()
            .filter_map(|succ| self.node_id_to_block_id(succ))
            .collect())
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_function() {
        let id = FunctionId::new(0, None, 0);
        let function = Function::new(id.clone());

        assert_eq!(function.id, id);
        assert_eq!(function.blocks.len(), 1);
    }

    #[test]
    fn create_block() {
        let id = FunctionId::new(0, None, 0);
        let mut function = Function::new(id.clone());
        let block_id = function.create_block(BasicBlockType::Normal).unwrap();

        assert_eq!(function.blocks.len(), 2);
        assert!(function.blocks.contains_key(&block_id));

        // check block id & node id mappings
        let node_id = function.block_to_node.get(&block_id).unwrap();
        let new_block_id = function.node_to_block.get(node_id).unwrap();
        assert_eq!(block_id, *new_block_id);

        // test EntryBlockAlreadyExists error
        let result = function.create_block(BasicBlockType::Entry);
        assert!(result.is_err());
    }

    #[test]
    fn get_block() {
        let id = FunctionId::new(0, None, 0);
        let mut function = Function::new(id.clone());
        let block_id = function.create_block(BasicBlockType::Normal).unwrap();
        let block = function.get_block(block_id).unwrap();

        assert_eq!(block.id, block_id);
    }

    #[test]
    fn get_block_mut() {
        let id = FunctionId::new(0, None, 0);
        let mut function = Function::new(id);
        let block_id = function.create_block(BasicBlockType::Normal).unwrap();
        let block = function.get_block_mut(block_id).unwrap();

        block.id = BasicBlockId::new(0, BasicBlockType::Exit);
        assert_eq!(block.id, BasicBlockId::new(0, BasicBlockType::Exit));
    }

    #[test]
    fn test_display_function_id() {
        let id = FunctionId::new(0, None, 0);
        assert_eq!(id.to_string(), "Function0");
    }

    #[test]
    fn test_is_named() {
        let id = FunctionId::new(0, None, 0);
        assert!(id.is_named());

        let id = FunctionId::new(0, Some("test".to_string()), 0);
        assert!(!id.is_named());
    }

    #[test]
    fn test_get_entry_block() {
        let id = FunctionId::new(0, None, 0);
        let function = Function::new(id.clone());
        let entry = function.get_entry_block();

        assert_eq!(entry.id, function.entry_block);
    }

    #[test]
    fn test_get_entry_block_mut() {
        let id = FunctionId::new(0, None, 0);
        let mut function = Function::new(id.clone());
        let entry_id = function.entry_block;
        let entry = function.get_entry_block_mut();

        assert_eq!(entry.id, entry_id);
    }

    #[test]
    fn test_add_edge() {
        let id = FunctionId::new(0, None, 0);
        let mut function = Function::new(id.clone());
        let block1 = function.create_block(BasicBlockType::Normal).unwrap();
        let block2 = function.create_block(BasicBlockType::Normal).unwrap();

        let result = function.add_edge(block1, block2);
        assert!(result.is_ok());

        let preds = function.get_predecessors(block2).unwrap();
        assert_eq!(preds.len(), 1);
        assert_eq!(preds[0], block1);

        let succs = function.get_successors(block1).unwrap();
        assert_eq!(succs.len(), 1);
        assert_eq!(succs[0], block2);
    }

    #[test]
    fn test_get_predecessors() {
        let id = FunctionId::new(0, None, 0);
        let mut function = Function::new(id.clone());
        let block1 = function.create_block(BasicBlockType::Normal).unwrap();
        let block2 = function.create_block(BasicBlockType::Normal).unwrap();

        function.add_edge(block1, block2).unwrap();
        let preds = function.get_predecessors(block2).unwrap();

        assert_eq!(preds.len(), 1);
        assert_eq!(preds[0], block1);
    }

    #[test]
    fn test_get_successors() {
        let id = FunctionId::new(0, None, 0);
        let mut function = Function::new(id.clone());
        let block1 = function.create_block(BasicBlockType::Normal).unwrap();
        let block2 = function.create_block(BasicBlockType::Normal).unwrap();

        function.add_edge(block1, block2).unwrap();
        let succs = function.get_successors(block1).unwrap();

        assert_eq!(succs.len(), 1);
        assert_eq!(succs[0], block2);
    }
}
