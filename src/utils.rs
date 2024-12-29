#![deny(missing_docs)]

/// A type representing a bytecode address.
pub type Gs2BytecodeAddress = usize;

/// A constant representing the current version of the software, in semver format.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
