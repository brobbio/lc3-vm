#[derive(Copy, Clone)]
enum Register
{
    R_R0 = 0,
    R_R1 = 1,
    R_R2 = 2,
    R_R3 = 3,
    R_R4 = 4,
    R_R5 = 5,
    R_R6 = 6,
    R_R7 = 7,
    R_PC = 8, /* program counter */
    R_COND = 9,
    R_COUNT = 10
};

enum ConditionFlags
{
    FL_POS = 1 << 0, /* P */
    FL_ZRO = 1 << 1, /* Z */
    FL_NEG = 1 << 2, /* N */
};

fn sign_extend(x: u16, bit_count: usize) {
    if ((x >> (bit_count - 1)) & 1) { // If sign bit is 1 (neg number)
        x = x | (0xFFFF << bit_count); // fill with 1's
    }
    return x;
}

pub struct VM {
    reg: [u16; 8],
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

    fn update_flags(r: usize) {
        if(self.reg[r] == 0) {
            self.reg[Register::R_COND] = FL_ZRO;
        } else if (self.reg[r] >> 15) {
            self.reg[Register::R_COND] = FL_NEG;
        } else {
            self.reg[Register::R_COND] = FL_POS;
        }
    }

    pub fn add(instruction: u16) {
        let r0: u16 = (instruction >> 9) & 0x7; // discard bottom 9 bits and keep last 3 bits
        let r1: u16 = (instruction >> 6) & 0x7; // discard bottom 6 bits and keep last 3 bits
    
        if ((instruction >> 5) & 0x1 != 0) {
            let imm5 = sign_extend(instr & 0x1F, 5);
            self.reg[r0] = self.reg[r1].wrapping_add(imm5);
        } else {
            let r2: u16 = (instruction & 0x7) as usize;
            self.reg[r0] = self.reg[r1].wrapping_add(self.reg[r2]);
        }
    
        self.update_flags(r0);
    }

    pub fn ldi(&self, instruction: u16) {
        r0: u16 = (instruction >> 9) & 0x7; // destination register
        pc_offset: u16 = sign_extend((instruction & 0x1FF, 9)); // obtain the 9-bit offset sign-extended

        self.reg[r0] = mem_read(mem_read(self.reg::R_PC + pc_offset));
        self.update_flags(r0);
    }

    pub fn and(mut &self, instruction: u16) {
        let r0: u16 = (instr >> 9) & 0x7;
        let r1: u16 = (instr >> 6) & 0x7;
        let imm_flag: u16 = (instr >> 5) & 0x1;
    
        if imm_flag != 0 {
        let imm5 = sign_extend(instr & 0x1F, 5);
        self.reg[r0] = self.reg[r1] & imm5;
        } else {
            let r2: usize = (instr & 0x7);
            self.reg[r0] = self.reg[r1] & self.reg[r2];
        }
    
        self.update_flags(r0);
    }

    pub fn not(mut &self, instruction: u16) {
        let r0: usize = (instruction >> 9) & 0x7;
        let r1: usize = (instruction >> 6) & 0x7;

        self.reg[r0] = !self.reg[r1];
        self.update_flags(r0);
    }

    pub fn branch(mut &self, instruction: u16){
        let pc_offset: u16 = sign_extend(instruction & 0x1FF, 9);
        let cond_flag: u16 = (instruction >> 9) & 0x7;

        if (cond_flag & self.reg[Register::R_COND]) {
            self.reg[Register::R_PC] = self.reg[Register::R_PC] + pc_offset;
        }
    }

    pub fn jump(mut &self, instruction: u16) {
        let long_flag: u16 = (instruction >> 11) & 1;
        self.reg[Register::R_R7] = self.reg[Register::R_PC];

        if (long_flag) {
            let long_offset: u16 = sign_extend(instruction & 0x7FF, 11);
            self.reg[Register::R_PC] = self.reg[Register::R_PC] + long_offset; 
        } else {
            let r1: u16 = (instruction >> 6) & 0x7;
            self.reg[Register::R_PC] = self.reg[r1];
        }
    }

    pub fn load(mut &self, instruction: u16)


}

