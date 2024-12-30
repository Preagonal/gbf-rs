#![deny(missing_docs)]

use std::collections::BTreeMap;
use std::fmt;
use std::hash::Hash;

use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::Direction;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::basic_block::{BasicBlock, BasicBlockId, BasicBlockType};
use crate::utils::Gs2BytecodeAddress;

/// Represents an error that can occur when working with functions.
#[derive(Error, Debug)]
pub enum FunctionError {
    /// The requested `BasicBlock` was not found by its block id.
    #[error("BasicBlock not found by its block id: {0}")]
    BasicBlockNotFoundById(BasicBlockId),

    /// The requested `BasicBlock` was not found by its address.
    #[error("BasicBlock not found by its address: {0}")]
    BasicBlockNotFoundByAddress(Gs2BytecodeAddress),

    /// The requested `BasicBlock` does not have a `NodeIndex`.
    #[error("BasicBlock with id {0} does not have a NodeIndex")]
    BasicBlockNodeIndexNotFound(BasicBlockId),

    /// The function already has an entry block.
    #[error("Function already has an entry block")]
    EntryBlockAlreadyExists,
}

/// Represents the identifier of a function.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct FunctionId {
    index: usize,
    // TODO: It would be nice to be able to copy instead of clone by
    // implementing Copy, but String does not implement Copy. We could
    // use a reference in the future.
    /// The name of the function, if it is not the entry point.
    pub name: Option<String>,
    /// The address of the function in the module.
    pub address: Gs2BytecodeAddress,
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
    /// - `address`: The address of the function in the module.
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
    pub fn new(index: usize, name: Option<String>, address: Gs2BytecodeAddress) -> Self {
        Self {
            index,
            name,
            address,
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
#[derive(Debug, Serialize, Deserialize)]
pub struct Function {
    /// The identifier of the function.
    pub id: FunctionId,
    entry_block: BasicBlockId,
    blocks: BTreeMap<BasicBlockId, BasicBlock>,

    // Our petgraph-based control-flow graph
    cfg: DiGraph<(), ()>,

    // used to convert NodeIndex to BasicBlockId
    node_to_block: BTreeMap<NodeIndex, BasicBlockId>,

    // used to convert BasicBlockId to NodeIndex
    block_to_node: BTreeMap<BasicBlockId, NodeIndex>,
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
        let mut blocks = BTreeMap::new();
        let mut node_to_block: BTreeMap<NodeIndex, BasicBlockId> = BTreeMap::new();
        let mut block_to_node: BTreeMap<BasicBlockId, NodeIndex> = BTreeMap::new();
        let mut cfg = DiGraph::new();

        // Initialize entry block
        let entry_block = BasicBlockId::new(blocks.len(), BasicBlockType::Entry, id.address);
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
    /// let block = function.create_block(BasicBlockType::Normal, 0);
    /// ```
    pub fn create_block(
        &mut self,
        block_type: BasicBlockType,
        address: Gs2BytecodeAddress,
    ) -> Result<BasicBlockId, FunctionError> {
        // do not allow entry block to be created more than once
        if block_type == BasicBlockType::Entry {
            return Err(FunctionError::EntryBlockAlreadyExists);
        }

        let id = BasicBlockId::new(self.blocks.len(), block_type, address);
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
    /// - The `BasicBlockId` of the block with the corresponding `NodeIndex`.
    fn node_id_to_block_id(&self, node_id: NodeIndex) -> Option<BasicBlockId> {
        self.node_to_block.get(&node_id).cloned()
    }

    /// Convert a `BasicBlockId` to a `NodeIndex`.
    ///
    /// # Arguments
    /// - `block_id`: The `BasicBlockId` to convert.
    ///
    /// # Returns
    /// - The `NodeIndex` of the block with the corresponding `BasicBlockId`.
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
    /// let block_id = function.create_block(BasicBlockType::Normal, 0).unwrap();
    /// let block_ref = function.get_block(block_id).unwrap();
    /// ```
    pub fn get_block(&self, id: BasicBlockId) -> Result<&BasicBlock, FunctionError> {
        self.blocks
            .get(&id)
            .ok_or(FunctionError::BasicBlockNotFoundById(id))
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
    /// let block_id = function.create_block(BasicBlockType::Normal, 0).unwrap();
    /// let block_ref = function.get_block_mut(block_id).unwrap();
    /// ```
    pub fn get_block_mut(&mut self, id: BasicBlockId) -> Result<&mut BasicBlock, FunctionError> {
        self.blocks
            .get_mut(&id)
            .ok_or(FunctionError::BasicBlockNotFoundById(id))
    }

    /// Gets a block id based on its address.
    ///
    /// # Arguments
    /// - `address`: The address of the block.
    ///
    /// # Returns
    /// - The `BasicBlockId` of the block with the corresponding address.
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
    /// let block_id = function.create_block(BasicBlockType::Normal, 0x100).unwrap();
    /// let block_id = function.get_block_id_by_address(0x100).unwrap();
    /// ```
    pub fn get_block_id_by_address(
        &self,
        address: Gs2BytecodeAddress,
    ) -> Result<BasicBlockId, FunctionError> {
        for block_id in self.blocks.keys() {
            if block_id.address == address {
                return Ok(*block_id);
            }
        }

        Err(FunctionError::BasicBlockNotFoundByAddress(address))
    }

    /// Check if a block exists by its address.
    ///
    /// # Arguments
    /// - `address`: The address of the block.
    ///
    /// # Returns
    /// - `true` if the block exists.
    /// - `false` if the block does not exist.
    ///
    /// # Example
    /// ```
    /// use gbf_rs::function::{Function, FunctionId};
    /// use gbf_rs::basic_block::BasicBlockType;
    ///
    /// let mut function = Function::new(FunctionId::new(0, None, 0));
    /// let block_id = function.create_block(BasicBlockType::Normal, 0x100).unwrap();
    /// assert!(function.has_basic_block_by_address(0x100));
    /// ```
    pub fn has_basic_block_by_address(&self, address: Gs2BytecodeAddress) -> bool {
        self.blocks
            .iter()
            .any(|(block_id, _)| block_id.address == address)
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
    /// - `FunctionError::BasicBlockNodeIndexNotFound` if either block does not have a `NodeIndex`.
    /// - `FunctionError::GraphError` if the edge could not be added to the graph.
    ///
    /// # Example
    /// ```
    /// use gbf_rs::function::{Function, FunctionId};
    /// use gbf_rs::basic_block::BasicBlockType;
    ///
    /// let mut function = Function::new(FunctionId::new(0, None, 0));
    /// let block1 = function.create_block(BasicBlockType::Normal, 0).unwrap();
    /// let block2 = function.create_block(BasicBlockType::Normal, 0).unwrap();
    /// function.add_edge(block1, block2);
    /// ```
    pub fn add_edge(
        &mut self,
        source: BasicBlockId,
        target: BasicBlockId,
    ) -> Result<(), FunctionError> {
        let source_node_id = self
            .block_id_to_node_id(source)
            .ok_or(FunctionError::BasicBlockNodeIndexNotFound(source))?;
        let target_node_id = self
            .block_id_to_node_id(target)
            .ok_or(FunctionError::BasicBlockNodeIndexNotFound(target))?;

        // With petgraph, this does not fail, so we simply do it:
        // It can panic if the node does not exist, but we have already checked that.
        self.cfg.add_edge(source_node_id, target_node_id, ());
        Ok(())
    }

    /// Get a vector of all the block ids in the function.
    ///
    /// # Arguments
    /// - `function`: The function to get the block ids from.
    ///
    /// # Returns
    /// - A vector of all the block ids in the function.
    ///
    /// # Example
    /// ```
    /// use gbf_rs::function::{Function, FunctionId};
    /// use gbf_rs::basic_block::BasicBlockType;
    /// use gbf_rs::basic_block::BasicBlockId;
    ///
    /// let mut function = Function::new(FunctionId::new(0, None, 0));
    /// let block1 = function.create_block(BasicBlockType::Normal, 0).unwrap();
    /// let block2 = function.create_block(BasicBlockType::Normal, 0).unwrap();
    /// let block3 = function.create_block(BasicBlockType::Normal, 0).unwrap();
    /// ```
    pub fn get_block_ids(&self) -> Vec<BasicBlockId> {
        self.blocks.keys().cloned().collect()
    }

    /// Get the number of `BasicBlock`s in the function.
    ///
    /// # Returns
    /// - The number of `BasicBlock`s in the function.
    ///
    /// # Example
    /// ```
    /// use gbf_rs::function::{Function, FunctionId};
    /// use gbf_rs::basic_block::BasicBlockType;
    ///
    /// let mut function = Function::new(FunctionId::new(0, None, 0));
    /// let block1 = function.create_block(BasicBlockType::Normal, 0).unwrap();
    /// let block2 = function.create_block(BasicBlockType::Normal, 0).unwrap();
    /// let block3 = function.create_block(BasicBlockType::Normal, 0).unwrap();
    ///
    /// assert_eq!(function.len(), 4);
    /// ```
    pub fn len(&self) -> usize {
        self.blocks.len()
    }

    /// Check if the function is empty.
    ///
    /// # Returns
    /// - `true` if the function is empty.
    ///
    /// # Example
    /// ```
    /// use gbf_rs::function::{Function, FunctionId};
    ///
    /// let function = Function::new(FunctionId::new(0, None, 0));
    /// assert!(!function.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        // TODO: This will always be false at the moment since we always create an entry block
        self.blocks.is_empty()
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
    /// - `FunctionError::BasicBlockNodeIndexNotFound` if the block does not exist.
    /// - `FunctionError::GraphError` if the predecessors could not be retrieved from the graph.
    ///
    /// # Example
    /// ```
    /// use gbf_rs::function::{Function, FunctionId};
    /// use gbf_rs::basic_block::BasicBlockType;
    ///
    /// let mut function = Function::new(FunctionId::new(0, None, 0));
    /// let block1 = function.create_block(BasicBlockType::Normal, 0).unwrap();
    /// let block2 = function.create_block(BasicBlockType::Normal, 0).unwrap();
    ///
    /// function.add_edge(block1, block2);
    /// let preds = function.get_predecessors(block2).unwrap();
    /// ```
    pub fn get_predecessors(&self, id: BasicBlockId) -> Result<Vec<BasicBlockId>, FunctionError> {
        let node_id = self
            .block_id_to_node_id(id)
            .ok_or(FunctionError::BasicBlockNodeIndexNotFound(id))?;

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
    /// - `FunctionError::BasicBlockNodeIndexNotFound` if the block does not exist.
    /// - `FunctionError::GraphError` if the successors could not be retrieved from the graph.
    ///
    /// # Example
    /// ```
    /// use gbf_rs::function::{Function, FunctionId};
    /// use gbf_rs::basic_block::BasicBlockType;
    ///
    /// let mut function = Function::new(FunctionId::new(0, None, 0));
    /// let block1 = function.create_block(BasicBlockType::Normal, 0).unwrap();
    /// let block2 = function.create_block(BasicBlockType::Normal, 0).unwrap();
    ///
    /// function.add_edge(block1, block2);
    /// let succs = function.get_successors(block1).unwrap();
    /// ```
    pub fn get_successors(&self, id: BasicBlockId) -> Result<Vec<BasicBlockId>, FunctionError> {
        let node_id = self
            .block_id_to_node_id(id)
            .ok_or(FunctionError::BasicBlockNodeIndexNotFound(id))?;

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

// Deref implementation for Function
impl std::ops::Deref for Function {
    type Target = BTreeMap<BasicBlockId, BasicBlock>;

    fn deref(&self) -> &Self::Target {
        &self.blocks
    }
}

// Index implementation for function, with usize
impl std::ops::Index<usize> for Function {
    type Output = BasicBlock;

    fn index(&self, index: usize) -> &Self::Output {
        self.blocks.values().nth(index).unwrap()
    }
}

// IntoIterator implementation for Function
impl IntoIterator for Function {
    type Item = BasicBlock;
    type IntoIter = std::collections::btree_map::IntoValues<BasicBlockId, BasicBlock>;

    fn into_iter(self) -> Self::IntoIter {
        self.blocks.into_values()
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
        let block_id = function.create_block(BasicBlockType::Normal, 32).unwrap();

        assert_eq!(function.blocks.len(), 2);
        assert!(function.blocks.contains_key(&block_id));

        // check block id & node id mappings
        let node_id = function.block_to_node.get(&block_id).unwrap();
        let new_block_id = function.node_to_block.get(node_id).unwrap();
        assert_eq!(block_id, *new_block_id);

        // test EntryBlockAlreadyExists error
        let result = function.create_block(BasicBlockType::Entry, 0);
        assert!(result.is_err());
    }

    #[test]
    fn get_block() {
        let id = FunctionId::new(0, None, 0);
        let mut function = Function::new(id.clone());
        let block_id = function.create_block(BasicBlockType::Normal, 32).unwrap();
        let block = function.get_block(block_id).unwrap();

        assert_eq!(block.id, block_id);
    }

    #[test]
    fn get_block_mut() {
        let id = FunctionId::new(0, None, 0);
        let mut function = Function::new(id);
        let block_id = function.create_block(BasicBlockType::Normal, 43).unwrap();
        let block = function.get_block_mut(block_id).unwrap();

        block.id = BasicBlockId::new(0, BasicBlockType::Exit, 43);
        assert_eq!(block.id, BasicBlockId::new(0, BasicBlockType::Exit, 43));
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
        let block1 = function.create_block(BasicBlockType::Normal, 32).unwrap();
        let block2 = function.create_block(BasicBlockType::Normal, 32).unwrap();

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
        let block1 = function.create_block(BasicBlockType::Normal, 32).unwrap();
        let block2 = function.create_block(BasicBlockType::Normal, 32).unwrap();

        function.add_edge(block1, block2).unwrap();
        let preds = function.get_predecessors(block2).unwrap();

        assert_eq!(preds.len(), 1);
        assert_eq!(preds[0], block1);
    }

    #[test]
    fn test_get_successors() {
        let id = FunctionId::new(0, None, 0);
        let mut function = Function::new(id.clone());
        let block1 = function.create_block(BasicBlockType::Normal, 32).unwrap();
        let block2 = function.create_block(BasicBlockType::Normal, 32).unwrap();

        function.add_edge(block1, block2).unwrap();
        let succs = function.get_successors(block1).unwrap();

        assert_eq!(succs.len(), 1);
        assert_eq!(succs[0], block2);
    }
}
