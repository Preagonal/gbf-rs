#![deny(missing_docs)]
#![deny(rustdoc::missing_doc_code_examples)]

use core::fmt;
use serde::{Deserialize, Serialize};

/// Represents an operand, which can be one of several types.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum Operand {
    /// A string operand.
    String(String),

    /// A floating-point operand (stored as a string).
    Float(String),

    /// An integer operand.
    Int(i32),
}

impl Operand {
    /// Converts the operand to a string.
    ///
    /// # Returns
    /// - The string representation of the operand.
    pub fn to_string(&self) -> String {
        match self {
            Operand::String(value) => value.clone(),
            Operand::Float(value) => value.clone(),
            Operand::Int(value) => value.to_string(),
        }
    }

    /// Creates a new string operand.
    ///
    /// # Arguments
    /// - `value`: The value of the string operand.
    ///
    /// # Returns
    /// - A new `Operand::String`.
    pub fn new_string(value: impl Into<String>) -> Self {
        Operand::String(value.into())
    }

    /// Creates a new float operand.
    ///
    /// # Arguments
    /// - `value`: The value of the float operand.
    ///
    /// # Returns
    /// - A new `Operand::Float`.
    pub fn new_float(value: impl Into<String>) -> Self {
        Operand::Float(value.into())
    }

    /// Creates a new integer operand.
    ///
    /// # Arguments
    /// - `value`: The value of the integer operand.
    ///
    /// # Returns
    /// - A new `Operand::Int`.
    pub fn new_int(value: i32) -> Self {
        Operand::Int(value)
    }

    /// Retrieves the value of the operand as a string reference, if applicable.
    ///
    /// # Returns
    /// - `Some(&str)` for `Operand::String` and `Operand::Float`.
    /// - `None` for `Operand::Int`.
    pub fn get_string_value(&self) -> Option<&str> {
        match self {
            Operand::String(value) | Operand::Float(value) => Some(value),
            Operand::Int(_) => None,
        }
    }

    /// Retrieves the value of the operand as an integer, if applicable.
    ///
    /// # Returns
    /// - `Some(i32)` for `Operand::Int`.
    /// - `None` for `Operand::String` and `Operand::Float`.
    pub fn get_int_value(&self) -> Option<i32> {
        if let Operand::Int(value) = self {
            Some(*value)
        } else {
            None
        }
    }
}

impl fmt::Display for Operand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn string_operand() {
        let operand = Operand::new_string("Hello, world!");
        assert_eq!(operand.get_string_value(), Some("Hello, world!"));
        assert_eq!(operand.to_string(), "Hello, world!");
    }

    #[test]
    fn float_operand() {
        let operand = Operand::new_float("3.14");
        assert_eq!(operand.get_string_value(), Some("3.14"));
        assert_eq!(operand.to_string(), "3.14");
    }

    #[test]
    fn int_operand() {
        let operand = Operand::new_int(42);
        assert_eq!(operand.get_int_value(), Some(42));
        assert_eq!(operand.to_string(), "42");
    }

    #[test]
    fn display_trait() {
        let operand = Operand::new_int(123);
        assert_eq!(operand.to_string(), "123");
        assert_eq!(format!("{}", operand), "123");
    }
}
