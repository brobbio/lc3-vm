pub mod vm;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vm::VM;

    #[test]
    fn test_mem_write_and_read() {
        let mut vm = VM::new();

        vm.mem_write(0x3000, 0x1234);
        let value = vm.mem_read(0x3000);

        assert_eq!(value, 0x1234);
    }
}