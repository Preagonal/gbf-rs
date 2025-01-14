#![deny(missing_docs)]

//! This crate provides basic block definitions, function definitions, module definitions,
//! graph definitions, instruction definitions, opcode definitions, and operand definitions.

use bytecode_loader::{BytecodeLoaderBuilder, BytecodeLoaderError};

/// This module contains basic block definitions and operations.
pub mod basic_block;
/// This module reads bytecode from a reader and disassembles it.
pub mod bytecode_loader;
/// This module contains the logic to visualize the control flow graph of a module.
pub mod cfg_dot;
/// Decompiler module
pub mod decompiler;
/// This module contains the definition of a function.
pub mod function;
/// This module contains the definition of Graal IO.
pub mod graal_io;
/// This module contains the definition of an instruction.
pub mod instruction;
/// This module contains the definition of a module.
pub mod module;
/// This module contains the definition of an opcode.
pub mod opcode;
/// This module contains the definition of an operand.
pub mod operand;
/// This module contains utility functions and types.
pub mod utils;

/// Disassemble bytecode using a reader.
///
/// # Arguments
/// - `reader`: The reader to read the bytecode from.
///
/// # Returns
/// - The string representation of the disassembled bytecode.
///
/// # Errors
/// - `BytecodeLoaderError`: An error occurred while loading the bytecode.
///
/// # Examples
/// ```
/// use gbf_core::disassemble_bytecode;
///
/// // read from a file
/// let reader = std::fs::File::open("tests/gs2bc/simple.gs2bc").unwrap();
/// let result = disassemble_bytecode(reader).unwrap();
/// ```
pub fn disassemble_bytecode<R: std::io::Read>(reader: R) -> Result<String, BytecodeLoaderError> {
    // create a new bytecode loader builder
    let loader = BytecodeLoaderBuilder::new(reader).build()?;

    // write a string representation of the bytecode using each instruction in the instructions vec
    let mut result = String::new();
    for (index, instruction) in loader.instructions.iter().enumerate() {
        result.push_str(&format!("{:08x}: {}\n", index, instruction));
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disassemble() {
        let reader = std::io::Cursor::new(vec![
            0x00, 0x00, 0x00, 0x01, // Section type: Gs1Flags
            0x00, 0x00, 0x00, 0x04, // Length: 4
            0x00, 0x00, 0x00, 0x00, // Flags: 0
            0x00, 0x00, 0x00, 0x02, // Section type: Functions
            0x00, 0x00, 0x00, 0x09, // Length: 9
            0x00, 0x00, 0x00, 0x00, // Function location: 0
            0x6d, 0x61, 0x69, 0x6e, // Function name: "main"
            0x00, // Null terminator
            0x00, 0x00, 0x00, 0x03, // Section type: Strings
            0x00, 0x00, 0x00, 0x04, // Length: 4
            0x61, 0x62, 0x63, 0x00, // String: "abc"
            0x00, 0x00, 0x00, 0x04, // Section type: Instructions
            0x00, 0x00, 0x00, 0x0c, // Length: 12
            0x01, // Opcode: Jmp
            0xF3, // Opcode: ImmByte
            0x01, // Operand: 1
            0x14, // Opcode: PushNumber
            0xF4, // Opcode: ImmShort
            0x00, 0x01, // Operand: 1
            0x15, // Opcode: PushString
            0xF0, // Opcode: ImmStringByte
            0x00, // Operand: 0
            0x1b, // Opcode: PushPi
            0x07, // Opcode: Ret
        ]);

        let result = disassemble_bytecode(reader).unwrap();

        assert_eq!(
            result,
            "00000000: Jmp 0x1\n\
            00000001: PushNumber 0x1\n\
            00000002: PushString abc\n\
            00000003: Pi\n\
            00000004: Ret\n"
        );

        // test failure case
        let reader = std::io::Cursor::new(vec![0x00, 0x00, 0x00, 0x01]);
        let result = disassemble_bytecode(reader);
        assert!(result.is_err());
    }
}
