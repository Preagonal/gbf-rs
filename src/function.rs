use std::fmt;
use std::{collections::HashMap, hash::Hash};
use thiserror::Error;

use crate::basic_block::BasicBlockType;
use crate::graph::directed_graph::{GraphError, NodeId, NodeResolver};
use crate::{
    basic_block::{BasicBlock, BasicBlockId},
    graph::directed_graph::DirectedGraph,
};

#[derive(Error, Debug)]
pub enum FunctionError {
    #[error("BasicBlock not found: {0}")]
    BasicBlockNotFound(BasicBlockId),

    #[error("BasicBlock with id {0} does not have a NodeId")]
    BasicBlockNodeIdNotFound(BasicBlockId),

    #[error("Function already has an entry block")]
    EntryBlockAlreadyExists,

    #[error("Graph error: {0}")]
    GraphError(#[from] GraphError),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FunctionId {
    index: usize,
    name: Option<String>,
    offset: u64,
}

impl fmt::Display for FunctionId {
    /// Display the `BasicBlockId` as `BasicBlock{index}`.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "BasicBlock{}", self.index)
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
        !self.name.is_some()
    }
}

#[derive(Debug)]
pub struct Function {
    pub id: FunctionId,
    entry_block: BasicBlockId,
    blocks: HashMap<BasicBlockId, BasicBlock>,
    cfg: DirectedGraph<BasicBlockId>,
    // used to convert NodeId to BasicBlockId
    node_to_block: HashMap<NodeId, BasicBlockId>,
    // used to convert BasicBlockId to NodeId
    block_to_node: HashMap<BasicBlockId, NodeId>,
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
        let mut node_to_block = HashMap::new();
        let mut block_to_node = HashMap::new();

        // create control flow graph
        let mut cfg: DirectedGraph<BasicBlockId> = DirectedGraph::new();

        // initialize entry block
        let entry_block = BasicBlockId::new(blocks.len(), BasicBlockType::Entry);
        blocks.insert(entry_block, BasicBlock::new(entry_block));
        let entry_node_id = cfg.add_node(entry_block);
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
        let node_id = self.cfg.add_node(id);
        self.block_to_node.insert(id, node_id);
        self.node_to_block.insert(node_id, id);
        Ok(id)
    }

    /// Convert a `NodeId` to a `BasicBlockId`.
    ///
    /// # Arguments
    /// - `node_id`: The `NodeId` to convert.
    ///
    /// # Returns
    /// - The `BasicBlockId` of the block with the corresponding `NodeId`.
    fn node_id_to_block_id(&self, node_id: NodeId) -> Option<BasicBlockId> {
        self.node_to_block.get(&node_id).cloned()
    }

    /// Convert a `BasicBlockId` to a `NodeId`.
    ///
    /// # Arguments
    /// - `block_id`: The `BasicBlockId` to convert.
    ///
    /// # Returns
    /// - The `NodeId` of the block with the corresponding `BasicBlockId`.
    fn block_id_to_node_id(&self, block_id: BasicBlockId) -> Option<NodeId> {
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
        self.cfg
            .add_edge(source_node_id, target_node_id)
            .or_else(|e| Err(FunctionError::GraphError(e)))
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
            .ok_or(FunctionError::BasicBlockNodeIdNotFound(id.clone()))?;
        let preds = self
            .cfg
            .get_predecessors(node_id)
            .map_err(FunctionError::GraphError)?;

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
            .ok_or(FunctionError::BasicBlockNodeIdNotFound(id.clone()))?;
        let succs = self
            .cfg
            .get_successors(node_id)
            .map_err(FunctionError::GraphError)?;

        Ok(succs
            .into_iter()
            .filter_map(|succ| self.node_id_to_block_id(succ))
            .collect())
    }

    /// Get dot representation of the function
    ///
    /// # Returns
    /// - A string containing the dot representation of the function.
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
    /// let dot = function.to_dot();
    /// ```
    pub fn to_dot(&self) -> String {
        self.cfg.to_dot(self)
    }
}

impl NodeResolver for Function {
    type NodeData = BasicBlock;

    /// Resolve a NodeId to the corresponding BasicBlock.
    fn resolve(&self, node_id: NodeId) -> Option<&Self::NodeData> {
        self.node_to_block
            .get(&node_id)
            .and_then(|block_id| self.blocks.get(block_id))
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
}
