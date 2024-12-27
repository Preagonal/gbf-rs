use gbf_rs::{
    basic_block::BasicBlockType,
    function::{Function, FunctionId},
    instruction::Instruction,
    opcode::Opcode,
    operand::Operand,
};

fn main() {
    let function_id = FunctionId::new(0, None, 0);
    let mut fun = Function::new(function_id);

    // block 1
    let block = fun.get_entry_block_mut();
    let block_id = block.id;
    block.add_instruction(Instruction::new_with_operand(
        Opcode::PushNumber,
        0,
        Operand::new_int(42),
    ));
    block.add_instruction(Instruction::new(Opcode::PushNumber, 1));
    block.add_instruction(Instruction::new(Opcode::Add, 2));
    block.add_instruction(Instruction::new_with_operand(
        Opcode::Jeq,
        3,
        Operand::new_int(8),
    ));

    // block 2
    let block2_id = fun.create_block(BasicBlockType::Exit).unwrap();
    let block2 = fun.get_block_mut(block2_id).unwrap();
    block2.add_instruction(Instruction::new_with_operand(
        Opcode::PushNumber,
        4,
        Operand::new_int(42),
    ));
    block2.add_instruction(Instruction::new(Opcode::PushNumber, 5));
    block2.add_instruction(Instruction::new(Opcode::Subtract, 6));
    block2.add_instruction(Instruction::new(Opcode::Ret, 7));

    // block 3: simple return block
    let block3_id = fun.create_block(BasicBlockType::Exit).unwrap();
    let block3 = fun.get_block_mut(block3_id).unwrap();
    block3.add_instruction(Instruction::new(Opcode::Ret, 8));

    // link blocks
    fun.add_edge(block_id, block2_id).unwrap();
    fun.add_edge(block_id, block3_id).unwrap();

    // print graph
    println!("{}", fun.to_dot());
}
