#![deny(missing_docs)]

use std::{collections::HashMap, sync::OnceLock};

use bin_op::BinaryOperationHandler;
use identifier::IdentifierHandler;
use literal::LiteralHandler;
use nop::NopHandler;
use variable_operand::VariableOperandHandler;

use crate::{instruction::Instruction, opcode::Opcode};

use super::{
    function_decompiler::FunctionDecompilerError,
    function_decompiler_context::FunctionDecompilerContext,
};

/// Handles binary operation instructions.
pub mod bin_op;
/// Handles identifier instructions.
pub mod identifier;
/// Handles literal instructions.
pub mod literal;
/// Handles instructions that are not useful to our decompiler.
pub mod nop;
/// Handles member access instructions.
pub mod special_two_operand;
/// Handles cases with a variable number of operands.
pub mod variable_operand;

/// Represents an opcode handler for the decompiler.
pub trait OpcodeHandler: Send + Sync {
    /// Handles the given instruction.
    ///
    /// # Arguments
    /// - `context`: The decompiler context.
    /// - `instruction`: The instruction to handle.
    ///
    /// # Errors
    /// - Returns a `FunctionDecompilerError` if there is an issue handling the instruction.
    fn handle_instruction(
        &self,
        context: &mut FunctionDecompilerContext,
        instruction: &Instruction,
    ) -> Result<(), FunctionDecompilerError>;
}

static GLOBAL_OPCODE_HANDLERS: OnceLock<HashMap<Opcode, Box<dyn OpcodeHandler>>> = OnceLock::new();

/// Gets the global opcode handlers.
pub fn global_opcode_handlers() -> &'static HashMap<Opcode, Box<dyn OpcodeHandler>> {
    GLOBAL_OPCODE_HANDLERS.get_or_init(|| {
        let mut handlers: HashMap<Opcode, Box<dyn OpcodeHandler>> = HashMap::new();

        // These handlers are used to create identifier nodes. All of them, with the
        // exception of `PushVariable`, use the lowercase opcode name as the identifier
        // name.
        handlers.insert(Opcode::Player, Box::new(IdentifierHandler));
        handlers.insert(Opcode::PlayerO, Box::new(IdentifierHandler));
        handlers.insert(Opcode::Temp, Box::new(IdentifierHandler));
        handlers.insert(Opcode::Level, Box::new(IdentifierHandler));
        handlers.insert(Opcode::This, Box::new(IdentifierHandler));
        handlers.insert(Opcode::ThisO, Box::new(IdentifierHandler));
        handlers.insert(Opcode::Params, Box::new(IdentifierHandler));
        handlers.insert(Opcode::PushVariable, Box::new(IdentifierHandler));

        // These handlers are used to create literal nodes.
        handlers.insert(Opcode::PushString, Box::new(LiteralHandler));
        handlers.insert(Opcode::PushNumber, Box::new(LiteralHandler));

        // These handlers are used to create binary operation nodes.
        handlers.insert(Opcode::Add, Box::new(BinaryOperationHandler));

        // These opcodes do nothing ATM
        handlers.insert(Opcode::ConvertToFloat, Box::new(NopHandler));

        // Special cases
        handlers.insert(
            Opcode::AccessMember,
            Box::new(special_two_operand::SpecialTwoOperandHandler),
        );
        handlers.insert(
            Opcode::Assign,
            Box::new(special_two_operand::SpecialTwoOperandHandler),
        );

        // Variable operand handlers
        handlers.insert(Opcode::Call, Box::new(VariableOperandHandler));

        handlers
    })
}
