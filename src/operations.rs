pub struct Operations {
    instruction: String;
}

impl Operations {


    pub fn and(&self) {
        let r0: usize = (instr >> 9) & 0x7;
        let r1: usize = (instr >> 6) & 0x7;
        let imm_flag = (instr >> 5) & 0x1;
    
        if imm_flag != 0 {
            let imm5 = sign_extend(instr & 0x1F, 5);
            reg[r0] = reg[r1] & imm5;
        } else {
            let r2 = (instr & 0x7) as usize;
            reg[r0] = reg[r1] & reg[r2];
        }
    
        update_flags(r0);
    }

    pub fn not(mut &self) {
        let r0: usize = (instr >> 9) & 0x7;
        let r1: usize = (instr >> 6) & 0x7;

        reg[r0] = !reg[r1];
        self.update_flags(r0);

    }

    pub fn branch(mut &self) {
        let pc_offset: usize = sign_extend(instruction & 0x1FF, 9);
        let cond_flag: usize = (instruction >> 9) & 0x7;
        if (cond_flag & reg::R_COND) {
            reg::R_PC = reg::R_PC + pc_offset;
        }
    }

    pub fn jump(mut &self) {
        let long_flag: u16 = (instruction >> 11) & 1;
        self.

        let r1: usize = (instruction >> 6) & 0x7;
        reg::R_PC = reg[r1];
    }

    pub fn 
}
