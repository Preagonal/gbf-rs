#![deny(missing_docs)]

use std::{
    fmt::{self, Write},
    ops::{Deref, Index},
    vec,
};

use serde::{Deserialize, Serialize};

use crate::{
    cfg_dot::RenderableNode,
    instruction::Instruction,
    utils::{Gs2BytecodeAddress, OPERAND_TRUNCATE_LENGTH},
};

/// Represents the type of basic block.
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

// == Implementations ==
impl fmt::Display for BasicBlockId {
    /// Display the `BasicBlockId`.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Block at address 0x{:X}", self.address)
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

/// IntoIterator implementation immutable reference
impl<'a> IntoIterator for &'a BasicBlock {
    type Item = &'a Instruction;
    type IntoIter = vec::IntoIter<&'a Instruction>;

    /// Get an iterator over the instructions in the block.
    ///
    /// # Returns
    /// - An iterator over the instructions in the block.
    fn into_iter(self) -> Self::IntoIter {
        self.instructions.iter().collect::<Vec<_>>().into_iter()
    }
}

/// IntoIterator implementation mutable reference
impl<'a> IntoIterator for &'a mut BasicBlock {
    type Item = &'a mut Instruction;
    type IntoIter = vec::IntoIter<&'a mut Instruction>;

    /// Get a mutable iterator over the instructions in the block.
    ///
    /// # Returns
    /// - A mutable iterator over the instructions in the block.
    fn into_iter(self) -> Self::IntoIter {
        self.instructions.iter_mut().collect::<Vec<_>>().into_iter()
    }
}

impl RenderableNode for BasicBlock {
    /// Render the block's node representation for Graphviz with customizable padding.
    ///
    /// # Arguments
    ///
    /// * `padding` - The number of spaces to use for indentation.
    fn render_node(&self, padding: usize) -> String {
        let mut label = String::new();
        let indent = " ".repeat(padding);

        // Start the HTML-like table for Graphviz.
        writeln!(
            &mut label,
            r#"{indent}<TABLE BORDER="0" CELLBORDER="0" CELLSPACING="0" CELLPADDING="0">"#,
            indent = indent
        )
        .unwrap();

        // Render each instruction as a table row with indentation.
        for inst in &self.instructions {
            // Get the string of an operand, if it exists, or a space.
            // If the resulting operand exceeds OPERAND_TRUNCATE_LENGTH,
            // truncate it and append an ellipsis.

            let operand = inst
                .operand
                .as_ref()
                .map(|op| {
                    let mut op_str = op.to_string();
                    if op_str.len() > OPERAND_TRUNCATE_LENGTH {
                        op_str.truncate(OPERAND_TRUNCATE_LENGTH);
                        op_str.push_str("...");
                    }
                    op_str
                })
                .unwrap_or_else(|| " ".to_string());

            writeln!(
                &mut label,
                r##"{indent}    <TR>
{indent}        <TD ALIGN="LEFT"><FONT COLOR="#bbff00">{:04X}</FONT></TD>
{indent}        <TD ALIGN="LEFT">  </TD>
{indent}        <TD ALIGN="LEFT"><FONT COLOR="#ffbb00">{}</FONT></TD>
{indent}        <TD ALIGN="LEFT">  </TD>
{indent}        <TD ALIGN="LEFT"><FONT COLOR="#00bbff">{}</FONT></TD>
{indent}    </TR>"##,
                inst.address,
                inst.opcode,
                operand,
                indent = indent
            )
            .unwrap();
        }

        // Close the HTML-like table.
        writeln!(&mut label, "{indent}</TABLE>", indent = indent).unwrap();

        label
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{instruction::Instruction, opcode::Opcode, operand::Operand};

    #[test]
    fn test_basic_block_id_display() {
        let block = BasicBlockId::new(0, BasicBlockType::Normal, 3);
        assert_eq!(block.to_string(), "Block at address 0x3");
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
