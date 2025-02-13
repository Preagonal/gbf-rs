use common::load_bytecode;
pub mod common;

use gbf_core::{
    cfg_dot::{CfgDotConfig, DotRenderableGraph},
    instruction::Instruction,
};

#[test]
fn load_simple_gs2() {
    let reader = load_bytecode("simple.gs2bc").unwrap();
    let module = gbf_core::module::ModuleBuilder::new()
        .name("simple.gs2".to_string())
        .reader(Box::new(reader))
        .build()
        .unwrap();

    // Check the number of functions. In this case, it should be one since
    // we count the entry point function.
    let functions = module.len();
    assert_eq!(functions, 1);

    // Find the number of Basic Blocks in the function. In this case, it should be 1
    let function = module.get_entry_function();
    let basic_blocks = function.len();

    // We have a ModuleEnd block at the end of the module
    assert_eq!(basic_blocks, 2);
    let basic_block = &function[0];
    let instructions = basic_block.len();
    assert_eq!(instructions, 32);

    // Re-initialize the reader & module, but this time, try out the iterators.
    let reader = load_bytecode("simple.gs2bc").unwrap();
    let module = gbf_core::module::ModuleBuilder::new()
        .name("simple.gs2".to_string())
        .reader(Box::new(reader))
        .build()
        .unwrap();

    let mut curr_addr = module.get_entry_function().id.address;
    assert_eq!(curr_addr, 0);
    for function in &module {
        for basic_block in function {
            for instruction in basic_block {
                // Each instruction should be sequential
                assert_eq!(instruction.address, curr_addr);
                curr_addr += 1;
            }
        }
    }
}

#[test]
fn load_multiple_functions() {
    let reader = load_bytecode("multiple-functions.gs2bc").unwrap();
    let module = gbf_core::module::ModuleBuilder::new()
        .name("multiple-functions.gs2".to_string())
        .reader(Box::new(reader))
        .build()
        .unwrap();

    // Check the number of functions. In this case, it should be 6 since
    // we count the entry point function.
    let functions = module.len();
    assert_eq!(functions, 6);

    // Function 0: Entry Point
    let function = module.get_entry_function();
    let basic_blocks = function.len();

    // We have a ModuleEnd block at the end of the module
    assert_eq!(basic_blocks, 2);

    // Function 1: onCreated
    let function = module
        .get_function_by_name("onCreated".to_string())
        .unwrap();
    let basic_blocks = function.len();
    assert_eq!(basic_blocks, 1);

    // first instruction PushArray, last instruction Ret
    let basic_block = function
        .get_basic_block_id_by_start_address(function.id.address)
        .unwrap();
    let basic_block = function.get_basic_block_by_id(basic_block).unwrap();
    let instructions = basic_block.len();
    assert_eq!(instructions, 20);
    assert_eq!(
        basic_block[0],
        Instruction::new(gbf_core::opcode::Opcode::PushArray, 1)
    );
    assert_eq!(
        basic_block[19],
        Instruction::new(gbf_core::opcode::Opcode::Ret, 20)
    );

    // Function 2: foo
    let function = module.get_function_by_name("foo".to_string()).unwrap();
    let basic_blocks = function.len();
    assert_eq!(basic_blocks, 1);

    // first instruction PushArray, last instruction Ret
    let basic_block = function
        .get_basic_block_id_by_start_address(function.id.address)
        .unwrap();
    let basic_block = function.get_basic_block_by_id(basic_block).unwrap();
    let instructions = basic_block.len();
    assert_eq!(instructions, 8);

    // Function 3: bar
    let function = module.get_function_by_name("bar".to_string()).unwrap();
    let basic_blocks = function.len();
    assert_eq!(basic_blocks, 1);
    let basic_block = function
        .get_basic_block_id_by_start_address(function.id.address)
        .unwrap();
    let basic_block = function.get_basic_block_by_id(basic_block).unwrap();
    let instructions = basic_block.len();
    assert_eq!(instructions, 8);

    // Function 4: baz
    let function = module.get_function_by_name("baz".to_string()).unwrap();
    let basic_blocks = function.len();
    assert_eq!(basic_blocks, 1);
    let basic_block = function
        .get_basic_block_id_by_start_address(function.id.address)
        .unwrap();
    let basic_block = function.get_basic_block_by_id(basic_block).unwrap();
    let instructions = basic_block.len();
    assert_eq!(instructions, 5);

    // Function 5: fib
    let function = module.get_function_by_name("fib".to_string()).unwrap();
    let basic_blocks = function.len();
    assert_eq!(basic_blocks, 3);

    let basic_block = &function[0];
    let instructions = basic_block.len();
    assert_eq!(instructions, 10);
    let first_instruction = &basic_block[0];
    assert_eq!(
        first_instruction,
        &Instruction::new(gbf_core::opcode::Opcode::PushArray, 46)
    );

    let basic_block = &function[1];
    let instructions = basic_block.len();
    assert_eq!(instructions, 2);

    let basic_block = &function[2];
    let instructions = basic_block.len();
    assert_eq!(instructions, 18);

    // iterate over the instructions in the last block of the function.
    // assert each instruction comes one after the other.
    let mut address = basic_block.id.address;
    for instruction in function[2].iter() {
        assert_eq!(instruction.address, address);
        address += 1;
    }

    // For the first function output the dot representation
    let function = module[5].render_dot(CfgDotConfig::default());
    assert!(function.contains("digraph CFG {"));
}
