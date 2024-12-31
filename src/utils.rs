#![deny(missing_docs)]

/// A type representing a bytecode address.
pub type Gs2BytecodeAddress = usize;

/// At what length text should be truncated for operands.
pub const OPERAND_TRUNCATE_LENGTH: usize = 100;

/// A constant representing the current version of the software, in semver format.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// A constant representing the name of the software.
pub const NAME: &str = env!("CARGO_PKG_NAME");
