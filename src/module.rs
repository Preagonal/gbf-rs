#![deny(missing_docs)]

use std::collections::HashMap;
use thiserror::Error;

use crate::{
    bytecode_loader::{self, BytecodeLoaderError},
    function::{Function, FunctionId},
};

/// Error type for module operations.
#[derive(Error, Debug)]
pub enum ModuleError {
    /// Error for when a function is not found in the module.
    #[error("Function not found: {0}")]
    FunctionNotFound(FunctionId),
}

/// Represents a GS2 module in a bytecode system. A module contains
/// functions, strings, and other data.
pub struct Module {
    /// The name of the module.
    pub name: String,
    functions: HashMap<FunctionId, Function>,
}

impl Module {
    /// Create a new module.
    ///
    /// # Arguments
    /// - `name`: The name of the module.
    ///
    /// # Returns
    /// - A new `Module` instance.
    ///
    /// # Example
    /// ```
    /// use gbf_rs::module::Module;
    ///
    /// let module = Module::new("test".to_string());
    /// ```
    pub fn new(name: String) -> Self {
        Self {
            name,
            functions: HashMap::new(),
        }
    }

    /// Create a new function in the module.
    ///
    /// # Returns
    /// - The `FunctionId` of the new function.
    ///
    /// # Example
    /// ```
    /// use gbf_rs::module::Module;
    ///
    /// let mut module = Module::new("test".to_string());
    /// let function_id = module.create_function();
    /// ```
    pub fn create_function(&mut self) -> FunctionId {
        let id = FunctionId::new(self.functions.len(), None, 0);
        self.functions.insert(id.clone(), Function::new(id.clone()));
        id
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
    /// use gbf_rs::module::Module;
    ///
    /// let mut module = Module::new("test".to_string());
    /// let function_id = module.create_function();
    /// let function = module.get_function(&function_id).unwrap();
    /// ```
    pub fn get_function(&self, id: &FunctionId) -> Result<&Function, ModuleError> {
        self.functions
            .get(id)
            .ok_or(ModuleError::FunctionNotFound(id.clone()))
    }

    /// Load bytecode into the module using a reader.
    ///
    /// # Arguments
    /// - `reader`: The reader to use to load the bytecode.
    ///
    /// # Errors
    /// - `BytecodeLoaderError`: An error occurred while loading the bytecode.
    ///
    /// # Example
    /// ```
    /// use gbf_rs::module::Module;
    ///
    /// let mut module = Module::new("test.gs2".to_string());
    /// let bytecode = vec![
    ///     0x00, 0x00, 0x00, 0x01,
    ///     0x00, 0x00, 0x00, 0x04,
    ///     0x00, 0x00, 0x00, 0x00,
    ///     0x00, 0x00, 0x00, 0x02,
    ///     0x00, 0x00, 0x00, 0x00,
    ///     0x00, 0x00, 0x00, 0x03,
    ///     0x00, 0x00, 0x00, 0x00,
    ///     0x00, 0x00, 0x00, 0x04,
    ///     0x00, 0x00, 0x00, 0x00
    /// ];
    /// module.load_bytecode(&bytecode[..]).unwrap();
    /// ```
    pub fn load_bytecode<R: std::io::Read>(
        &mut self,
        reader: R,
    ) -> Result<(), BytecodeLoaderError> {
        let _loader = bytecode_loader::BytecodeLoaderBuilder::new(reader).build()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mut module = Module::new("test".to_string());
        let function_id = module.create_function();
        let function = module.get_function(&function_id).unwrap();
        assert_eq!(function.id, function_id);
    }

    #[test]
    fn load_bytecode() {
        let mut module = Module::new("test.gs2".to_string());
        let bytecode = [
            0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00,
        ];
        module.load_bytecode(&bytecode[..]).unwrap();

        // test failure case
        let bytecode = [0x00, 0x00, 0x00, 0x01];
        assert!(module.load_bytecode(&bytecode[..]).is_err());
    }
}
