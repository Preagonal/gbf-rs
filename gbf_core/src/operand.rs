#![deny(missing_docs)]

use core::fmt;
use serde::{Deserialize, Serialize};

use thiserror::Error;

use crate::utils::Gs2BytecodeAddress;

/// Represents an error that can occur when parsing an operand.
#[derive(Debug, Error)]
pub enum OperandError {
    /// Invalid conversion
    #[error("Attempted to convert {0} operand to a {1}")]
    InvalidConversion(String, String),

    /// Invalid jump target
    #[error("Invalid jump target: {0}")]
    InvalidJumpTarget(Gs2BytecodeAddress),
}

/// Represents an operand, which can be one of several types.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum Operand {
    /// A string operand.
    String(String),

    /// A floating-point operand (stored as a string).
    Float(String),

    /// An integer operand.
    Number(i32),
}

impl Operand {
    /// Creates a new string operand.
    ///
    /// # Arguments
    /// - `value`: The value of the string operand.
    ///
    /// # Returns
    /// - A new `Operand::String`.
    ///
    /// # Examples
    /// ```
    /// use gbf_core::operand::Operand;
    ///
    /// let operand = Operand::new_string("Hello, world!");
    /// ```
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
    ///
    /// # Examples
    /// ```
    /// use gbf_core::operand::Operand;
    ///
    /// let operand = Operand::new_float("3.14");
    /// ```
    pub fn new_float(value: impl Into<String>) -> Self {
        Operand::Float(value.into())
    }

    /// Creates a new number operand.
    ///
    /// # Arguments
    /// - `value`: The value of the number operand.
    ///
    /// # Returns
    /// - A new `Operand::Number`.
    ///
    /// # Examples
    /// ```
    /// use gbf_core::operand::Operand;
    ///
    /// let operand = Operand::new_number(42);
    /// ```
    pub fn new_number(value: i32) -> Self {
        Operand::Number(value)
    }

    /// Retrieves the value of the operand as a string reference, if applicable.
    ///
    /// # Returns
    /// - The value of the operand as a string reference.
    ///
    /// # Errors
    /// - `OperandError::InvalidConversion` if the operand is a number.
    ///
    /// # Examples
    /// ```
    /// use gbf_core::operand::Operand;
    ///
    /// let operand = Operand::new_string("Hello, world!");
    /// let value = operand.get_string_value().unwrap();
    /// assert_eq!(value, "Hello, world!");
    /// ```
    pub fn get_string_value(&self) -> Result<&str, OperandError> {
        match self {
            Operand::String(value) | Operand::Float(value) => Ok(value),
            Operand::Number(_) => Err(OperandError::InvalidConversion(
                "Number".to_string(),
                "String".to_string(),
            )),
        }
    }

    /// Retrieves the value of the operand as a number, if applicable.
    ///
    /// # Returns
    /// - The value of the operand as a number.
    ///
    /// # Errors
    /// - `OperandError::InvalidConversion` if the operand is a string.
    ///
    /// # Examples
    /// ```
    /// use gbf_core::operand::Operand;
    ///
    /// let operand = Operand::new_number(42);
    /// let value = operand.get_number_value().unwrap();
    /// assert_eq!(value, 42);
    /// ```
    pub fn get_number_value(&self) -> Result<i32, OperandError> {
        match self {
            Operand::Number(value) => Ok(*value),
            Operand::String(_) | Operand::Float(_) => Err(OperandError::InvalidConversion(
                "String/Float".to_string(),
                "Number".to_string(),
            )),
        }
    }
}

impl fmt::Display for Operand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Operand::String(value) => value.clone(),
            Operand::Float(value) => value.clone(),
            Operand::Number(value) => format!("{:#x}", value),
        }
        .fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn string_operand() {
        let operand = Operand::new_string("Hello, world!");
        assert_eq!(operand.get_string_value().unwrap(), "Hello, world!");
        assert_eq!(operand.to_string(), "Hello, world!");
    }

    #[test]
    fn test_illegal_conversion() {
        // Create number -> string
        let operand = Operand::new_number(42);
        assert!(operand.get_string_value().is_err());

        // Create string -> number
        let operand = Operand::new_string("Hello, world!");
        assert!(operand.get_number_value().is_err());

        // double -> number
        let operand = Operand::new_float("3.14");
        assert!(operand.get_number_value().is_err());
    }

    #[test]
    fn float_operand() {
        let operand = Operand::new_float("3.14");
        assert_eq!(operand.get_string_value().unwrap(), "3.14");
        assert_eq!(operand.to_string(), "3.14");
    }

    #[test]
    fn int_operand() {
        let operand = Operand::new_number(42);
        assert_eq!(operand.get_number_value().unwrap(), 42);
        assert_eq!(operand.to_string(), "0x2a");
    }

    #[test]
    fn display_trait() {
        let operand = Operand::new_number(123);
        assert_eq!(operand.to_string(), "0x7b");
        assert_eq!(format!("{}", operand), "0x7b");
    }
}
