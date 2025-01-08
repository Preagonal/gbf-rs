#![deny(missing_docs)]

/// A type representing a bytecode address.
pub type Gs2BytecodeAddress = usize;

/// At what length text should be truncated for operands.
pub const OPERAND_TRUNCATE_LENGTH: usize = 100;

/// A constant representing the current version of the software, in semver format.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// A constant representing the name of the software.
pub const NAME: &str = env!("CARGO_PKG_NAME");

/// A constant representing GBF green
pub const GBF_GREEN: &str = "#99ff66";

/// A constant representing GBF red
pub const GBF_RED: &str = "#ff6666";

/// A constant representing GBF blue
pub const GBF_BLUE: &str = "#66b3ff";

/// A constant representing GBF yellow
pub const GBF_YELLOW: &str = "#ffd966";

/// A constant representing GBF light gray
pub const GBF_DARK_GRAY: &str = "#666666";

/// A constant representing GBF dark gray
pub const GBF_LIGHT_GRAY: &str = "#1a1a1a";
