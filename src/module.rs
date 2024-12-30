#![deny(missing_docs)]

use std::collections::BTreeMap;
use thiserror::Error;

use crate::{
    basic_block::BasicBlockType,
    bytecode_loader::{self, BytecodeLoaderError},
    function::{Function, FunctionId},
    utils::Gs2BytecodeAddress,
};

/// Error type for module operations.
#[derive(Error, Debug)]
pub enum ModuleError {
    /// Error for when a function is not found in the module.
    #[error("Function not found: {0}")]
    FunctionNotFound(FunctionId),

    /// Error for when a "nameless" or entry module is defined more than once.
    #[error("Entry module created more than once")]
    EntryModuleDefinedMoreThanOnce,

    /// Error for when the bytecode loader fails to load bytecode.
    #[error("BytecodeLoaderError: {0}")]
    BytecodeLoaderError(#[from] BytecodeLoaderError),
}

/// Represents a builder for a `Module`.
pub struct ModuleBuilder {
    name: Option<String>,
    reader: Option<Box<dyn std::io::Read>>,
}

impl ModuleBuilder {
    /// Create a new `ModuleBuilder`.
    ///
    /// # Arguments
    /// - `name`: The name of the module.
    ///
    /// # Returns
    /// - A new `ModuleBuilder` instance.
    ///
    /// # Example
    /// ```
    /// use gbf_rs::module::ModuleBuilder;
    ///
    /// let builder = ModuleBuilder::new();
    /// ```
    pub fn new() -> Self {
        Self {
            name: None,
            reader: None,
        }
    }
    /// Set the name of the module.
    ///
    /// # Arguments
    /// - `name`: The name of the module.
    ///
    /// # Returns
    /// - A reference to the builder.
    ///
    /// # Example
    /// ```
    /// use gbf_rs::module::ModuleBuilder;
    ///
    /// let builder = ModuleBuilder::new().name("test".to_string());
    /// ```
    pub fn name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    /// Set the reader for the module.
    ///
    /// # Arguments
    /// - `reader`: The reader to use for the module.
    ///
    /// # Returns
    /// - A reference to the builder.
    ///
    /// # Example
    /// ```
    /// use gbf_rs::module::ModuleBuilder;
    ///
    /// let builder = ModuleBuilder::new().reader(Box::new(std::io::Cursor::new(vec![0x00, 0x01])));
    /// ```
    pub fn reader(mut self, reader: Box<dyn std::io::Read>) -> Self {
        self.reader = Some(reader);
        self
    }

    /// Build the `Module` from the builder.
    ///
    /// # Returns
    /// - A new `Module` instance.
    ///
    /// # Example
    /// ```
    /// use gbf_rs::module::ModuleBuilder;
    ///
    /// let module = ModuleBuilder::new().name("test".to_string()).build().unwrap();
    /// ```
    pub fn build(self) -> Result<Module, ModuleError> {
        let mut module = Module {
            name: self.name,
            functions: BTreeMap::new(),
            entry_function: None,
            name_to_function: BTreeMap::new(),
        };

        // Create entry function
        let fun_id = FunctionId::new(0, None, 0);

        // Create new function struct
        module
            .functions
            .insert(fun_id.clone(), Function::new(fun_id.clone()));

        module.name_to_function.insert(None, fun_id.clone());

        module.entry_function = Some(fun_id);

        if let Some(reader) = self.reader {
            module.load(reader)?;
        }

        Ok(module)
    }
}

impl Default for ModuleBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Represents a GS2 module in a bytecode system. A module contains
/// functions, strings, and other data.
pub struct Module {
    /// The name of the module.
    pub name: Option<String>,
    functions: BTreeMap<FunctionId, Function>,
    name_to_function: BTreeMap<Option<String>, FunctionId>,
    entry_function: Option<FunctionId>,
}

impl Module {
    /// Create a new function in the module.
    ///
    /// # Returns
    /// - The `FunctionId` of the new function.
    ///
    /// # Errors
    /// - `ModuleError::EntryModuleDefinedMoreThanOnce` if the entry function is already set.
    ///
    /// # Example
    /// ```
    /// use gbf_rs::module::ModuleBuilder;
    ///
    /// let mut module = ModuleBuilder::new().name("test.gs2".to_string()).build().unwrap();
    /// let function_id = module.create_function("test_function".to_string(), 123).unwrap();
    /// ```
    pub fn create_function(
        &mut self,
        name: String,
        offset: Gs2BytecodeAddress,
    ) -> Result<FunctionId, ModuleError> {
        // If offset is 0, throw error
        if offset == 0 {
            return Err(ModuleError::EntryModuleDefinedMoreThanOnce);
        }
        let function_id = FunctionId::new(self.functions.len(), Some(name.clone()), offset);

        // Create new function struct
        self.functions
            .insert(function_id.clone(), Function::new(function_id.clone()));
        self.name_to_function
            .insert(Some(name.clone()), function_id.clone());
        Ok(function_id)
    }

    /// Check if the function exists in the module
    ///
    /// # Arguments
    /// - `name`: The name of the function to check.
    ///
    /// # Returns
    /// - A boolean indicating if the function exists.
    ///
    /// # Example
    /// ```
    /// use gbf_rs::module::ModuleBuilder;
    ///
    /// let mut module = ModuleBuilder::new().name("test.gs2".to_string()).build().unwrap();
    /// let function_id = module.create_function("test_function".to_string(), 123).unwrap();
    /// assert!(module.has_function(Some("test_function".to_string())));
    /// ```
    pub fn has_function(&self, name: Option<String>) -> bool {
        // Entry function is always present
        if name.is_none() {
            return true;
        }
        let name = name.unwrap();

        self.functions
            .keys()
            .any(|id| (id.name.as_ref() == Some(&name)))
    }

    /// Get a function by its `FunctionId`.
    ///
    /// # Arguments
    /// - `id`: The `FunctionId` of the function to retrieve.
    ///
    /// # Returns
    /// - A reference to the function, if it exists.
    ///
    /// # Errors
    /// - `ModuleError::FunctionNotFound` if the function does not exist.
    ///
    /// # Example
    /// ```
    /// use gbf_rs::module::ModuleBuilder;
    ///
    /// let mut module = ModuleBuilder::new().name("test.gs2".to_string()).build().unwrap();
    /// let function_id = module.create_function("test_function".to_string(), 123).unwrap();
    /// let function = module.get_function(&function_id).unwrap();
    /// ```
    pub fn get_function(&self, id: &FunctionId) -> Result<&Function, ModuleError> {
        self.functions
            .get(id)
            .ok_or(ModuleError::FunctionNotFound(id.clone()))
    }

    /// Get a mutable reference to a function by its `FunctionId`.
    ///
    /// # Arguments
    /// - `id`: The `FunctionId` of the function to retrieve.
    ///
    /// # Returns
    /// - A mutable reference to the function, if it exists.
    ///
    /// # Errors
    /// - `ModuleError::FunctionNotFound` if the function does not exist.
    ///
    /// # Example
    /// ```
    /// use gbf_rs::module::ModuleBuilder;
    ///
    /// let mut module = ModuleBuilder::new().name("test.gs2".to_string()).build().unwrap();
    /// let function_id = module.create_function("test_function".to_string(), 123).unwrap();
    /// let function = module.get_function_mut(&function_id).unwrap();
    /// ```
    pub fn get_function_mut(&mut self, id: &FunctionId) -> Result<&mut Function, ModuleError> {
        self.functions
            .get_mut(id)
            .ok_or(ModuleError::FunctionNotFound(id.clone()))
    }

    /// Get function id by name
    ///
    /// # Arguments
    /// - `name`: The name of the function to retrieve.
    ///
    /// # Returns
    /// - The `FunctionId` of the function, if it exists.
    pub fn get_function_id_by_name(&self, name: Option<String>) -> Option<FunctionId> {
        self.name_to_function.get(&name).cloned()
    }

    /// Load bytecode into the module using a reader.
    ///
    /// # Arguments
    /// - `reader`: The reader to use to load the bytecode.
    ///
    /// # Errors
    /// - `ModuleError::BytecodeLoaderError` if the bytecode loader fails to load the bytecode.
    /// - `ModuleError::EntryModuleDefinedMoreThanOnce` if the entry function is already set.
    fn load<R: std::io::Read>(&mut self, reader: R) -> Result<(), ModuleError> {
        let loaded_bytecode = bytecode_loader::BytecodeLoaderBuilder::new(reader).build()?;

        // Iterate through each instruction in the bytecode
        for (offset, instruction) in loaded_bytecode.instructions.iter().enumerate() {
            let function_name = loaded_bytecode.get_function_name_for_address(offset);

            // Create new function if it doesn't exist
            if !self.has_function(function_name.clone()) {
                debug_assert!(function_name.is_some());
                let function_name = function_name.clone().unwrap();
                self.create_function(
                    function_name.clone(),
                    *loaded_bytecode
                        .function_map
                        .get(&function_name.clone())
                        // TODO: Safely unwrap since we know the function exists
                        .unwrap(),
                )?;
            }

            // Create BasicBlock for the function if it doesn't exist
            // We can safely unwrap here since we know the function exists
            let function_id = self.get_function_id_by_name(function_name).unwrap();

            // Get the function reference
            let function = self.functions.get_mut(&function_id).unwrap();

            // Get the start address for the basic block
            let start_address = loaded_bytecode.find_block_start_address(offset);

            // Create new basic block if it doesn't exist
            if !function.has_basic_block_by_address(start_address) {
                // We won't run into this error because we are not making an entry block here
                function
                    .create_block(BasicBlockType::Normal, start_address)
                    .unwrap();
            }

            // Get the basic block reference
            let block_id = function.get_block_id_by_address(start_address).unwrap();
            let block = function.get_block_mut(block_id).unwrap();

            // Add the instruction to the basic block
            block.add_instruction(instruction.clone());
        }

        // To the entry block, let's create a new basic block with address set to the length of the bytecode
        // This is the block that will be used to represent the end of the module
        let entry = self.get_function_id_by_name(None);
        let entry = self.get_function_mut(&entry.unwrap()).unwrap();
        entry
            .create_block(
                BasicBlockType::ModuleEnd,
                loaded_bytecode.instructions.len() as Gs2BytecodeAddress,
            )
            .unwrap();

        // Iterate through each function that was created. For each function, we will iterate through
        // each basic block and find the terminator instruction. Based on the terminator opcode,
        // we will connect edges in the graph.
        for function in self.functions.values_mut() {
            let block_ids = function.get_block_ids();
            for block_id in block_ids {
                let block = function.get_block(block_id).unwrap();
                // TODO: Check explicit clone here
                let terminator = block.last_instruction().cloned();

                // TODO: Handle this better, blocks should always have a terminator
                if terminator.is_none() {
                    continue;
                }

                // Get the terminator instruction
                let terminator = terminator.unwrap();
                let terminator_opcode = terminator.opcode;
                let terminator_operand = terminator.operand;
                let terminator_address = terminator.address;

                if terminator_opcode.has_fall_through() {
                    // Get the next block address
                    let next_block_address = terminator_address + 1 as Gs2BytecodeAddress;
                    // TODO: Double check unwrap here
                    let next_block_id = function
                        .get_block_id_by_address(next_block_address)
                        .unwrap();

                    // Connect the blocks
                    // TODO: Double check unwrap here
                    function.add_edge(block_id, next_block_id).unwrap();
                }

                if terminator_opcode.has_jump_target() {
                    // Get the branch address
                    let branch_address = terminator_operand.unwrap().get_number_value().unwrap()
                        as Gs2BytecodeAddress;
                    // TODO: Double check unwrap here
                    let branch_block_id = function.get_block_id_by_address(branch_address).unwrap(); // TODO: Double check unwrap here

                    // Connect the blocks
                    // TODO: Check unwrap here
                    function.add_edge(block_id, branch_block_id).unwrap();
                }
            }
        }
        Ok(())
    }

    /// Get the number of functions in the module.
    ///
    /// # Returns
    /// - The number of functions in the module.
    ///
    /// # Example
    /// ```
    /// use gbf_rs::module::ModuleBuilder;
    ///
    /// let mut module = ModuleBuilder::new().name("test.gs2".to_string()).build().unwrap();
    /// let function_id = module.create_function("test_function".to_string(), 123).unwrap();
    /// assert_eq!(module.len(), 2);
    /// ```
    pub fn len(&self) -> usize {
        self.functions.len()
    }

    /// Check if the `Module` is empty.
    ///
    /// # Returns
    /// - A boolean indicating if the `Module` is empty.
    ///
    /// # Example
    /// ```
    /// use gbf_rs::module::ModuleBuilder;
    ///
    /// let module = ModuleBuilder::new().name("test.gs2".to_string()).build().unwrap();
    /// assert!(!module.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        // The module will always have an entry function, so this is always false
        self.functions.is_empty()
    }
}

// Deref implementation for Module
impl std::ops::Deref for Module {
    type Target = BTreeMap<FunctionId, Function>;

    fn deref(&self) -> &Self::Target {
        &self.functions
    }
}

// Index implementation for Module using usize
impl std::ops::Index<usize> for Module {
    type Output = Function;

    fn index(&self, index: usize) -> &Self::Output {
        self.functions.values().nth(index).unwrap()
    }
}

// IntoIterator implementation for Module
impl IntoIterator for Module {
    type Item = (FunctionId, Function);
    type IntoIter = std::collections::btree_map::IntoIter<FunctionId, Function>;

    fn into_iter(self) -> Self::IntoIter {
        self.functions.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_bytecode() {
        let bytecode = [
            0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00,
        ];
        // make new module with generics
        let module = ModuleBuilder::new()
            .reader(Box::new(std::io::Cursor::new(bytecode.to_vec())))
            .build();

        assert!(module.is_ok());

        // test failure case
        let bytecode = [0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x04];
        let module = ModuleBuilder::new()
            .reader(Box::new(std::io::Cursor::new(bytecode.to_vec())))
            .build();
        assert!(module.is_err());
    }
}
