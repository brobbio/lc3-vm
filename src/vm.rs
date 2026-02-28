use std::fs::File;
use std::io::{self, Read, Write};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum VMError {
    #[error("Register Index {0} out of bounds")]
    RegisterIndexOutOfBounds(usize),
    #[error("Out of bounds PC {0:#x} should fit in 16 bits")]
    PcOutOfBounds(usize),
    #[error("IO Error failed to flush")]
    FlushFailed,
    #[error("Memory out of bounds")]
    MemoryOutOfBounds,
    #[error("Invalid opcode")]
    InvalidOpcode,
}

// impl fmt::Display for VMError {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         match self {
//             VMError::RegisterIndexOutOfBounds(_) => write!(f, "Register Index out of bounds"),
//             VMError::PcOutOfBounds(_) => write!(f, "Out of bounds PC should fit in 16 bits"),
//             VMError::FlushFailed => write!(f, "Memory out of bounds"),
//             VMError::InvalidOpcode => write!(f, "Invalid opcode"),
//             VMError::MemoryOutOfBounds => write!(f, "Memory access out of bounds"),
//         }
//     }
// }

#[derive(Copy, Clone)]
#[allow(dead_code)]
pub enum Register
{
    R0 = 0,
    R1 = 1,
    R2 = 2,
    R3 = 3,
    R4 = 4,
    R5 = 5,
    R6 = 6,
    R7 = 7,
    PC = 8, /* program counter */
    COND = 9,
    COUNT = 10
}

const MR_KBSR: u16 = 0xFE00; // keyboard status
const MR_KBDR: u16 = 0xFE02; // keyboard data

#[repr(u16)]
enum TrapCodes {
    GETC = 0x20,  /* get character from keyboard, not echoed onto the terminal */
    OUT = 0x21,   /* output a character */
    PUTS = 0x22,  /* output a word string */
    IN = 0x23,    /* get character from keyboard, echoed onto the terminal */
    PUTSP = 0x24, /* output a byte string */
    HALT = 0x25   /* halt the program */
}

impl From<u16> for TrapCodes {
    fn from(value: u16) -> Self {
        match value {
            0x20 => TrapCodes::GETC,
            0x21 => TrapCodes::OUT,
            0x22 => TrapCodes::PUTS,
            0x23 => TrapCodes::IN,
            0x24 => TrapCodes::PUTSP,
            0x25 => TrapCodes::HALT,
            _ => panic!("Invalid opcode {}", value),
        }
    }
}

const PC_START: u16 = 0x3000;
const MEMORY_SIZE: usize = 2_usize.pow(16);

#[repr(u16)]
enum ConditionFlags
{
    FlPOS = 1 << 0, /* P */
    FlZRO = 1 << 1, /* Z */
    FlNEG = 1 << 2, /* N */
}

impl From<u16> for ConditionFlags {
    fn from(value: u16) -> Self {
        match value {
            1 => ConditionFlags::FlPOS,
            2 => ConditionFlags::FlZRO,
            4 => ConditionFlags::FlNEG,
            _ => panic!("Invalid conditional flag"),
        }
    }
}

fn sign_extend(mut x: u16, bit_count: usize) -> u16 {
    if ((x >> (bit_count - 1)) & 1) != 0 { // If sign bit is 1 (neg number)
        x = x | (0xFFFF << bit_count); // fill with 1's
    }
    x
}


pub struct VM {
    reg: [u16; 10],
    mem: [u16; MEMORY_SIZE],
    running: bool,
}

impl VM {
    pub fn new() -> Self {
        let mut vm = VM {
            reg: [0; 10],
            mem: [0; MEMORY_SIZE], 
            running: false,
        };

        //Setup
        vm.set_reg(Register::PC as usize, PC_START);
        /* set the PC to starting position */
        /* 0x3000 is the default */
            
        vm.reg[Register::COND as usize] = ConditionFlags::FlZRO as u16;
        //println!("{}", vm.reg[9]);
        // println!("{}", vm.reg[..10]);
        vm
    }

     pub fn read_image(&mut self, path: &str) -> io::Result<()> {
        let mut file = File::open(path)?;
        self.read_image_file(&mut file)
    }

    fn read_image_file(&mut self,file: &mut File) -> io::Result<()> {
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)?;
    
        let origin = u16::from_be_bytes([buf[0], buf[1]]);

    
        for (i, chunk) in buf[2..].chunks_exact(2).enumerate() {
            self.mem[origin.wrapping_add(i as u16) as usize] = u16::from_be_bytes([chunk[0], chunk[1]]);
        }

        // for (i, &value) in self.mem.iter().enumerate() {
        //     if value != 0 {
        //         println!("mem[0x{:04X}] = 0x{:04X}", i, value);
        //     }
        // }
        Ok(())
    }

    pub fn advance_pc(&mut self){
        self.reg[Register::PC as usize] = self.reg[Register::PC as usize].wrapping_add(1);
    }

    pub fn get_pc(&mut self) -> u16 {
        self.reg[Register::PC as usize]
    }

    pub fn mem_read(&mut self, addr: u16) -> u16 {
        if addr == MR_KBSR.into() {
            let mut buffer = [0; 1];
            io::stdin().read_exact(&mut buffer).unwrap();
            if buffer[0] != 0 {
                self.mem[MR_KBSR as usize] = 1 << 15;
                self.mem[MR_KBDR as usize] = buffer[0] as u16;
            } else {
                self.mem[MR_KBSR as usize] = 0;
            }
        }
        self.mem[addr as usize]
        
    }

    pub fn mem_write(&mut self, addr: u16, val: u16) {
        self.mem[addr as usize] = val;
    }

    pub fn read_reg(&mut self, id: usize) -> u16 {
        self.reg.get(id).copied().expect("Warning: value must be set")
    }

    pub fn turn_on(&mut self) {
        self.running = true;
    }

    pub fn is_running(&mut self) -> bool {
        self.running
    }

    pub fn set_pc(&mut self, pc: u16) {
        self.reg[Register::PC as usize] = pc;
    }

    pub fn set_reg(&mut self, id: usize, value: u16) {
        self.reg[id] = value
    }

    fn update_flags(&mut self, r: usize) {
        // println!("I will update flag {}", r);
        if self.reg[r] == 0 {
            self.reg[Register::COND as usize] = ConditionFlags::FlZRO as u16;
        } else if self.reg[r] >> 15 != 0 {
            self.reg[Register::COND as usize] = ConditionFlags::FlNEG as u16;
        } else {
            self.reg[Register::COND as usize] = ConditionFlags::FlPOS as u16;
        }
    }

    pub fn add(&mut self, instruction: u16) {
        let r0 = ((instruction >> 9) & 0x7) as usize;
        let r1 = ((instruction >> 6) & 0x7) as usize;
        if (instruction >> 5) & 0x1 != 0 {
            let imm5 = sign_extend(instruction & 0x1F, 5);
            self.reg[r0] = self.reg[r1].wrapping_add(imm5);
        } else {
            let r2= (instruction & 0x7) as usize;
            self.reg[r0] = self.reg[r1].wrapping_add(self.reg[r2]);
        }
    
        self.update_flags(r0);
    }

    pub fn ldi(&mut self, instruction: u16) {
        let r0 = ((instruction >> 9) & 0x7) as usize;
        
        let pc_offset: u16 = sign_extend(instruction & 0x1FF, 9); // obtain the 9-bit offset sign-extended

        let addr = self.mem_read(
            self.reg[Register::PC as usize].wrapping_add(pc_offset)
        );
        self.reg[r0] = self.mem_read(addr);
        self.update_flags(r0);
    }

    pub fn and(&mut self, instruction: u16) {
        let r0 = ((instruction >> 9) & 0x7) as usize;
        let r1 = ((instruction >> 6) & 0x7) as usize;
        let imm_flag: u16 = (instruction >> 5) & 0x1;
    
        if imm_flag != 0 {
        let imm5 = sign_extend(instruction & 0x1F, 5);
        self.reg[r0] = self.reg[r1] & imm5;
        } else {
            let r2 = (instruction & 0x7) as usize;
            self.reg[r0] = self.reg[r1] & self.reg[r2];
        }
    
        self.update_flags(r0);
    }

    pub fn not(&mut self, instruction: u16) {
        let r0 = ((instruction >> 9) & 0x7) as usize;
        let r1 = ((instruction >> 6) & 0x7) as usize;

        self.reg[r0] = !self.reg[r1];
        self.update_flags(r0);
    }

    pub fn branch(&mut self, instruction: u16){
        let pc_offset: u16 = sign_extend(instruction & 0x1FF, 9);
        let cond_flag: u16 = (instruction >> 9) & 0x7;

        if cond_flag & self.reg[Register::COND as usize] != 0 {
            self.reg[Register::PC as usize] = self.reg[Register::PC as usize].wrapping_add(pc_offset);
        }
    }

    pub fn jmp(&mut self, instruction: u16) {
        let r0: u16 = (instruction >> 6) & 0x7;
        self.set_reg(Register::PC as usize, self.reg[r0 as usize]);
    }

    pub fn jump(&mut self, instruction: u16) {
        let long_flag: u16 = (instruction >> 11) & 1;
        self.reg[Register::R7 as usize] = self.reg[Register::PC as usize];

        if long_flag != 0 {
            let long_offset: u16 = sign_extend(instruction & 0x7FF, 11);
            self.reg[Register::PC as usize] = self.reg[Register::PC as usize].wrapping_add(long_offset); 
        } else {
            let r1 = ((instruction >> 6) & 0x7) as usize;
            self.reg[Register::PC as usize] = self.reg[r1];
        }
    }

    pub fn load(&mut self, instruction: u16) {
        let r0 = ((instruction >> 9) & 0x7) as usize;
        let pc_offset: u16 = sign_extend(instruction & 0x1FF, 9);
        self.reg[r0] = self.mem_read(self.reg[Register::PC as usize].wrapping_add(pc_offset));
        self.update_flags(r0);
    }

    pub fn ldr(&mut self, instruction: u16) {
        let r0  = ((instruction >> 9) & 0x7) as usize;
        let r1  = ((instruction >> 6) & 0x7) as usize;
        let offset = sign_extend(instruction & 0x3F, 6);
        self.reg[r0] = self.mem_read(self.reg[r1].wrapping_add(offset));
        self.update_flags(r0);
    }

    pub fn lea(&mut self, instruction: u16) { // load effective address
        let r0: usize = ((instruction >> 9) & 0x7) as usize;
        let pc_offset: u16 = sign_extend(instruction & 0x1FF, 9);
        self.reg[r0] = self.reg[Register::PC as usize] + pc_offset;
        self.update_flags(r0); 
    }

    pub fn store(&mut self, instruction: u16) {
        let r0 = ((instruction >> 9) & 0x7) as usize;
        let pc_offset: u16 = sign_extend(instruction & 0x1FF, 9);
        self.mem_write(self.reg[Register::PC as usize].wrapping_add(pc_offset), self.reg[r0])
    }

    pub fn store_indirect(&mut self, instruction: u16) {
        let r0: usize = ((instruction >> 9) & 0x7) as usize;
        let pc_offset: u16 = sign_extend(instruction & 0x1FF, 9);
        let indirect_addr = self.mem_read(self.reg[Register::PC as usize] + pc_offset);
        self.mem_write(indirect_addr, self.reg[r0]);
    }

    pub fn store_register(&mut self, instruction: u16) {
        let r0: usize = ((instruction >> 9) & 0x7) as usize;
        let r1: usize = ((instruction >> 6) & 0x7) as usize;
        let offset: u16 = sign_extend(instruction & 0x3F, 6);
        self.mem_write(self.reg[r1].wrapping_add(offset), self.reg[r0]);
    }

    pub fn execute_trap_routine(&mut self, instruction: u16) {
        //self.reg[Register::R7 as usize] = self.reg[Register::PC as usize];
        //println!("TRAP instruction: {:04X}", instruction);
        let operation = TrapCodes::from(instruction & 0xFF);
        match operation {
            TrapCodes::HALT => {
                self.running = false;
            }
            TrapCodes::PUTS => {
                let mut i = self.reg[0] as u16;
                while self.mem_read(i) != 0x0000 {
                    let ch = self.mem_read(i) as u8;
                    print!("{}", ch as char);
                    //eprint!("{}", ch as char);
                    i += 1;
                }
                io::stdout()
                    .flush()
                    .unwrap_or_else(|_| panic!("{}", VMError::FlushFailed));
            }
            TrapCodes::OUT => {
                let ch = self.reg[Register::R0 as usize] as u8 as char;
                print!("{}", ch);
                io::stdout().flush().unwrap();
            }
            TrapCodes::IN => {
                    print!("Enter a character: ");
                    io::stdout().flush().unwrap();

                    let mut buffer = [0u8; 1];
                    io::stdin().read_exact(&mut buffer).unwrap();

                    let c: u16 = buffer[0] as u16;

                    io::stdout().write_all(&[c.try_into().unwrap()]).unwrap();
                    io::stdout().flush().unwrap();

                    self.reg[Register::R0 as usize] = c;

                    self.update_flags(Register::R0 as usize);
            }
            TrapCodes::GETC => {
                let mut buffer = [0; 1];
                io::stdin().read_exact(&mut buffer).unwrap();
                self.reg[0] = buffer[0] as u16;
                self.update_flags(Register::R0 as usize);
            }
            TrapCodes::PUTSP => {
                let mut i = self.reg[0];
                while self.mem[i as usize] != 0x0000 {
                    let ch = self.mem[i as usize];
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

