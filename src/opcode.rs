use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Error type for invalid opcodes.
#[derive(Error, Debug)]
pub enum OpcodeError {
    #[error("Invalid opcode: {0}")]
    InvalidOpcode(u8),

    #[error("Invalid opcode: {0}")]
    InvalidOpcodeString(String),
}

/// A macro to define opcodes as a `#[repr(u8)]` enum.
/// 
/// # Overview
/// This macro simplifies defining opcodes by automatically generating:
/// - An `Opcode` enum with associated `u8` values.
/// - Utility methods like `from_byte` for safe conversion and `to_byte` for reverse mapping.
macro_rules! define_opcodes {
    (
        $( $name:ident = $value:expr ),* $(,)?
    ) => {
        /// Enum representing opcodes for a bytecode system.
        ///
        /// Each variant corresponds to an opcode, with its numeric value
        /// defined as a `u8`.
        #[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize, Hash)]
        #[repr(u8)]
        pub enum Opcode {
            $(
                $name = $value,
            )*
        }

        impl Opcode {
            /// Converts a `u8` into an `Opcode`, if possible.
            ///
            /// # Arguments
            /// - `byte`: The `u8` value to convert.
            ///
            /// # Returns
            /// - `Some(Opcode)` if the value corresponds to a valid opcode.
            /// - `None` if the value does not match any defined opcode.
            pub fn from_byte(byte: u8) -> Result<Self, OpcodeError> {
                match byte {
                    $(
                        $value => Ok(Opcode::$name),
                    )*
                    _ => Err(OpcodeError::InvalidOpcode(byte)),
                }
            }

            /// Converts an `Opcode` to its `u8` value.
            ///
            /// # Returns
            /// - The numeric value (`u8`) of the opcode.
            pub fn to_byte(self) -> u8 {
                self as u8
            }

            /// Convert an `Opcode` to a human-readable string.
            /// 
            /// # Returns
            /// - A string representation of the opcode.
            pub fn to_string(self) -> &'static str {
                match self {
                    $(
                        Opcode::$name => stringify!($name),
                    )*
                }
            }

            /// Convert a string to an `Opcode`
            /// 
            /// # Arguments
            /// - `name`: The string to convert.
            pub fn from_str(name: &str) -> Result<Self, OpcodeError> {
                match name {
                    $(
                        stringify!($name) => Ok(Opcode::$name),
                    )*
                    _ => Err(OpcodeError::InvalidOpcodeString(name.to_string())),
                }
            }
        }
    };
}

// Using the macro to define the opcodes
define_opcodes! {
    Jmp = 0x1,
    Jeq = 0x2,
    ShortCircuitOr = 0x3,
    Jne = 0x4,
    ShortCircuitAnd = 0x5,
    Call = 0x6,
    Ret = 0x7,
    Sleep = 0x8,
    IncreaseLoopCounter = 0x9,
    FunctionStart = 0xa,
    WaitFor = 0xb,
    // Unknown values: 0xc - 0x13
    PushNumber = 0x14,
    PushString = 0x15,
    PushVariable = 0x16,
    PushArray = 0x17,
    PushTrue = 0x18,
    PushFalse = 0x19,
    PushNull = 0x1a,
    PushPi = 0x1b,
    // Unknown values: 0x1c  - 0x1d
    Copy = 0x1e,
    Swap = 0x1f,
    Pop = 0x20,
    ConvertToFloat = 0x21,
    ConvertToString = 0x22,
    AccessMember = 0x23,
    ConvertToObject = 0x24,
    EndArray = 0x25,
    NewArray = 0x26,
    SetArray = 0x27,
    New = 0x28,
    MakeVar = 0x29,
    NewObject = 0x2a,
    ConvertToVariable = 0x2b,
    ShortCircuitEnd = 0x2c,
    SetLoopVariable = 0x2d,
    GetLoopVariable = 0x2e,
    MarkLoopVariable = 0x2f,
    // Unknown values: 0x30 - 0x31
    Assign = 0x32,
    EndParams = 0x33,
    Inc = 0x34,
    Dec = 0x35,
    // Unknown values: 0x36 - 0x3b
    Add = 0x3c,
    Subtract = 0x3d,
    Multiply = 0x3e,
    Divide = 0x3f,
    Modulo = 0x40,
    Power = 0x41,
    // Unknown values: 0x42 - 0x43
    Negate = 0x44,
    UnarySubtract = 0x45,
    Equal = 0x46,
    NotEqual = 0x47,
    LessThan = 0x48,
    GreaterThan = 0x49,
    LessThanOrEqual = 0x4a,
    GreaterThanOrEqual = 0x4b,
    BitwiseOr = 0x4c,
    BitwiseAnd = 0x4d,
    BitwiseXor = 0x4e,
    BitwiseInvert = 0x4f,
    InRange = 0x50,
    In = 0x51,
    ObjIndex = 0x52,
    ObjType = 0x53,
    Format = 0x54,
    Int = 0x55,
    Abs = 0x56,
    Random = 0x57,
    Sin = 0x58,
    Cos = 0x59,
    ArcTan = 0x5a,
    Exp = 0x5b,
    Log = 0x5c,
    Min = 0x5d,
    Max = 0x5e,
    GetAngle = 0x5f,
    GetDir = 0x60,
    VecX = 0x61,
    VecY = 0x62,
    ObjIndices = 0x63,
    ObjLink = 0x64,
    ShiftLeft = 0x65,
    ShiftRight = 0x66,
    Char = 0x67,
    // Unknown values: 0x68 - 0x6d
    ObjTrim = 0x6e,
    ObjLength = 0x6f,
    ObjPos = 0x70,
    Join = 0x71,
    ObjCharAt = 0x72,
    ObjSubstring = 0x73,
    ObjStarts = 0x74,
    ObjEnds = 0x75,
    ObjTokenize = 0x76,
    GetTranslation = 0x77,
    // Unknown values: 0x78 - 0x81
    ObjSize = 0x82,
    AssignArrayIndex = 0x83,
    AssignArray = 0x84,
    AssignMultiDimensionalArrayIndex = 0x85,
    AssignMultiDimensionalArray = 0x86,
    // Unknown values: 0x87
    ObjAddString = 0x88,
    ObjDeleteString = 0x89,
    ObjRemoveString = 0x8a,
    ObjReplaceString = 0x8b,
    ObjInsertString = 0x8c,
    ObjClear = 0x8d,
    MultiDimenArray = 0x8e,
    // Unknown values: 0x8f - 0x95
    With = 0x96,
    WithEnd = 0x97,
    // Unknown values: 0x98 - 0xa2
    ForEach = 0xa3,
    // Unknown values: 0xa4 - 0xb3
    This = 0xb4,
    ThisO = 0xb5,
    Player = 0xb6,
    PlayerO = 0xb7,
    Level = 0xb8,
    // Unknown values: 0xb9 - 0xbc
    Temp = 0xbd,
    Params = 0xbe,
    // Unknown values: 0xbf - 0xef
    ImmStringByte = 0xf0,
    ImmStringShort = 0xf1,
    ImmStringInt = 0xf2,
    ImmByte = 0xf3,
    ImmShort = 0xf4,
    ImmInt = 0xf5,
    ImmFloat = 0xf6,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_conversion() {
        assert_eq!(Opcode::from_byte(0x1).unwrap(), Opcode::Jmp);
        assert_eq!(Opcode::from_byte(0x21).unwrap(), Opcode::ConvertToFloat);
    }

    #[test]
    fn test_invalid_conversion() {
        assert!(Opcode::from_byte(0xFF).is_err());
        assert!(Opcode::from_byte(0x68).is_err());
    }

    #[test]
    fn test_to_byte() {
        assert_eq!(Opcode::Jmp.to_byte(), 0x1);
        assert_eq!(Opcode::ConvertToString.to_byte(), 0x22);
    }

    #[test]
    fn test_to_string() {
        assert_eq!(Opcode::Jmp.to_string(), "Jmp");
        assert_eq!(Opcode::ConvertToString.to_string(), "ConvertToString");
    }

    #[test]
    fn test_from_str() {
        assert_eq!(Opcode::from_str("Jmp").unwrap(), Opcode::Jmp);
        assert_eq!(Opcode::from_str("ConvertToString").unwrap(), Opcode::ConvertToString);
        assert!(Opcode::from_str("Invalid").is_err());
    }
}
