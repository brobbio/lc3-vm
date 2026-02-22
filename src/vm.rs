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

const MR_KBSR: u16 = 0xFE00; // keyboard status
const MR_KBDR: u16 = 0xFE02; // keyboard data

enum TRAP_CODES {
    TRAP_GETC = 0x20,  /* get character from keyboard, not echoed onto the terminal */
    TRAP_OUT = 0x21,   /* output a character */
    TRAP_PUTS = 0x22,  /* output a word string */
    TRAP_IN = 0x23,    /* get character from keyboard, echoed onto the terminal */
    TRAP_PUTSP = 0x24, /* output a byte string */
    TRAP_HALT = 0x25   /* halt the program */
};

const PC_START: u16 = 0x3000;
const MEMORY_SIZE: usize = 2_usize.power(16);


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
    reg: [u16; Register::R_COUNT],
    memory: [u16; MEMORY_SIZE],
    running: bool,
}

impl VM {
    pub fn new(&mut self) -> Self {
        self.mem = [u16; MEMORY_SIZE];
        self.reg = [u16; Registers::R_COUNT];
        self.running = false;
        
        //Setup
        self.reg[Register::R_PC] = PC_START;
        /* set the PC to starting position */
        /* 0x3000 is the default */
            
        self.reg[Register::R_COND] = ConditionFlags::FL_ZRO;

    }

    fn mem_read(mut &self, addr: usize) -> u16 {
        if (addr == MR_KBSR) {
            let mut buffer = [0; 1];
            io::stdin().read_exact(&mut buffer).unwrap();
            if buffer[0] != 0 {
                self.mem[MR_KBSR as usize] = 1 << 15;
                self.mem[MR_KBDR as usize] = buffer[0] as u16;
            } else {
                self.mem[MR_KBSR as usize] = 0;
            }

            self.mem[addr as usize]
        }
    }

    fn mem_write(mut &self, addr: usize, val: u16) {
        self.mem[addr as usize] = value;
    }

    pub fn reg(&mut self, id: usize) -> u16 {
        self.registers.get(id).copied()
    }

    pub fn turn_on(&mut self) {
        self.running = true;
    }

    pub fn is_running(&mut self) {
        self.running
    }

    pub fn set_pc(&mut self, pc: usize) {
        self.reg[Registers::R_PC] = pc;
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
            let imm5 = sign_extend(instruction & 0x1F, 5);
            self.reg[r0] = self.reg[r1].wrapping_add(imm5);
        } else {
            let r2: usize = (instruction & 0x7);
            self.reg[r0] = self.reg[r1].wrapping_add(self.reg[r2]);
        }
    
        self.update_flags(r0);
    }

    pub fn ldi(&self, instruction: u16) {
        r0: u16 = (instruction >> 9) & 0x7; // destination register
        pc_offset: u16 = sign_extend((instruction & 0x1FF, 9)); // obtain the 9-bit offset sign-extended

        self.reg[r0] = self.mem_read(self.reg::R_PC + pc_offset);
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

    pub fn load(mut &self, instruction: u16) {
        let r0: usize = (instruction >> 9) & 0x7;
        let pc_offset: u16 = sign_extend(instruction & 0x1FF, 9);
        self.reg[r0] = self.mem_read(self.reg[Register::R_PC] + pc_offset);
        self.update_flags(r0);
    }

    pub fn ldr(mut &self, instruction: u16) {
        let r0: usize = (instruction >> 9) & 0x7;
        let r1: usize = (instruction >> + 6) & 0x7;
        let offset: usize = sign_extend(instruction & 0x3F, 6);
        self.reg[r0] = self.mem_read(self.reg[r1] + offset);
        self.update_flags(r0);
    }

    pub fn lea(mut &self, instruction: u16) { // load effective address
        let r0: usize = (instruction >> 9) & 0x7;
        let pc_offset: u16 = sign_extend(instruction & 0x1FF, 9);
        self.reg[r0] = self.reg[Register::R_PC] + pc_offset;
        self.update_flags(r0); 
    }

    pub fn store(mut &self, instruction: u16) {
        let r0: usize = (instruction >> 9) & 0x7;
        let pc_offset: u16 = (sign_extend(instruction & 0x1FF, 9));
        mem_write(self.reg[Register::R_PC] + pc_offset, self.reg[r0])
    }

    pub fn store_indirect(mut &self, instruction: u16) {
        let r0: usize = (instruction >> 9) & 0x7;
        let pc_offset: u16 = sign_extend(instruction & 0x1FF, 9);
        mem_write(mem_read(self.reg[Register::R_PC] + pc_offset), se lf.reg[r0]);
    }

    pub fn store_register(mut &self, instruction: u16) {
        let r0: usize = (instruction >> 9) & 0x7;
        let r1: usize = (instruction >> 6) & 0x7;
        let offset: u16 = sign_extend(instruction & 0x3F, 6);
        mem_write(self.reg[r1] + offset, self.reg[r0]);
    }

    pub fn execute_trap_routine(mut &self, instruction: u16) {
        let operation: usize = (instruction >> 12);
        match operation {
            TRAP_CODES::TRAP_HALT => {
                self.running = false;
            }
            TRAP_CODES::TRAP_PUTS => {
                let mut i = self.reg[0] as usize;
                while self.MEM.read[i] != 0x0000 {
                    let ch = self.memory.read(i) as u8;
                    print!("{}", ch as char);
                    eprint!("{}", ch as char);
                    i += 1;
                }
                io::stdout()
                    .flush()
                    .unwrap_or_else(|_| panic!("{}", VMError::FlushFailed));
            }
            TRAP_CODES::TRAP_OUT => {
                let ch = self.reg[Register::R_R0] as u8 as char;
                print!("{}", ch);
                io::stdout().flush().unwrap();
            }
            TRAP_CODES::TRAP_IN => {
                    print!("Enter a character: ");
                    io::stdout().flush().unwrap();

                    let mut buffer = [0u8; 1];
                    io::stdin().read_exact(&mut buffer).unwrap();

                    let c: u16 = buffer[0];

                    io::stdout().write_all(&[c]).unwrap();
                    io::stdout().flush().unwrap();

                    self.reg[Register::R_R0] = c;

                    self.update_flags(Register::R_R0);
            }
            TRAP_CODES::TRAP_GETC => {
                let mut buffer = [0; 1];
                io::stdin().read_exact(&mut buffer).unwrap();
                self.reg[] = buffer[0];
                self.update_flags(self.reg[0]);
            }
            TRAP_CODES::TRAP_PUTSP => {
                let mut i: usize = self.reg[0];
                while self.mem[i] != 0x0000 {
                    let ch = self.mem[i];
                    let (ch1, ch2) = (ch & 0xFF, ch >> 8);
                    print!("{}", (ch1 as u8) as char);
                    eprint!("{}", (ch1 as u8) as char);
                    if ch2 != 0x00 {
                        print!("{}", (ch2 as u8) as char);
                        eprint!("{}", (ch2 as u8) as char);
                    }
                    i += 1;
                }
            }

        }
    }

}

