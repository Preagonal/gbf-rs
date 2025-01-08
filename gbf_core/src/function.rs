#![deny(missing_docs)]

use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::{DfsPostOrder, Walker};
use petgraph::Direction;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{self, Display, Formatter};
use std::hash::Hash;
use std::ops::{Deref, Index};
use thiserror::Error;

use crate::basic_block::{BasicBlock, BasicBlockId, BasicBlockType};
use crate::cfg_dot::{CfgDot, CfgDotConfig, DotRenderableGraph, NodeResolver};
use crate::utils::{Gs2BytecodeAddress, GBF_BLUE, GBF_GREEN, GBF_RED};

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
    /// The name of the function, if it is not the entry point.
    pub name: Option<String>,
    /// The address of the function in the module.
    pub address: Gs2BytecodeAddress,
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
    /// use gbf_core::function::FunctionId;
    ///
    /// let entry = FunctionId::new_without_name(0, 0);
    /// let add = FunctionId::new(1, Some("add"), 0x100);
    /// ```
    pub fn new<S>(index: usize, name: Option<S>, address: Gs2BytecodeAddress) -> Self
    where
        S: Into<String>,
    {
        Self {
            index,
            name: name.map(|n| n.into()),
            address,
        }
    }

    /// Helper method for creating a `FunctionId` without a name.
    pub fn new_without_name(index: usize, address: Gs2BytecodeAddress) -> Self {
        Self::new(index, None::<String>, address)
    }

    /// If the function has a name.
    ///
    /// # Returns
    /// - `true` if the function has a name.
    /// - `false` if the function does not have a name.
    ///
    /// # Example
    /// ```
    /// use gbf_core::function::FunctionId;
    ///
    /// let entry = FunctionId::new_without_name(0, 0);
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
    /// A vector of all the `BasicBlock`s in the function.
    blocks: Vec<BasicBlock>,
    /// Maps `BasicBlockId` to their index in the `blocks` vector.
    block_map: HashMap<BasicBlockId, usize>,
    /// The control-flow graph of the function.
    cfg: DiGraph<(), ()>,
    /// Used to convert `NodeIndex` to `BasicBlockId`.
    graph_node_to_block: HashMap<NodeIndex, BasicBlockId>,
    /// Used to convert `BasicBlockId` to `NodeIndex`.
    block_to_graph_node: HashMap<BasicBlockId, NodeIndex>,
    /// A map of function addresses to their IDs.
    address_to_id: HashMap<Gs2BytecodeAddress, FunctionId>,
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
        let mut blocks = Vec::new();
        let mut block_map = HashMap::new();
        let mut graph_node_to_block = HashMap::new();
        let mut block_to_graph_node = HashMap::new();
        let address_to_id = HashMap::new();
        let mut cfg = DiGraph::new();

        // Initialize entry block
        let entry_block = BasicBlockId::new(blocks.len(), BasicBlockType::Entry, id.address);
        blocks.push(BasicBlock::new(entry_block));
        block_map.insert(entry_block, 0);

        // Add an empty node in the graph to represent this BasicBlock
        let entry_node_id = cfg.add_node(());
        graph_node_to_block.insert(entry_node_id, entry_block);
        block_to_graph_node.insert(entry_block, entry_node_id);

        Self {
            id,
            blocks,
            block_map,
            cfg,
            graph_node_to_block,
            block_to_graph_node,
            address_to_id,
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
    /// use gbf_core::function::{Function, FunctionId};
    /// use gbf_core::basic_block::BasicBlockType;
    ///
    /// let mut function = Function::new(FunctionId::new_without_name(0, 0));
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
        self.blocks.push(BasicBlock::new(id));
        self.block_map.insert(id, self.blocks.len() - 1);

        // Insert a node in the petgraph to represent this BasicBlock
        let node_id = self.cfg.add_node(());
        self.block_to_graph_node.insert(id, node_id);
        self.graph_node_to_block.insert(node_id, id);

        Ok(id)
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
    /// use gbf_core::function::{Function, FunctionId};
    /// use gbf_core::basic_block::BasicBlockType;
    ///
    /// let mut function = Function::new(FunctionId::new_without_name(0, 0));
    /// let block_id = function.create_block(BasicBlockType::Normal, 0).unwrap();
    /// let block_ref = function.get_basic_block_by_id(block_id).unwrap();
    /// ```
    pub fn get_basic_block_by_id(&self, id: BasicBlockId) -> Result<&BasicBlock, FunctionError> {
        let index = self
            .block_map
            .get(&id)
            .ok_or(FunctionError::BasicBlockNotFoundById(id))?;
        Ok(&self.blocks[*index])
    }

    /// Get a reference to a `BasicBlock` by its address. The block address
    /// -must- be the start address of the block.
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
    /// use gbf_core::function::{Function, FunctionId};
    /// use gbf_core::basic_block::BasicBlockType;
    ///
    /// let mut function = Function::new(FunctionId::new_without_name(0, 0));
    /// let block_id = function.create_block(BasicBlockType::Normal, 0).unwrap();
    /// let block_ref = function.get_basic_block_by_id_mut(block_id).unwrap();
    /// ```
    pub fn get_basic_block_by_id_mut(
        &mut self,
        id: BasicBlockId,
    ) -> Result<&mut BasicBlock, FunctionError> {
        let index = self
            .block_map
            .get(&id)
            .ok_or(FunctionError::BasicBlockNotFoundById(id))?;
        Ok(&mut self.blocks[*index])
    }

    /// Get a reference to a `BasicBlock` by its address.
    ///
    /// # Arguments
    /// - `address`: The address of the block.
    ///
    /// # Returns
    /// - A reference to the `BasicBlock`.
    ///
    /// # Errors
    /// - `FunctionError::BasicBlockNotFoundByAddress` if the block does not exist.
    ///
    /// # Example
    /// ```
    /// use gbf_core::function::{Function, FunctionId};
    /// use gbf_core::basic_block::BasicBlockType;
    ///
    /// let mut function = Function::new(FunctionId::new_without_name(0, 0));
    /// let block_id = function.create_block(BasicBlockType::Normal, 0x100).unwrap();
    /// let block_ref = function.get_basic_block_by_start_address(0x100).unwrap();
    /// ```
    pub fn get_basic_block_by_start_address(
        &self,
        address: Gs2BytecodeAddress,
    ) -> Result<&BasicBlock, FunctionError> {
        let id = self.get_basic_block_id_by_start_address(address)?;
        self.get_basic_block_by_id(id)
    }

    /// Get a reference to a `BasicBlock` by its address (mutable). The block address
    /// -must- be the start address of the block.
    ///
    /// # Arguments
    /// - `address`: The address of the block.
    ///
    /// # Returns
    /// - A reference to the `BasicBlock`.
    ///
    /// # Errors
    /// - `FunctionError::BasicBlockNotFoundByAddress` if the block does not exist.
    ///
    /// # Example
    /// ```
    /// use gbf_core::function::{Function, FunctionId};
    /// use gbf_core::basic_block::BasicBlockType;
    ///
    /// let mut function = Function::new(FunctionId::new_without_name(0, 0));
    /// let block_id = function.create_block(BasicBlockType::Normal, 0x100).unwrap();
    /// let block_ref = function.get_basic_block_by_start_address_mut(0x100).unwrap();
    /// ```
    pub fn get_basic_block_by_start_address_mut(
        &mut self,
        address: Gs2BytecodeAddress,
    ) -> Result<&mut BasicBlock, FunctionError> {
        let id = self.get_basic_block_id_by_start_address(address)?;
        self.get_basic_block_by_id_mut(id)
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
    /// use gbf_core::function::{Function, FunctionId};
    /// use gbf_core::basic_block::BasicBlockType;
    ///
    /// let mut function = Function::new(FunctionId::new_without_name(0, 0));
    /// let block_id = function.create_block(BasicBlockType::Normal, 0x100).unwrap();
    /// assert!(function.basic_block_exists_by_address(0x100));
    /// ```
    pub fn basic_block_exists_by_address(&self, address: Gs2BytecodeAddress) -> bool {
        self.blocks.iter().any(|block| block.id.address == address)
    }

    /// Gets the entry basic block id of the function.
    ///
    /// # Returns
    /// - The `BasicBlockId` of the entry block.
    ///
    /// # Example
    /// ```
    /// use gbf_core::function::{Function, FunctionId};
    ///
    /// let mut function = Function::new(FunctionId::new_without_name(0, 0));
    /// let entry = function.get_entry_basic_block_id();
    /// ```
    pub fn get_entry_basic_block_id(&self) -> BasicBlockId {
        self.blocks[0].id
    }

    /// Get the entry basic block of the function.
    ///
    /// # Returns
    /// - A reference to the entry block.
    ///
    /// # Example
    /// ```
    /// use gbf_core::function::{Function, FunctionId};
    ///
    /// let mut function = Function::new(FunctionId::new_without_name(0, 0));
    /// let entry = function.get_entry_basic_block();
    /// ```
    pub fn get_entry_basic_block(&self) -> &BasicBlock {
        self.blocks.first().unwrap()
    }

    /// Get the entry block of the function.
    ///
    /// # Returns
    /// - A mutable reference to the entry block.
    ///
    /// # Example
    /// ```
    /// use gbf_core::function::{Function, FunctionId};
    ///
    /// let mut function = Function::new(FunctionId::new_without_name(0, 0));
    /// let entry = function.get_entry_basic_block_mut();
    /// ```
    pub fn get_entry_basic_block_mut(&mut self) -> &mut BasicBlock {
        self.blocks.first_mut().unwrap()
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
    /// use gbf_core::function::{Function, FunctionId};
    /// use gbf_core::basic_block::BasicBlockType;
    ///
    /// let mut function = Function::new(FunctionId::new_without_name(0, 0));
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

    /// Get the number of `BasicBlock`s in the function.
    ///
    /// # Returns
    /// - The number of `BasicBlock`s in the function.
    ///
    /// # Example
    /// ```
    /// use gbf_core::function::{Function, FunctionId};
    /// use gbf_core::basic_block::BasicBlockType;
    ///
    /// let mut function = Function::new(FunctionId::new_without_name(0, 0));
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
    /// use gbf_core::function::{Function, FunctionId};
    ///
    /// let function = Function::new(FunctionId::new_without_name(0, 0));
    /// assert!(!function.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        // This will always be false since we always create an entry block
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
    /// use gbf_core::function::{Function, FunctionId};
    /// use gbf_core::basic_block::BasicBlockType;
    ///
    /// let mut function = Function::new(FunctionId::new_without_name(0, 0));
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
    /// use gbf_core::function::{Function, FunctionId};
    /// use gbf_core::basic_block::BasicBlockType;
    ///
    /// let mut function = Function::new(FunctionId::new_without_name(0, 0));
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

    /// Get the blocks in reverse post order
    ///
    /// # Arguments
    /// - `id`: The `BasicBlockId` of the starting block
    ///
    /// # Returns
    /// - A vector of `BasicBlockId`s that sort the graph in reverse post order
    ///
    /// # Errors
    /// - `FunctionError::BasicBlockNodeIndexNotFound` if the block does not exist.
    /// - `FunctionError::GraphError` if the successors could not be retrieved from the graph.
    pub fn get_reverse_post_order(
        &self,
        id: BasicBlockId,
    ) -> Result<Vec<BasicBlockId>, FunctionError> {
        let node_id = self
            .block_id_to_node_id(id)
            .ok_or(FunctionError::BasicBlockNodeIndexNotFound(id))?;

        let dfs = DfsPostOrder::new(&self.cfg, node_id)
            .iter(&self.cfg)
            .collect::<Vec<_>>();

        Ok(dfs
            .into_iter()
            .filter_map(|node_id| self.node_id_to_block_id(node_id))
            .collect())
    }
}

/// Internal API for `Function`.
impl Function {
    /// Gets a block id based on its address.
    ///
    /// # Arguments
    /// - `address`: The address of the block.
    ///
    /// # Returns
    /// - The `BasicBlockId` of the block with the corresponding address.
    ///
    /// # Errors
    /// - `FunctionError::BasicBlockNotFoundByAddress` if the block does not exist.
    pub fn get_basic_block_id_by_start_address(
        &self,
        address: Gs2BytecodeAddress,
    ) -> Result<BasicBlockId, FunctionError> {
        self.blocks
            .iter()
            .find(|block| block.id.address == address)
            .map(|block| block.id)
            .ok_or(FunctionError::BasicBlockNotFoundByAddress(address))
    }

    /// Convert a `NodeIndex` to a `BasicBlockId`.
    ///
    /// # Arguments
    /// - `node_id`: The `NodeIndex` to convert.
    ///
    /// # Returns
    /// - The `BasicBlockId` of the block with the corresponding `NodeIndex`.
    fn node_id_to_block_id(&self, node_id: NodeIndex) -> Option<BasicBlockId> {
        self.graph_node_to_block.get(&node_id).cloned()
    }

    /// Convert a `BasicBlockId` to a `NodeIndex`.
    ///
    /// # Arguments
    /// - `block_id`: The `BasicBlockId` to convert.
    ///
    /// # Returns
    /// - The `NodeIndex` of the block with the corresponding `BasicBlockId`.
    fn block_id_to_node_id(&self, block_id: BasicBlockId) -> Option<NodeIndex> {
        self.block_to_graph_node.get(&block_id).cloned()
    }
}

// === Implementations ===

/// Display implementation for `FunctionId`.
impl Display for FunctionId {
    /// Display the `Function` as its name.
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        if let Some(name) = &self.name {
            write!(f, "{}", name)
        } else {
            write!(f, "Unnamed Function (Entry)")
        }
    }
}

/// Clone implementation for Function
impl Clone for Function {
    fn clone(&self) -> Self {
        let mut blocks = Vec::new();
        let mut block_map = HashMap::new();
        let mut graph_node_to_block = HashMap::new();
        let mut block_to_graph_node = HashMap::new();
        let address_to_id = HashMap::new();
        let mut cfg = DiGraph::new();

        // Clone blocks
        for block in &self.blocks {
            let new_block = block.clone();
            let new_block_id = new_block.id;
            blocks.push(new_block);
            block_map.insert(new_block_id, blocks.len() - 1);

            // Insert a node in the petgraph to represent this BasicBlock
            let node_id = cfg.add_node(());
            block_to_graph_node.insert(new_block_id, node_id);
            graph_node_to_block.insert(node_id, new_block_id);
        }

        // Clone edges
        for edge in self.cfg.raw_edges() {
            let source = self.graph_node_to_block[&edge.source()];
            let target = self.graph_node_to_block[&edge.target()];
            let source_node_id = block_to_graph_node[&source];
            let target_node_id = block_to_graph_node[&target];
            cfg.add_edge(source_node_id, target_node_id, ());
        }

        Self {
            id: self.id.clone(),
            blocks,
            block_map,
            cfg,
            graph_node_to_block,
            block_to_graph_node,
            address_to_id,
        }
    }
}

/// Deref implementation for Function
impl Deref for Function {
    type Target = [BasicBlock];

    fn deref(&self) -> &Self::Target {
        &self.blocks
    }
}

/// Index implementation for function, with usize
impl Index<usize> for Function {
    type Output = BasicBlock;

    fn index(&self, index: usize) -> &Self::Output {
        &self.blocks[index]
    }
}

/// IntoIterator implementation immutable reference
impl<'a> IntoIterator for &'a Function {
    type Item = &'a BasicBlock;
    type IntoIter = std::slice::Iter<'a, BasicBlock>;

    fn into_iter(self) -> Self::IntoIter {
        self.blocks.iter()
    }
}

/// IntoIterator implementation mutable reference
impl<'a> IntoIterator for &'a mut Function {
    type Item = &'a mut BasicBlock;
    type IntoIter = std::slice::IterMut<'a, BasicBlock>;

    fn into_iter(self) -> Self::IntoIter {
        self.blocks.iter_mut()
    }
}

impl NodeResolver for Function {
    type NodeData = BasicBlock;

    fn resolve(&self, node_index: NodeIndex) -> Option<&Self::NodeData> {
        self.graph_node_to_block
            .get(&node_index)
            .and_then(|block_id| {
                self.block_map
                    .get(block_id)
                    .and_then(|index| self.blocks.get(*index))
            })
    }

    fn resolve_edge_color(&self, source: NodeIndex, target: NodeIndex) -> String {
        // Get the last instruction of the source block
        let source_block_id = self
            .graph_node_to_block
            .get(&source)
            .expect("Source block not found");
        let source_block = self
            .get_basic_block_by_id(*source_block_id)
            .expect("Source block not found");
        let source_last_instruction = source_block.last().unwrap();

        let target_block_id = self
            .graph_node_to_block
            .get(&target)
            .expect("Target block not found");
        let target_block = self
            .get_basic_block_by_id(*target_block_id)
            .expect("Target block not found");

        // Figure out if the edge represents a branch by seeing if the target
        // block address is NOT the next address after the source instruction.
        let source_last_address = source_last_instruction.address;
        let target_address = target_block.id.address;
        if source_last_address + 1 != target_address {
            // This represents a branch. Color the edge green.
            return GBF_GREEN.to_string();
        }

        // If the opcode of the last instruction is a fall through, color the edge red since
        // the target block's address is the next address
        if source_last_instruction.opcode.has_fall_through() {
            return GBF_RED.to_string();
        }

        // Otherwise, color the edge cyan (e.g. normal control flow)
        GBF_BLUE.to_string()
    }
}

impl DotRenderableGraph for Function {
    /// Convert the Graph to `dot` format.
    ///
    /// # Returns
    /// - A `String` containing the `dot` representation of the graph.
    fn render_dot(&self, config: CfgDotConfig) -> String {
        let cfg = CfgDot { config };
        cfg.render(&self.cfg, self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_function() {
        let id = FunctionId::new_without_name(0, 0);
        let function = Function::new(id.clone());

        assert_eq!(function.id, id);
        assert_eq!(function.blocks.len(), 1);
    }

    #[test]
    fn create_block() {
        let id = FunctionId::new_without_name(0, 0);
        let mut function = Function::new(id.clone());
        let block_id = function.create_block(BasicBlockType::Normal, 32).unwrap();

        assert_eq!(function.len(), 2);

        // check block id & node id mappings
        let node_id = function.block_to_graph_node.get(&block_id).unwrap();
        let new_block_id = function.graph_node_to_block.get(node_id).unwrap();
        assert_eq!(*new_block_id, block_id);

        // test EntryBlockAlreadyExists error
        let result = function.create_block(BasicBlockType::Entry, 0);
        assert!(result.is_err());
    }

    #[test]
    fn get_block() {
        let id = FunctionId::new_without_name(0, 0);
        let mut function = Function::new(id.clone());
        let block_id = function.create_block(BasicBlockType::Normal, 32).unwrap();
        let block = function.get_basic_block_by_id(block_id).unwrap();

        assert_eq!(block.id, block_id);
    }

    #[test]
    fn get_block_mut() {
        let id = FunctionId::new_without_name(0, 0);
        let mut function = Function::new(id);
        let block_id = function.create_block(BasicBlockType::Normal, 43).unwrap();
        let block = function.get_basic_block_by_id_mut(block_id).unwrap();

        block.id = BasicBlockId::new(0, BasicBlockType::Exit, 43);
        assert_eq!(block.id, BasicBlockId::new(0, BasicBlockType::Exit, 43));
    }

    #[test]
    fn test_get_block_not_found() {
        let id = FunctionId::new_without_name(0, 0);
        let function = Function::new(id.clone());
        let result =
            function.get_basic_block_by_id(BasicBlockId::new(1234, BasicBlockType::Normal, 0));

        assert!(result.is_err());

        // test mut version
        let mut function = Function::new(id.clone());
        let result =
            function.get_basic_block_by_id_mut(BasicBlockId::new(1234, BasicBlockType::Normal, 0));

        assert!(result.is_err());

        // get by start address
        let result = function.get_basic_block_by_start_address(0x100);
        assert!(result.is_err());

        // get by start address mut
        let result = function.get_basic_block_by_start_address_mut(0x100);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_block_by_address() {
        let id = FunctionId::new_without_name(0, 0);
        let mut function = Function::new(id.clone());
        let block_id = function
            .create_block(BasicBlockType::Normal, 0x100)
            .unwrap();
        let block = function.get_basic_block_by_start_address(0x100).unwrap();

        assert_eq!(block.id, block_id);

        // test mut version
        let block = function
            .get_basic_block_by_start_address_mut(0x100)
            .unwrap();
        block.id = BasicBlockId::new(0, BasicBlockType::Exit, 0x100);
        assert_eq!(block.id, BasicBlockId::new(0, BasicBlockType::Exit, 0x100));
    }

    #[test]
    fn test_display_function_id() {
        let id = FunctionId::new_without_name(0, 0);
        assert_eq!(id.to_string(), "Unnamed Function (Entry)");

        let id = FunctionId::new(0, Some("test".to_string()), 0);
        assert_eq!(id.to_string(), "test");
    }

    #[test]
    fn test_into_iter_mut() {
        let id = FunctionId::new_without_name(0, 0);
        let mut function = Function::new(id.clone());
        let block_id = function.create_block(BasicBlockType::Normal, 32).unwrap();

        for block in &mut function {
            if block.id == block_id {
                block.id = BasicBlockId::new(0, BasicBlockType::Exit, 32);
            }
        }

        let block = function.get_basic_block_by_id(block_id).unwrap();
        assert_eq!(block.id, BasicBlockId::new(0, BasicBlockType::Exit, 32));
    }

    #[test]
    fn test_is_named() {
        let id = FunctionId::new_without_name(0, 0);
        assert!(id.is_named());

        let id = FunctionId::new(0, Some("test".to_string()), 0);
        assert!(!id.is_named());
    }

    #[test]
    fn test_get_entry_block() {
        let id = FunctionId::new_without_name(0, 0);
        let function = Function::new(id.clone());
        let entry = function.get_entry_basic_block();

        assert_eq!(entry.id, function.get_entry_basic_block().id);
    }

    #[test]
    fn test_get_entry_block_mut() {
        let id = FunctionId::new_without_name(0, 0);
        let mut function = Function::new(id.clone());
        let entry_id = function.get_entry_basic_block().id;
        let entry = function.get_entry_basic_block_mut();

        assert_eq!(entry.id, entry_id);
    }

    #[test]
    fn test_add_edge() {
        let id = FunctionId::new_without_name(0, 0);
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

        // test source not found
        let result = function.add_edge(BasicBlockId::new(1234, BasicBlockType::Normal, 0), block2);
        assert!(result.is_err());

        // test target not found
        let result = function.add_edge(block1, BasicBlockId::new(1234, BasicBlockType::Normal, 0));
        assert!(result.is_err());
    }

    #[test]
    fn test_basic_block_is_empty() {
        // will always be false since we always create an entry block
        let id = FunctionId::new_without_name(0, 0);
        let function = Function::new(id.clone());
        assert!(!function.is_empty());
    }

    #[test]
    fn test_get_predecessors() {
        let id = FunctionId::new_without_name(0, 0);
        let mut function = Function::new(id.clone());
        let block1 = function.create_block(BasicBlockType::Normal, 32).unwrap();
        let block2 = function.create_block(BasicBlockType::Normal, 32).unwrap();

        function.add_edge(block1, block2).unwrap();
        let preds = function.get_predecessors(block2).unwrap();

        assert_eq!(preds.len(), 1);
        assert_eq!(preds[0], block1);

        // test error where block not found
        let result = function.get_predecessors(BasicBlockId::new(1234, BasicBlockType::Normal, 0));
        assert!(result.is_err());
    }

    #[test]
    fn test_get_successors() {
        let id = FunctionId::new_without_name(0, 0);
        let mut function = Function::new(id.clone());
        let block1 = function.create_block(BasicBlockType::Normal, 32).unwrap();
        let block2 = function.create_block(BasicBlockType::Normal, 32).unwrap();

        function.add_edge(block1, block2).unwrap();
        let succs = function.get_successors(block1).unwrap();

        assert_eq!(succs.len(), 1);
        assert_eq!(succs[0], block2);

        // test error where block not found
        let result = function.get_successors(BasicBlockId::new(1234, BasicBlockType::Normal, 0));
        assert!(result.is_err());
    }

    #[test]
    fn test_get_entry_basic_block_id() {
        let id = FunctionId::new_without_name(0, 0);
        let function = Function::new(id.clone());
        let entry = function.get_entry_basic_block_id();

        assert_eq!(entry, function.get_entry_basic_block().id);
    }
}
