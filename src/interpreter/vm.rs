use crate::interpreter::instructions::Instruction;

struct AxeVM {
    code: Vec<u8>,
    ip: usize,
}

impl AxeVM {
    fn new(code: Vec<u8>) -> Self {
        AxeVM { code, ip: 0 }
    }

    fn exec(&mut self) {
        self.ip = 0;
        self.eval();
    }

    fn eval(&mut self) {
        loop {
            let opcode = self.code.get(self.ip).unwrap();
            self.ip += 1;
            match *opcode {
                Instruction::HALT => {
                    break;
                }
                _ => panic!("Unknown opcode: {}", opcode),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_halt() {
        let code = vec![Instruction::HALT];
        let mut vm = AxeVM::new(code);
        vm.exec();
    }
}
