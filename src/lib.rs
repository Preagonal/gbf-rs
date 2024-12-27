#![feature(rustdoc_missing_doc_code_examples)]
#![deny(missing_docs)]
#![deny(rustdoc::missing_doc_code_examples)]

//! This crate provides basic block definitions, function definitions, module definitions,
//! graph definitions, instruction definitions, opcode definitions, and operand definitions.

/// This module contains basic block definitions and operations.
pub mod basic_block;
/// This module contains the definition of a function.
pub mod function;
/// This module contains the definition of a module.
pub mod graal_io;
/// This module contains the definition of a module.
pub mod graph;
/// This module contains the definition of an instruction.
pub mod instruction;
/// This module contains the definition of a module.
pub mod module;
/// This module contains the definition of an operand.
pub mod opcode;
/// This module contains the definition of an operand.
pub mod operand;
