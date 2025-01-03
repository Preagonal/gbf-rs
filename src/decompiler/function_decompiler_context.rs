#![deny(missing_docs)]

/// TODO: Map instructions to a reference value (for usage with loop variables, etc.)
/// TODO: We should call loop variables instruction references (InstrRef)
/// TODO: We should have an AST pass that identifies variables with identifiers that are
/// TODO: the same, and wrap them in an InstrRef (for MemberAccess & Identifier) since
/// TODO: this will help further analysis
pub struct FunctionDecompilerContext {}
