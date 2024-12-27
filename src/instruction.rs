use serde::{Deserialize, Serialize};

use crate::opcode::Opcode;
use crate::operand::Operand;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Instruction {
    pub opcode: Opcode,
    pub address: usize,
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

    /// Convert the Instruction to a string
    ///
    /// # Returns
    /// - A string representation of the instruction.
    ///
    /// # Example
    /// ```
    /// use gbf_rs::instruction::Instruction;
    /// use gbf_rs::operand::Operand;
    /// use gbf_rs::opcode::Opcode;
    ///
    /// let instruction = Instruction::new_with_operand(Opcode::PushNumber, 0, Operand::new_int(42));
    /// let string = instruction.to_string();
    /// assert_eq!(string, "PushNumber 42");
    /// ```
    pub fn to_string(&self) -> String {
        match &self.operand {
            Some(operand) => format!("{} {}", self.opcode.to_string(), operand.to_string()),
            None => self.opcode.to_string().to_string(),
        }
    }
}

/// Implement the `Display` trait for `Instruction`.
impl std::fmt::Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
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
            Instruction::new_with_operand(Opcode::PushNumber, 0, Operand::new_int(42));
        assert_eq!(instruction.to_string(), "PushNumber 42");

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
