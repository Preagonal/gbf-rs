#![deny(missing_docs)]

use std::{
    fmt,
    ops::{Deref, Index},
    vec,
};

use serde::{Deserialize, Serialize};

use crate::{instruction::Instruction, utils::Gs2BytecodeAddress};

/// Represents the type of a basic block.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub enum BasicBlockType {
    /// Used for blocks that are entry blocks of a function.
    Entry,
    /// Used for blocks that are exit blocks of a function.
    Exit,
    /// Used for blocks that are neither entry nor exit blocks to a function.
    Normal,
    /// Used for blocks that are both entry and exit blocks. This is
    /// possible when a function has a single block, or when a block
    /// is both the entry and exit block of a function.
    ///
    /// Example:
    /// ```rs, no_run
    /// function onCreated()
    /// {
    ///   temp.foo = 1;
    ///   return temp.foo == 1 ? 1 : 0;
    /// }
    /// ```
    EntryAndExit,
    /// Special case for a block that is at the end of a module
    ModuleEnd,
}

/// Represents the identifier of a basic block.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct BasicBlockId {
    index: usize,

    /// The type of the basic block.
    pub block_type: BasicBlockType,

    /// The offset of the block
    pub address: Gs2BytecodeAddress,
}

impl fmt::Display for BasicBlockId {
    /// Display the `BasicBlockId` as `block_{index}`.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Block{}", self.index)
    }
}

/// Represents the edge type between two basic blocks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BasicBlockConnectionType {
    /// The connection represents a conditional branch.
    Conditional,

    /// The edge represents a fallthrough.
    Fallthrough,

    /// The edge represents an unconditional branch.
    Unconditional,

    /// The edge represents the start of a "With" block.
    With,

    /// The edge represents the start of a "ForEach" block.
    ForEach,

    /// The edge represents a short-circuit
    ShortCircuit,
}

/// Represents an edge between two basic blocks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BasicBlockConnection {
    /// The type of the connection.
    pub connection_type: BasicBlockConnectionType,
}

impl BasicBlockId {
    /// Create a new `BasicBlockId`.
    ///
    /// # Arguments
    /// - `index`: The index of the basic block in the function.
    ///
    /// # Returns
    /// - A new `BasicBlockId` instance.
    ///
    /// Example
    /// ```
    /// use gbf_rs::basic_block::BasicBlockId;
    /// use gbf_rs::basic_block::BasicBlockType;
    ///
    /// let block = BasicBlockId::new(0, BasicBlockType::Normal, 0);
    /// ```
    pub fn new(index: usize, block_type: BasicBlockType, offset: Gs2BytecodeAddress) -> Self {
        Self {
            index,
            block_type,
            address: offset,
        }
    }
}

/// Represents a basic block in a function.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BasicBlock {
    /// The identifier of the basic block.
    pub id: BasicBlockId,
    /// The instructions in the basic block.
    pub instructions: Vec<Instruction>,
}

impl BasicBlock {
    /// Create a new `BasicBlock`.
    ///
    /// # Arguments
    /// - `id`: The `BasicBlockId` of the block.
    ///
    /// # Returns
    /// - A new `BasicBlock` instance.
    ///
    /// # Example
    /// ```
    /// use gbf_rs::basic_block::{BasicBlock, BasicBlockId, BasicBlockType};
    ///
    /// let block = BasicBlock::new(BasicBlockId::new(0, BasicBlockType::Normal, 0));
    /// ```
    pub fn new(id: BasicBlockId) -> Self {
        Self {
            id,
            instructions: Vec::new(),
        }
    }

    /// Add an instruction to the basic block.
    ///
    /// # Arguments
    /// - `instruction`: The instruction to add.
    ///
    /// # Example
    /// ```
    /// use gbf_rs::basic_block::{BasicBlock, BasicBlockId, BasicBlockType};
    /// use gbf_rs::instruction::Instruction;
    /// use gbf_rs::opcode::Opcode;
    /// use gbf_rs::operand::Operand;
    ///
    /// let mut block = BasicBlock::new(BasicBlockId::new(0, BasicBlockType::Normal, 0));
    /// block.add_instruction(Instruction::new_with_operand(Opcode::PushNumber, 0, Operand::new_number(42)));
    /// ```
    pub fn add_instruction(&mut self, instruction: Instruction) {
        self.instructions.push(instruction);
    }

    /// Gets the last instruction in the block.
    ///
    /// # Returns
    /// - A reference to the last instruction in the block
    ///
    /// # Example
    /// ```
    /// use gbf_rs::basic_block::{BasicBlock, BasicBlockId, BasicBlockType};
    /// use gbf_rs::instruction::Instruction;
    /// use gbf_rs::opcode::Opcode;
    ///
    /// let mut block = BasicBlock::new(BasicBlockId::new(0, BasicBlockType::Normal, 0));
    /// block.add_instruction(Instruction::new(Opcode::PushNumber, 0));
    /// block.add_instruction(Instruction::new(Opcode::Ret, 1));
    /// let last_instruction = block.last_instruction();
    /// assert_eq!(last_instruction.unwrap().opcode, Opcode::Ret);
    /// ```
    pub fn last_instruction(&self) -> Option<&Instruction> {
        self.instructions.last()
    }

    /// Find an instruction based on a predicate.
    ///
    /// # Arguments
    /// - `predicate`: The predicate to use to find the instruction.
    ///
    /// # Returns
    /// A reference to the instruction if found, or `None` if not found.
    ///
    /// # Example
    /// ```
    /// use gbf_rs::basic_block::{BasicBlock, BasicBlockId, BasicBlockType};
    /// use gbf_rs::instruction::Instruction;
    /// use gbf_rs::opcode::Opcode;
    /// use gbf_rs::operand::Operand;
    ///
    /// let mut block = BasicBlock::new(BasicBlockId::new(0, BasicBlockType::Normal, 0));
    /// block.add_instruction(Instruction::new_with_operand(Opcode::PushNumber, 0, Operand::new_number(42)));
    /// let instruction = block.find_instruction(|i| i.opcode == Opcode::PushNumber);
    /// ```
    pub fn find_instruction<F>(&self, predicate: F) -> Option<&Instruction>
    where
        F: Fn(&Instruction) -> bool,
    {
        self.instructions.iter().find(|i| predicate(i))
    }

    /// Get the number of instructions in the block.
    ///
    /// # Returns
    /// - The number of instructions in the block.
    ///
    /// # Example
    /// ```
    /// use gbf_rs::basic_block::{BasicBlock, BasicBlockId, BasicBlockType};
    ///
    /// let mut block = BasicBlock::new(BasicBlockId::new(0, BasicBlockType::Normal, 0));
    /// assert_eq!(block.len(), 0);
    /// ```
    pub fn len(&self) -> usize {
        self.instructions.len()
    }

    /// Check if the block is empty.
    ///
    /// # Returns
    /// - `true` if the block is empty, `false` otherwise.
    ///
    /// # Example
    /// ```
    /// use gbf_rs::basic_block::{BasicBlock, BasicBlockId, BasicBlockType};
    ///
    /// let mut block = BasicBlock::new(BasicBlockId::new(0, BasicBlockType::Normal, 0));
    /// assert!(block.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.instructions.is_empty()
    }
}

/// Implement Deref
impl Deref for BasicBlock {
    type Target = Vec<Instruction>;

    /// Get a reference to the instructions in the block.
    ///
    /// # Returns
    /// - A reference to the instructions in the block.
    fn deref(&self) -> &Self::Target {
        &self.instructions
    }
}

/// Allow iterating over `BasicBlock` (owned) to consume it and get owned instructions.
impl IntoIterator for BasicBlock {
    type Item = Instruction;
    type IntoIter = vec::IntoIter<Instruction>;

    /// Create an iterator over the instructions in the block.
    ///
    /// # Returns
    /// - An iterator over the instructions in the block.
    fn into_iter(self) -> Self::IntoIter {
        self.instructions.into_iter()
    }
}

impl Index<usize> for BasicBlock {
    type Output = Instruction;

    /// Get an instruction by index.
    ///
    /// # Arguments
    /// - `index`: The index of the instruction to get.
    ///
    /// # Returns
    /// - A reference to the instruction at the given index.
    fn index(&self, index: usize) -> &Self::Output {
        &self.instructions[index]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{instruction::Instruction, opcode::Opcode, operand::Operand};

    #[test]
    fn test_basic_block_id_display() {
        let block = BasicBlockId::new(0, BasicBlockType::Normal, 3);
        assert_eq!(block.to_string(), "Block0");
    }

    #[test]
    fn test_basic_block_new() {
        let block = BasicBlock::new(BasicBlockId::new(0, BasicBlockType::Normal, 4));
        assert_eq!(block.id.index, 0);
        assert_eq!(block.id.block_type, BasicBlockType::Normal);
        assert!(block.instructions.is_empty());
    }

    #[test]
    fn test_basic_block_add_instruction() {
        let mut block = BasicBlock::new(BasicBlockId::new(0, BasicBlockType::Normal, 7));
        block.add_instruction(Instruction::new_with_operand(
            Opcode::PushNumber,
            0,
            Operand::new_number(42),
        ));
        assert_eq!(block.instructions.len(), 1);
    }

    #[test]
    fn test_basic_block_find_instruction() {
        let mut block = BasicBlock::new(BasicBlockId::new(0, BasicBlockType::Normal, 10));
        block.add_instruction(Instruction::new_with_operand(
            Opcode::PushNumber,
            0,
            Operand::new_number(42),
        ));
        let instruction = block.find_instruction(|i| i.opcode == Opcode::PushNumber);
        assert!(instruction.is_some());
    }

    #[test]
    fn test_basic_block_len() {
        let mut block = BasicBlock::new(BasicBlockId::new(0, BasicBlockType::Normal, 32));
        block.add_instruction(Instruction::new_with_operand(
            Opcode::PushNumber,
            0,
            Operand::new_number(42),
        ));
        assert_eq!(block.len(), 1);
    }

    #[test]
    fn test_basic_block_is_empty() {
        let block = BasicBlock::new(BasicBlockId::new(0, BasicBlockType::Normal, 23));
        assert!(block.is_empty());
    }

    #[test]
    fn test_basic_block_into_iter() {
        let mut block = BasicBlock::new(BasicBlockId::new(0, BasicBlockType::Normal, 11));
        block.add_instruction(Instruction::new_with_operand(
            Opcode::PushNumber,
            0,
            Operand::new_number(42),
        ));
        block.add_instruction(Instruction::new_with_operand(
            Opcode::PushNumber,
            1,
            Operand::new_number(42),
        ));
        let mut iter = block.into_iter();
        assert_eq!(iter.next().unwrap().address, 0);
        assert_eq!(iter.next().unwrap().address, 1);
        assert!(iter.next().is_none());
    }

    #[test]
    fn test_basic_block_into_iter_ref() {
        let mut block = BasicBlock::new(BasicBlockId::new(0, BasicBlockType::Normal, 42));
        block.add_instruction(Instruction::new_with_operand(
            Opcode::PushNumber,
            0,
            Operand::new_number(42),
        ));
        block.add_instruction(Instruction::new_with_operand(
            Opcode::PushNumber,
            1,
            Operand::new_number(42),
        ));
        let mut iter = block.iter();
        assert_eq!(iter.next().unwrap().address, 0);
        assert_eq!(iter.next().unwrap().address, 1);
        assert!(iter.next().is_none());
    }
}
