pub mod opcode;
pub mod graal_io;
pub mod module;
pub mod graph;
pub mod function;
pub mod basic_block;
pub mod instruction;
pub mod operand;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
