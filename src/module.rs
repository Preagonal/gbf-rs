#![deny(missing_docs)]

use std::{
    collections::HashMap,
    fmt::{self, Display, Formatter},
};
use thiserror::Error;

use crate::{
    basic_block::{BasicBlockId, BasicBlockType},
    bytecode_loader::{self, BytecodeLoaderError},
    function::{Function, FunctionId},
    instruction::Instruction,
    utils::Gs2BytecodeAddress,
};

/// Error type for module operations.
#[derive(Error, Debug)]
pub enum ModuleError {
    /// Error for when a function is not found in the module.
    #[error("Function not found: {0}")]
    FunctionNotFoundById(FunctionId),

    /// Error for when a function is not found in the module.
    #[error("Function not found: {0}")]
    FunctionNotFoundByName(String),

    /// When a function is created with a name that already exists.
    #[error("Function with name {0} already exists.")]
    DuplicateFunctionName(String),

    /// When a function is created with an address that already exists.
    #[error("Function with address {0} already exists with the name {1}.")]
    DuplicateFunctionAddress(Gs2BytecodeAddress, String),

    /// Error for when the bytecode loader fails to load bytecode.
    #[error("BytecodeLoaderError: {0}")]
    BytecodeLoaderError(#[from] BytecodeLoaderError),
}

/// Represents a builder for a `Module`.
pub struct ModuleBuilder {
    name: Option<String>,
    reader: Option<Box<dyn std::io::Read>>,
}

/// Public API for `ModuleBuilder`.
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
    /// let builder = ModuleBuilder::new().name("test");
    /// ```
    pub fn name<N: Into<String>>(mut self, name: N) -> Self {
        self.name = Some(name.into());
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
    /// let module = ModuleBuilder::new().name("test").build().unwrap();
    /// ```
    pub fn build(self) -> Result<Module, ModuleError> {
        let mut module = Module {
            name: self.name,
            functions: Vec::new(),
            id_to_index: HashMap::new(),
            name_to_id: HashMap::new(),
            address_to_id: HashMap::new(),
        };

        // Create entry function
        let fun_id = FunctionId::new(module.functions.len(), None, 0);

        // Create new function struct
        module.functions.push(Function::new(fun_id.clone()));
        module.id_to_index.insert(fun_id.clone(), 0);
        module.name_to_id.insert(None, fun_id.clone());
        module.address_to_id.insert(0, fun_id.clone());

        if let Some(reader) = self.reader {
            module.load(reader)?;
        }

        Ok(module)
    }
}

/// Represents a GS2 module in a bytecode system. A module contains
/// functions, strings, and other data.
pub struct Module {
    /// The name of the module.
    pub name: Option<String>,
    /// A list of functions in the module, which provides fast sequential access.
    functions: Vec<Function>,
    /// A map of function IDs to their index in the functions vector.
    id_to_index: HashMap<FunctionId, usize>,
    /// A map of function names to their IDs.
    name_to_id: HashMap<Option<String>, FunctionId>,
    /// A map of function addresses to their IDs.
    address_to_id: HashMap<Gs2BytecodeAddress, FunctionId>,
}

/// Public API for `Module`.
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
    /// let mut module = ModuleBuilder::new().name("test.gs2").build().unwrap();
    /// let function_id = module.create_function("test_function", 123).unwrap();
    /// ```
    pub fn create_function<N: Into<String>>(
        &mut self,
        name: N,
        address: Gs2BytecodeAddress,
    ) -> Result<FunctionId, ModuleError> {
        let name = name.into();
        let function_id = FunctionId::new(self.functions.len(), Some(name.clone()), address);

        // Check for duplicate function name
        if self.name_to_id.contains_key(&Some(name.clone())) {
            return Err(ModuleError::DuplicateFunctionName(name));
        }

        // Check for duplicate function address
        if self.address_to_id.contains_key(&address) {
            let existing_id = self.address_to_id.get(&address).unwrap().clone();
            let existing_name = existing_id.name.unwrap_or("{entry function}".to_string());
            return Err(ModuleError::DuplicateFunctionAddress(
                address,
                existing_name,
            ));
        }

        // Create new function struct
        self.functions.push(Function::new(function_id.clone()));
        self.id_to_index
            .insert(function_id.clone(), self.functions.len() - 1);
        self.name_to_id
            .insert(Some(name.clone()), function_id.clone());
        self.address_to_id.insert(address, function_id.clone());

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
    /// let mut module = ModuleBuilder::new().name("test.gs2").build().unwrap();
    /// let function_id = module.create_function("test_function", 123).unwrap();
    /// assert!(module.has_function("test_function"));
    /// ```
    pub fn has_function<N: Into<String>>(&self, name: N) -> bool {
        let name = name.into();
        self.name_to_id.contains_key(&Some(name))
    }

    /// Get function by name
    ///
    /// # Arguments
    /// - `name`: The name of the function to retrieve.
    ///
    /// # Returns
    /// - A reference to the function, if it exists.
    ///
    /// # Errors
    /// - `ModuleError::FunctionNotFoundByName` if the function does not exist.
    ///
    /// # Example
    /// ```
    /// use gbf_rs::module::ModuleBuilder;
    ///
    /// let mut module = ModuleBuilder::new().name("test.gs2").build().unwrap();
    /// let function_id = module.create_function("test_function", 123).unwrap();
    /// let function = module.get_function_by_name("test_function").unwrap();
    /// ```
    pub fn get_function_by_name<N: Into<String>>(&self, name: N) -> Result<&Function, ModuleError> {
        let name = name.into();
        let id = self.get_function_id_by_name(name)?;
        self.get_function_by_id(&id)
    }

    /// Get the entry function of the module.
    ///
    /// # Returns
    /// - A reference to the entry function.
    ///
    /// # Example
    /// ```
    /// use gbf_rs::module::ModuleBuilder;
    ///
    /// let mut module = ModuleBuilder::new().name("test.gs2").build().unwrap();
    /// let function_id = module.create_function("test_function", 123).unwrap();
    /// let entry_function = module.get_entry_function();
    /// ```
    pub fn get_entry_function(&self) -> &Function {
        // Get the function at address 0
        self.functions.first().expect("Entry function must exist")
    }

    /// Get the entry function id of the module (mutable).
    ///
    /// # Returns
    /// - A mutable reference to the entry function.
    ///
    /// # Example
    /// ```
    /// use gbf_rs::module::ModuleBuilder;
    ///
    /// let mut module = ModuleBuilder::new().name("test.gs2").build().unwrap();
    /// let function_id = module.create_function("test_function", 123).unwrap();
    /// let entry_function = module.get_entry_function_mut();
    /// ```
    pub fn get_entry_function_mut(&mut self) -> &mut Function {
        // Get the function at address 0
        self.functions
            .get_mut(0)
            .expect("Entry function must exist")
    }

    /// Get function by name (mutable)
    ///
    /// # Arguments
    /// - `name`: The name of the function to retrieve.
    ///
    /// # Returns
    /// - A mutable reference to the function, if it exists.
    ///
    /// # Errors
    /// - `ModuleError::FunctionNotFoundByName` if the function does not exist.
    ///
    /// # Example
    /// ```
    /// use gbf_rs::module::ModuleBuilder;
    ///
    /// let mut module = ModuleBuilder::new().name("test.gs2").build().unwrap();
    /// let function_id = module.create_function("test_function", 123).unwrap();
    /// let function = module.get_function_by_name_mut("test_function").unwrap();
    /// ```
    pub fn get_function_by_name_mut<N: Into<String>>(
        &mut self,
        name: N,
    ) -> Result<&mut Function, ModuleError> {
        let name = name.into();
        let id = self.get_function_id_by_name(name)?;
        self.get_function_by_id_mut(&id)
    }

    /// Get function id by name
    ///
    /// # Arguments
    /// - `name`: The name of the function to retrieve.
    ///
    /// # Returns
    /// - The `FunctionId` of the function, if it exists.
    ///
    /// # Errors
    /// - `ModuleError::FunctionNotFoundByName` if the function does not exist.
    ///
    /// # Example
    /// ```
    /// use gbf_rs::module::ModuleBuilder;
    ///
    /// let mut module = ModuleBuilder::new().name("test.gs2").build().unwrap();
    /// let function_id = module.create_function("test_function", 123).unwrap();
    /// let function_id = module.get_function_id_by_name("test_function").unwrap();
    /// ```
    pub fn get_function_id_by_name<N: Into<String>>(
        &self,
        name: N,
    ) -> Result<FunctionId, ModuleError> {
        let name = name.into();
        self.name_to_id
            .get(&Some(name.clone()))
            .cloned()
            .ok_or(ModuleError::FunctionNotFoundByName(name))
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
    /// let mut module = ModuleBuilder::new().name("test.gs2").build().unwrap();
    /// let function_id = module.create_function("test_function", 123).unwrap();
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
    /// let module = ModuleBuilder::new().name("test.gs2").build().unwrap();
    /// assert!(!module.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        // The module will always have an entry function, so this is always false
        self.functions.is_empty()
    }
}

/// Internal API for `Module`.
impl Module {
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
            // Check if instruction is even reachable. If it's not, we can skip it
            if !loaded_bytecode.is_instruction_reachable(offset) {
                continue;
            }
            let function_name = loaded_bytecode.get_function_name_for_address(offset);
            // The precondition above guarantees that this will always be true
            debug_assert!(function_name.is_ok());
            let function_name = function_name.unwrap().clone();

            if let Some(function_name) = function_name.clone() {
                if !self.has_function(function_name.clone()) {
                    let function_name_clone = function_name.clone();
                    let offset = loaded_bytecode
                        .function_map
                        .get(&Some(function_name_clone))
                        .expect("Function must exist in the function map");
                    self.create_function(function_name, *offset)?;
                }
            }

            let is_entry = function_name.is_none();

            // Get the function reference. If the function is an entry function, we will use the entry function, otherwise we will use the function name
            let function = if is_entry {
                self.get_entry_function_mut()
            } else {
                self.get_function_by_name_mut(function_name.unwrap())?
            };

            // Get the start address for the basic block
            let start_address = loaded_bytecode.find_block_start_address(offset);

            // Create new basic block if it doesn't exist
            if !function.basic_block_exists_by_address(start_address) {
                // We won't run into this error because we are not making an entry block here
                function
                    .create_block(BasicBlockType::Normal, start_address)
                    .expect("Block collisions are not possible");
            }

            // Get the basic block reference
            let block = function
                .get_basic_block_by_start_address_mut(start_address)
                .unwrap();

            // Add the instruction to the basic block
            block.add_instruction(instruction.clone());
        }

        // To the entry block, let's create a new basic block with address set to the length of the bytecode
        // This is the block that will be used to represent the end of the module
        let entry = self.get_entry_function_mut();
        entry
            .create_block(
                BasicBlockType::ModuleEnd,
                loaded_bytecode.instructions.len() as Gs2BytecodeAddress,
            )
            .unwrap();

        // Iterate through each function that was created. For each function, we will iterate through
        // each basic block and find the terminator instruction. Based on the terminator opcode,
        // we will connect edges in the graph.
        for function in self.functions.iter_mut() {
            let block_data: Vec<_> = function
                .iter()
                .map(|block| (block.id, block.last_instruction().cloned()))
                .collect();

            for (id, terminator) in block_data {
                Self::process_block_edges(function, id, terminator);
            }
        }
        Ok(())
    }

    fn process_block_edges(
        function: &mut Function,
        id: BasicBlockId,
        terminator: Option<Instruction>,
    ) {
        if let Some(terminator) = terminator {
            let terminator_opcode = terminator.opcode;
            let terminator_operand = terminator.operand;
            let terminator_address = terminator.address;
            if terminator_opcode.has_jump_target() {
                if let Some(branch_address) =
                    terminator_operand.and_then(|o| o.get_number_value().ok())
                {
                    let branch_block_id = function
                        .get_basic_block_id_by_start_address(branch_address as Gs2BytecodeAddress)
                        .expect("Block must exist");
                    function.add_edge(id, branch_block_id).unwrap();
                }
            }

            // If appropriate, connect the next block
            if terminator_opcode.connects_to_next_block() {
                let next_block_id = function
                    .get_basic_block_id_by_start_address(terminator_address + 1)
                    .expect("Block must exist");
                function.add_edge(id, next_block_id).unwrap();
            }
        }
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
    /// - `ModuleError::FunctionNotFoundById` if the function does not exist.
    fn get_function_by_id(&self, id: &FunctionId) -> Result<&Function, ModuleError> {
        let index = self
            .id_to_index
            .get(id)
            .ok_or(ModuleError::FunctionNotFoundById(id.clone()))?;

        // Provides fast sequential access, but panics if the index is out of bounds
        Ok(&self.functions[*index])
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
    /// - `ModuleError::FunctionNotFoundById` if the function does not exist.
    fn get_function_by_id_mut(&mut self, id: &FunctionId) -> Result<&mut Function, ModuleError> {
        let index = self
            .id_to_index
            .get(id)
            .ok_or(ModuleError::FunctionNotFoundById(id.clone()))?;

        // Provides fast sequential access, but panics if the index is out of bounds
        Ok(&mut self.functions[*index])
    }
}

// === Implementations ===

/// Display implementation for `Module`.
impl Display for Module {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.name.as_deref().unwrap_or("Unnamed Module"))
    }
}

/// Default implementation for `ModuleBuilder`.
impl Default for ModuleBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Deref implementation for `Module`.
impl std::ops::Deref for Module {
    type Target = Vec<Function>;

    fn deref(&self) -> &Self::Target {
        &self.functions
    }
}

/// Index implementation for `Module`.
impl std::ops::Index<usize> for Module {
    type Output = Function;

    fn index(&self, index: usize) -> &Self::Output {
        &self.functions[index]
    }
}

/// Immutable IntoIterator implementation for `Module`.
impl<'a> IntoIterator for &'a Module {
    type Item = &'a Function;
    type IntoIter = std::slice::Iter<'a, Function>;

    fn into_iter(self) -> Self::IntoIter {
        self.functions.iter()
    }
}

/// Mutable IntoIterator implementation for `Module`.
impl<'a> IntoIterator for &'a mut Module {
    type Item = &'a mut Function;
    type IntoIter = std::slice::IterMut<'a, Function>;

    fn into_iter(self) -> Self::IntoIter {
        self.functions.iter_mut()
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
