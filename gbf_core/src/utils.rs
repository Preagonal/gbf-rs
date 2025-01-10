#![deny(missing_docs)]

use std::ascii::escape_default;

/// A type representing a bytecode address.
pub type Gs2BytecodeAddress = usize;

/// At what length text should be truncated for operands.
pub const OPERAND_TRUNCATE_LENGTH: usize = 100;

/// A constant representing the current version of the software, in semver format.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// A constant representing the name of the software.
pub const NAME: &str = env!("CARGO_PKG_NAME");

/// A constant representing GBF green
pub const GBF_GREEN: &str = "#98ff64";

/// A constant representing GBF red
pub const GBF_RED: &str = "#ff6464";

/// A constant representing GBF blue
pub const GBF_BLUE: &str = "#64b2ff";

/// A constant representing GBF yellow
pub const GBF_YELLOW: &str = "#ffd964";

/// A constant representing GBF light gray
pub const GBF_LIGHT_GRAY: &str = "#cdcdcd";

/// A constant representing GBF dark gray
pub const GBF_DARK_GRAY: &str = "#1e1e1e";

/// Escapes a string using `std::ascii::escape_default`.
///
/// # Arguments
/// * `input` - A value that can be converted into a `String`.
///
/// # Returns
/// A new `String` where each character is escaped according to `escape_default`.
pub fn escape_string<S>(input: S) -> String
where
    S: Into<String>,
{
    input
        .into()
        .bytes()
        .flat_map(escape_default)
        .map(char::from)
        .collect()
}
