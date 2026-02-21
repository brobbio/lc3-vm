fn sign_extend(x: u16, bit_count: usize) {
    if ((x >> (bit_count - 1)) & 1) { // If sign bit is 1 (neg number)
        x = x | (0xFFFF << bit_count); // fill with 1's
    }
    return x;
}



pub struct VM {
    reg: Registers,
    memory: Memory,
}

impl VM {
    pub fn new(&mut self, mem, reg) {
        self.registers = reg;
        self.memory = mem;
    }

    pub fn reg(&mut self, id: usize) -> u16 {
        self.registers.get(id).copied()
    }

    pub fn set_pc(&mut self, pc: usize) {
        Registers::R_PC = pc;
    }

    pub fn set_reg(&mut self, id: usize, value: u16) {
        self.registers.set(id, value)
    }

    fn update_flags(r: u16) {
        if(self.reg[r] == 0) {
            self.reg::R_COND = FL_ZRO;
        } else if (self.reg[r] >> 15) {
            self.reg::R_COND = FL_NEG;
        } else {
            self.reg::R_COND = FL_POS;
        }
    }

    pub fn add(instruction: usize) {
        let r0: usize = (instruction >> 9) & 0x7; // discard bottom 9 bits and keep last 3 bits
        let r1: usize = (instruction >> 6) & 0x7; // discard bottom 6 bits and keep last 3 bits
    
        if ((instruction >> 5) & 0x1 != 0) {
            let imm5 = sign_extend(instr & 0x1F, 5);
            self.set_reg(r0,self.reg[r1].wrapping_add(imm5));
        } else {
            let r2 = (instruction & 0x7) as usize;
            self.set_reg(r0, self.reg[r1].wrapping_add(self.reg[r2]);
        }
    
        self.update_flags(r0);
    }

}

