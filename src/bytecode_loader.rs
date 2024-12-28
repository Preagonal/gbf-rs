#![deny(missing_docs)]

use crate::{
    graal_io::{GraalIoError, GraalReader},
    instruction::Instruction,
    opcode::{Opcode, OpcodeError},
    operand::Operand,
    utils::Gs2BytecodeAddress,
};
use std::{collections::HashMap, io::Read};

use thiserror::Error;

/// Error type for bytecode operations.
#[derive(Error, Debug)]
pub enum BytecodeLoaderError {
    /// Error for when an invalid section type is encountered.
    #[error("Invalid section type: {0}")]
    InvalidSectionType(u32),

    /// Error for when an invalid section length is encountered.
    #[error("Invalid section length for {0}: {1}")]
    InvalidSectionLength(SectionType, u32),

    /// Error when string index is out of bounds.
    #[error("String index {0} is out of bounds. Length: {1}")]
    StringIndexOutOfBounds(usize, usize),

    /// Error for when there is no previous instruction when setting an operand.
    #[error("No previous instruction to set operand")]
    NoPreviousInstruction,

    /// Error for when an I/O error occurs.
    #[error("GraalIo error: {0}")]
    GraalIo(#[from] GraalIoError),

    /// Error for when an invalid opcode is encountered.
    #[error("Invalid opcode: {0}")]
    OpcodeError(#[from] OpcodeError),
}

impl std::fmt::Display for SectionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SectionType::Gs1Flags => write!(f, "Gs1Flags"),
            SectionType::Functions => write!(f, "Functions"),
            SectionType::Strings => write!(f, "Strings"),
            SectionType::Instructions => write!(f, "Instructions"),
        }
    }
}

/// Represents a section type in a module.
#[derive(Debug)]
#[repr(u32)]
pub enum SectionType {
    /// The section contains flags for the module.
    Gs1Flags = 1,

    /// The section contains the module's functions.
    Functions = 2,

    /// The section contains the module's strings.
    Strings = 3,

    /// The section contains the module's instructions.
    Instructions = 4,
}

/// A structure for loading bytecode from a reader.
pub struct BytecodeLoader<R: Read> {
    reader: GraalReader<R>,
    strings: Vec<String>,

    /// A map of function names to their addresses.
    pub function_map: HashMap<String, Gs2BytecodeAddress>,

    /// The instructions in the module.
    pub instructions: Vec<Instruction>,
}

impl<R: Read> BytecodeLoader<R> {
    /// Creates a new BytecodeLoader.
    ///
    /// # Arguments
    /// - `reader`: The reader to read the bytecode from.
    ///
    /// # Returns
    /// - A new `BytecodeLoader` instance.
    ///
    /// # Example
    /// ```
    /// use gbf_rs::bytecode_loader::BytecodeLoader;
    ///
    /// let reader = std::io::Cursor::new(vec![0x00, 0x00, 0x00, 0x00]);
    /// let loader = BytecodeLoader::new(reader);
    /// ```
    pub fn new(reader: R) -> Self {
        Self {
            reader: GraalReader::new(reader),
            function_map: HashMap::new(),
            strings: Vec::new(),
            instructions: Vec::new(),
        }
    }

    /// Reads the flags section from the reader.
    ///
    /// We don't actually need to do anything with the flags section, so we just read it and ignore it.
    fn read_gs1_flags(&mut self) -> Result<(), BytecodeLoaderError> {
        let section_length = self.reader.read_u32().map_err(BytecodeLoaderError::from)?;
        let _flags = self.reader.read_u32().map_err(BytecodeLoaderError::from)?;

        // assert that the section length is correct
        if section_length != 4 {
            return Err(BytecodeLoaderError::InvalidSectionLength(
                SectionType::Gs1Flags,
                section_length,
            ));
        }

        Ok(())
    }

    /// Reads the functions section from the reader. This section contains the names of the functions
    /// in the module.
    ///
    /// # Returns
    /// - A `Result` indicating success or failure.
    ///
    /// # Errors
    /// - `BytecodeLoaderError::InvalidSectionLength` if the section length is incorrect.
    /// - `BytecodeLoaderError::GraalIo` if an I/O error occurs.
    fn read_functions(&mut self) -> Result<(), BytecodeLoaderError> {
        let section_length = self.reader.read_u32().map_err(BytecodeLoaderError::from)?;

        // For each function, use self.reader.read_u32() to get the location of the function,
        // and then use self.reader.read_string() to get the name of the function. We should
        // only read up to section_length bytes.
        let mut bytes_read = 0;
        while bytes_read < section_length {
            let function_location =
                self.reader.read_u32().map_err(BytecodeLoaderError::from)? as Gs2BytecodeAddress;
            let function_name = self
                .reader
                .read_string()
                .map_err(BytecodeLoaderError::from)?;
            self.function_map
                .insert(function_name.clone(), function_location);
            bytes_read += 4 + function_name.len() as u32;
            bytes_read += 1; // Null terminator
        }

        // assert that the section length is correct
        if bytes_read != section_length {
            return Err(BytecodeLoaderError::InvalidSectionLength(
                SectionType::Functions,
                section_length,
            ));
        }

        Ok(())
    }

    /// Reads the strings section from the reader. This section contains the strings used in the module.
    ///
    /// # Returns
    /// - A `Result` indicating success or failure.
    ///
    /// # Errors
    /// - `BytecodeLoaderError::GraalIo` if an I/O error occurs.
    /// - `BytecodeLoaderError::InvalidSectionLength` if the section length is incorrect.
    fn read_strings(&mut self) -> Result<(), BytecodeLoaderError> {
        let section_length = self.reader.read_u32().map_err(BytecodeLoaderError::from)?;

        // For each string, use self.reader.read_string() to get the string. We should only read up to section_length bytes.
        let mut bytes_read = 0;
        while bytes_read < section_length {
            let string = self
                .reader
                .read_string()
                .map_err(BytecodeLoaderError::from)?;
            self.strings.push(string.clone());
            bytes_read += string.len() as u32;
            bytes_read += 1; // Null terminator
        }

        // assert that the section length is correct
        if bytes_read != section_length {
            return Err(BytecodeLoaderError::InvalidSectionLength(
                SectionType::Strings,
                section_length,
            ));
        }

        Ok(())
    }

    /// Read one opcode from the reader and return it.
    fn read_opcode(&mut self) -> Result<Opcode, BytecodeLoaderError> {
        let opcode_byte = self.reader.read_u8().map_err(BytecodeLoaderError::from)?;
        let opcode = Opcode::from_byte(opcode_byte)?;
        Ok(opcode)
    }

    /// Read one operand from the reader and return it along with the number of bytes read.
    fn read_operand(
        &mut self,
        opcode: Opcode,
    ) -> Result<Option<(Operand, usize)>, BytecodeLoaderError> {
        match opcode {
            Opcode::ImmStringByte => {
                let string_index = self.reader.read_u8().map_err(BytecodeLoaderError::from)?;
                let string = self.strings.get(string_index as usize).ok_or(
                    BytecodeLoaderError::StringIndexOutOfBounds(
                        string_index as usize,
                        self.strings.len(),
                    ),
                )?;
                Ok(Some((Operand::new_string(string), 1)))
            }
            Opcode::ImmStringShort => {
                let string_index = self.reader.read_u16().map_err(BytecodeLoaderError::from)?;
                let string = self.strings.get(string_index as usize).ok_or(
                    BytecodeLoaderError::StringIndexOutOfBounds(
                        string_index as usize,
                        self.strings.len(),
                    ),
                )?;
                Ok(Some((Operand::new_string(string), 2)))
            }
            Opcode::ImmStringInt => {
                let string_index = self.reader.read_u32().map_err(BytecodeLoaderError::from)?;
                let string = self.strings.get(string_index as usize).ok_or(
                    BytecodeLoaderError::StringIndexOutOfBounds(
                        string_index as usize,
                        self.strings.len(),
                    ),
                )?;
                Ok(Some((Operand::new_string(string), 4)))
            }
            Opcode::ImmByte => {
                let value = self.reader.read_u8().map_err(BytecodeLoaderError::from)?;
                Ok(Some((Operand::new_int(value as i32), 1)))
            }
            Opcode::ImmShort => {
                let value = self.reader.read_u16().map_err(BytecodeLoaderError::from)?;
                Ok(Some((Operand::new_int(value as i32), 2)))
            }
            Opcode::ImmInt => {
                let value = self.reader.read_u32().map_err(BytecodeLoaderError::from)?;
                Ok(Some((Operand::new_int(value as i32), 4)))
            }
            Opcode::ImmFloat => {
                let value = self
                    .reader
                    .read_string()
                    .map_err(BytecodeLoaderError::from)?;
                Ok(Some((Operand::new_float(value.clone()), value.len() + 1)))
            }
            _ => Ok(None),
        }
    }

    /// Reads the instructions section from the reader. This section contains the bytecode instructions.
    fn read_instructions(&mut self) -> Result<(), BytecodeLoaderError> {
        let section_length = self.reader.read_u32().map_err(BytecodeLoaderError::from)?;

        // For each instruction, use self.read_opcode() to get the opcode, and then use self.read_operand() to get the operand (if any).
        // We should only read up to section_length bytes.
        let mut bytes_read = 0;
        while bytes_read < section_length {
            let opcode = self.read_opcode()?;
            bytes_read += 1;

            let operand = self.read_operand(opcode)?;
            // If the operand exists, we add the operand to the last instruction.
            // If the operand does not exist, we create a new instruction with the opcode.
            if let Some(operand) = operand {
                let last_instruction = self.instructions.last_mut();
                if let Some(last_instruction) = last_instruction {
                    last_instruction.set_operand(operand.0);
                    bytes_read += operand.1 as u32;
                } else {
                    return Err(BytecodeLoaderError::NoPreviousInstruction);
                }
            } else {
                self.instructions
                    .push(Instruction::new(opcode, self.instructions.len()));
            }
        }

        // assert that the section length is correct
        if bytes_read != section_length {
            return Err(BytecodeLoaderError::InvalidSectionLength(
                SectionType::Instructions,
                section_length,
            ));
        }

        Ok(())
    }

    /// Loads the bytecode from the reader into the structure.
    ///
    /// # Returns
    /// - A `Result` indicating success or failure.
    ///
    /// # Errors
    /// - `BytecodeLoaderError::InvalidSectionType` if an invalid section type is encountered.
    /// - `BytecodeLoaderError::InvalidSectionLength` if an invalid section length is encountered.
    /// - `BytecodeLoaderError::StringIndexOutOfBounds` if a string index is out of bounds.
    /// - `BytecodeLoaderError::NoPreviousInstruction` if there is no previous instruction when setting an operand.
    /// - `BytecodeLoaderError::GraalIo` if an I/O error occurs.
    /// - `BytecodeLoaderError::OpcodeError` if an invalid opcode is encountered.
    ///
    /// # Example
    /// ```
    /// use gbf_rs::bytecode_loader::BytecodeLoader;
    ///
    /// // fill each section with no data
    /// let reader = std::io::Cursor::new(vec![
    ///     0x00, 0x00, 0x00, 0x01, // Section type: Gs1Flags
    ///     0x00, 0x00, 0x00, 0x04, // Length: 4
    ///     0x00, 0x00, 0x00, 0x00, // Flags: 0
    ///     0x00, 0x00, 0x00, 0x02, // Section type: Functions
    ///     0x00, 0x00, 0x00, 0x00, // Length: 0
    ///     0x00, 0x00, 0x00, 0x03, // Section type: Strings
    ///     0x00, 0x00, 0x00, 0x00, // Length: 0
    ///     0x00, 0x00, 0x00, 0x04, // Section type: Instructions
    ///     0x00, 0x00, 0x00, 0x00, // Length: 0
    /// ]);
    /// let mut loader = BytecodeLoader::new(reader);
    /// loader.load().unwrap();
    /// ```
    pub fn load(&mut self) -> Result<(), BytecodeLoaderError> {
        // TODO: I know there will only be 4 sections, but I'd like to make this more dynamic.
        for _ in 0..4 {
            let section_type = self.read_section_type()?;
            match section_type {
                SectionType::Gs1Flags => {
                    self.read_gs1_flags()?;
                }
                SectionType::Functions => {
                    self.read_functions()?;
                }
                SectionType::Strings => {
                    self.read_strings()?;
                }
                SectionType::Instructions => {
                    self.read_instructions()?;
                }
            }
        }

        Ok(())
    }

    fn read_section_type(&mut self) -> Result<SectionType, BytecodeLoaderError> {
        let section_type = self.reader.read_u32().map_err(BytecodeLoaderError::from)?;
        match section_type {
            1 => Ok(SectionType::Gs1Flags),
            2 => Ok(SectionType::Functions),
            3 => Ok(SectionType::Strings),
            4 => Ok(SectionType::Instructions),
            _ => Err(BytecodeLoaderError::InvalidSectionType(section_type)),
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_load() {
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
        let mut loader = super::BytecodeLoader::new(reader);
        loader.load().unwrap();

        assert_eq!(loader.function_map.len(), 1);
        assert_eq!(loader.function_map.get("main"), Some(&0));
        assert_eq!(loader.strings.len(), 1);
        assert_eq!(loader.strings.first(), Some(&"abc".to_string()));
        assert_eq!(loader.instructions.len(), 5);
        assert_eq!(loader.instructions[0].opcode, crate::opcode::Opcode::Jmp);
        assert_eq!(
            loader.instructions[1].opcode,
            crate::opcode::Opcode::PushNumber
        );
        assert_eq!(
            loader.instructions[1].operand,
            Some(crate::operand::Operand::new_int(1))
        );
        assert_eq!(
            loader.instructions[2].opcode,
            crate::opcode::Opcode::PushString
        );
        assert_eq!(
            loader.instructions[2].operand,
            Some(crate::operand::Operand::new_string("abc"))
        );
        assert_eq!(loader.instructions[3].opcode, crate::opcode::Opcode::PushPi);
        assert_eq!(loader.instructions[4].opcode, crate::opcode::Opcode::Ret);
    }

    #[test]
    fn test_load_invalid_section_type() {
        let reader = std::io::Cursor::new(vec![0x00, 0x00, 0x00, 0x05]);
        let mut loader = super::BytecodeLoader::new(reader);
        let result = loader.load();
        assert!(result.is_err());
    }

    #[test]
    fn test_load_invalid_section_length() {
        let reader = std::io::Cursor::new(vec![
            0x00, 0x00, 0x00, 0x01, // Section type: Gs1Flags
            0x00, 0x00, 0x00, 0x05, // Length: 5
            0x00, 0x00, 0x00, 0x00, // Flags: 0
        ]);
        let mut loader = super::BytecodeLoader::new(reader);
        let result = loader.load();
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Invalid section length for Gs1Flags: 5".to_string()
        );
    }

    #[test]
    fn test_load_string_index_out_of_bounds() {
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
            0x01, // Operand: 1 (out of bounds)
            0x1b, // Opcode: PushPi
            0x07, // Opcode: Ret
        ]);

        let mut loader = super::BytecodeLoader::new(reader);
        let result = loader.load();

        assert!(result.is_err());
    }
}
