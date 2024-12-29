use std::{fs::File, io::Read, path::Path};

/// Load bytecode file from gs2bc directory and return the reader.
///
/// # Arguments
/// - `name`: The name of the file to load.
///
/// # Returns
/// - A `Result` containing the reader if the file was found, or an error if it was not.
fn load_bytecode(name: &str) -> Result<impl Read, std::io::Error> {
    let path = Path::new("tests").join("gs2bc").join(name);
    let file = File::open(path)?;
    Ok(file)
}

#[test]
fn load_simple_gs2() {
    let reader = load_bytecode("simple.gs2bc").unwrap();
    let mut bytecode_loader = gbf_rs::bytecode_loader::BytecodeLoader::new(reader);
    bytecode_loader.load().unwrap();

    // Check the number of functions. In this case, it should be zero since
    // GS2 does not officially consider our case of an "nameless" function.
    let functions = bytecode_loader.function_map;
    assert_eq!(functions.len(), 0);

    // Check the number of instructions. In this case, it should be 31.
    let instructions = bytecode_loader.instructions;
    assert_eq!(instructions.len(), 32);

    // Check the first instruction. In this case, it should be `Player`
    let first_instruction = instructions.first().unwrap();
    assert_eq!(first_instruction.opcode, gbf_rs::opcode::Opcode::Player);

    // Check the last instruction. In this case, it should be `Pop`
    let last_instruction = instructions.last().unwrap();
    assert_eq!(last_instruction.opcode, gbf_rs::opcode::Opcode::Pop);

    // The instruction at the third index should be PushString with the string "Hello, World!"
    let third_instruction = instructions.get(3).unwrap();
    assert_eq!(third_instruction.opcode, gbf_rs::opcode::Opcode::PushString);
    assert_eq!(
        third_instruction.operand,
        Some(gbf_rs::operand::Operand::new_string("Hello, World!"))
    );

    // The instruction at the 13th index should be PushNumber with the number 0
    let thirteenth_instruction = instructions.get(13).unwrap();
    assert_eq!(
        thirteenth_instruction.opcode,
        gbf_rs::opcode::Opcode::PushNumber
    );
    assert_eq!(
        thirteenth_instruction.operand,
        Some(gbf_rs::operand::Operand::new_number(0))
    );
}

#[test]
fn load_multiple_functions() {
    let reader = load_bytecode("multiple-functions.gs2bc").unwrap();
    let mut bytecode_loader = gbf_rs::bytecode_loader::BytecodeLoader::new(reader);
    bytecode_loader.load().unwrap();

    // Check the number of functions. In this case, it should be 5
    let functions = bytecode_loader.function_map;
    assert_eq!(functions.len(), 5);

    // Check the number of instructions. In this case, it should be 79
    let instructions = bytecode_loader.instructions;
    assert_eq!(instructions.len(), 79);

    // Check the first instruction. In this case, it should be `Jmp` with the address 79
    let first_instruction = instructions.first().unwrap();
    assert_eq!(first_instruction.opcode, gbf_rs::opcode::Opcode::Jmp);
    assert_eq!(
        first_instruction.operand,
        Some(gbf_rs::operand::Operand::new_number(79))
    );

    // Check the last instruction. In this case, it should be `Ret`
    let last_instruction = instructions.last().unwrap();
    assert_eq!(last_instruction.opcode, gbf_rs::opcode::Opcode::Ret);
}
