#![deny(missing_docs)]

use serde::{Deserialize, Serialize};

use crate::opcode::Opcode;
use crate::operand::Operand;
use crate::utils::Gs2BytecodeAddress;

/// Represents an instruction in a bytecode system.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Instruction {
    /// The opcode of the instruction.
    pub opcode: Opcode,

    /// The address of the instruction.
    pub address: Gs2BytecodeAddress,

    /// The operand of the instruction, if any.
    pub operand: Option<Operand>,
}

impl Instruction {
    /// Create a new `Instruction`.
    ///
    /// # Arguments
    /// - `opcode`: The opcode of the instruction.
    /// - `address`: The address of the instruction.
    ///
    /// # Returns
    /// - A new `Instruction` instance.
    pub fn new(opcode: Opcode, address: usize) -> Self {
        Self {
            opcode,
            address,
            operand: None,
        }
    }

    /// Create a new `Instruction` with an operand.
    ///
    /// # Arguments
    /// - `opcode`: The opcode of the instruction.
    /// - `address`: The address of the instruction.
    /// - `operand`: The operand of the instruction.
    ///
    /// # Returns
    /// - A new `Instruction` instance.
    pub fn new_with_operand(opcode: Opcode, address: usize, operand: Operand) -> Self {
        Self {
            opcode,
            address,
            operand: Some(operand),
        }
    }

    /// Set the operand of the instruction.
    ///
    /// # Arguments
    /// - `operand`: The operand to set.
    ///
    /// # Example
    /// ```
    /// use gbf_core::instruction::Instruction;
    /// use gbf_core::operand::Operand;
    /// use gbf_core::opcode::Opcode;
    ///
    /// let mut instruction = Instruction::new(Opcode::PushNumber, 0);
    /// instruction.set_operand(Operand::new_number(42));
    /// ```
    pub fn set_operand(&mut self, operand: Operand) {
        self.operand = Some(operand);
    }
}

/// Implement the `Display` trait for `Instruction`.
impl std::fmt::Display for Instruction {
    /// Convert the Instruction to a string
    ///
    /// # Returns
    /// - A string representation of the instruction.
    ///
    /// # Example
    /// ```
    /// use gbf_core::instruction::Instruction;
    /// use gbf_core::operand::Operand;
    /// use gbf_core::opcode::Opcode;
    ///
    /// let instruction = Instruction::new_with_operand(Opcode::PushNumber, 0, Operand::new_number(42));
    /// let string = instruction.to_string();
    /// assert_eq!(string, "PushNumber 0x2a");
    /// ```
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match &self.operand {
            Some(operand) => write!(f, "{} {}", self.opcode, operand),
            None => write!(f, "{}", self.opcode),
        }
    }
}

impl Default for Instruction {
    fn default() -> Self {
        Self {
            // TODO: Change this to a more appropriate default opcode
            opcode: Opcode::ConvertToFloat,
            address: 0,
            operand: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::opcode::Opcode;

    #[test]
    fn instruction_to_string() {
        let instruction = Instruction::new(Opcode::PushNumber, 0);
        assert_eq!(instruction.to_string(), "PushNumber");

        let instruction =
            Instruction::new_with_operand(Opcode::PushNumber, 0, Operand::new_number(42));
        assert_eq!(instruction.to_string(), "PushNumber 0x2a");

        let instruction = Instruction::new_with_operand(
            Opcode::PushString,
            0,
            Operand::new_string("Hello, world!"),
        );
        assert_eq!(instruction.to_string(), "PushString Hello, world!");

        let instruction =
            Instruction::new_with_operand(Opcode::PushNumber, 0, Operand::new_float("3.14"));
        assert_eq!(instruction.to_string(), "PushNumber 3.14");
    }
}
