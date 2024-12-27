use std::collections::HashMap;
use thiserror::Error;

use crate::function::{Function, FunctionId};

#[derive(Error, Debug)]
pub enum ModuleError {
    #[error("Function not found: {0}")]
    FunctionNotFound(FunctionId)
}

pub enum SectionType {
    Gs1Flags,
    Functions,
    Strings,
    Instructions,
}

pub struct Module {
    pub name: String,
    strings: Vec<String>,
    functions: HashMap<FunctionId, Function>
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
            strings: Vec::new(),
            functions: HashMap::new()
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
        self.functions.get(id).ok_or(ModuleError::FunctionNotFound(id.clone()))
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
}
