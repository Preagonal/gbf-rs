use std::{fmt::{self, Write}, vec};

use serde::{Deserialize, Serialize};

use crate::{graph::directed_graph::RenderableNode, instruction::Instruction};

use std::slice::Iter;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
    EntryAndExit
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BasicBlockId {
    index: usize,

    /// The type of the basic block.
    pub block_type: BasicBlockType
}

impl fmt::Display for BasicBlockId {
    /// Display the `BasicBlockId` as `block_{index}`.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Block{}", self.index)
    }
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
    /// let block = BasicBlockId::new(0, BasicBlockType::Normal);
    /// ```
    pub fn new(index: usize, block_type: BasicBlockType) -> Self {
        Self {
            index,
            block_type
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BasicBlock {
    pub id: BasicBlockId,
    pub instructions: Vec<Instruction>
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
    /// let block = BasicBlock::new(BasicBlockId::new(0, BasicBlockType::Normal));
    /// ```
    pub fn new(id: BasicBlockId) -> Self {
        Self {
            id,
            instructions: Vec::new()
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
    /// let mut block = BasicBlock::new(BasicBlockId::new(0, BasicBlockType::Normal));
    /// block.add_instruction(Instruction::new_with_operand(Opcode::PushNumber, 0, Operand::new_int(42)));
    /// ```
    pub fn add_instruction(&mut self, instruction: Instruction) {
        self.instructions.push(instruction);
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
    /// let mut block = BasicBlock::new(BasicBlockId::new(0, BasicBlockType::Normal));
    /// block.add_instruction(Instruction::new_with_operand(Opcode::PushNumber, 0, Operand::new_int(42)));
    /// let instruction = block.find_instruction(|i| i.opcode == Opcode::PushNumber);
    /// ```
    pub fn find_instruction<F>(&self, predicate: F) -> Option<&Instruction>
    where
        F: Fn(&Instruction) -> bool
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
    /// let mut block = BasicBlock::new(BasicBlockId::new(0, BasicBlockType::Normal));
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
    /// let mut block = BasicBlock::new(BasicBlockId::new(0, BasicBlockType::Normal));
    /// assert!(block.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.instructions.is_empty()
    }
}

/// Allow iterating over `&BasicBlock` to get immutable references to instructions.
impl<'a> IntoIterator for &'a BasicBlock {
    type Item = &'a Instruction;
    type IntoIter = Iter<'a, Instruction>;

    /// Create an iterator over the instructions in the block.
    /// 
    /// # Returns
    /// - An iterator over the instructions in the block.
    fn into_iter(self) -> Self::IntoIter {
        self.instructions.iter()
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
            r#"{indent}<TABLE BORDER="1" CELLBORDER="0" CELLSPACING="0" CELLPADDING="2">"#,
            indent = indent
        )
        .unwrap();

        // Render each instruction as a table row with indentation.
        for inst in &self.instructions {
            writeln!(
                &mut label,
                r#"{indent}    <TR>
{indent}        <TD ALIGN="LEFT">{:04X}</TD>
{indent}        <TD ALIGN="LEFT">  </TD>
{indent}        <TD ALIGN="LEFT">{}</TD>
{indent}    </TR>"#,
                inst.address,
                inst,
                indent = indent
            )
            .unwrap();
        }

        // Close the HTML-like table.
        writeln!(&mut label, "{indent}</TABLE>", indent = indent).unwrap();

        label
    }
}
