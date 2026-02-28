use std::{env};
//use std::io;
use std::io::{Read, Write};
// use std::process;
// use std::alloc;
// use libc::{read, write, close, fork};
// use std::fs::OpenOptions;
use libc::{termios, tcgetattr, tcsetattr};
// use std::time::{Duration, Instant, SystemTime};
// use libc::timeval;
// use libc::{pid_t, size_t};
// use termios::Termios;
use nix::sys::select::{select, FdSet};
use nix::sys::time::TimeVal;
use libc::STDIN_FILENO;
use std::env::args;
use crate::vm::VM;
use crate::vm::Register;
use std::io::stdin;
use std::os::fd::AsFd;
static mut ORIGINAL_TIO: Option<termios> = None;

mod vm;


fn disable_input_buffering() {
    unsafe {
        let fd = STDIN_FILENO;

        let mut term: termios = std::mem::zeroed();

        if libc::tcgetattr(fd, &mut term) != 0 {
            panic!("tcgetattr failed");
        }

        ORIGINAL_TIO = Some(term);

        // disable ICANON and ECHO
        term.c_lflag &= !(libc::ICANON | libc::ECHO);

        if libc::tcsetattr(fd, libc::TCSANOW, &term) != 0 {
            panic!("tcsetattr failed");
        }
    }
}

fn restore_input_buffering() {
    unsafe {
        if let Some(ref original) = ORIGINAL_TIO {
            if libc::tcsetattr(STDIN_FILENO, libc::TCSANOW, original) != 0 {
                panic!("tcsetattr failed while restoring terminal");
            }
        }
    }
}

fn check_key() -> bool {
    let stdin = stdin();

    let mut readfds = FdSet::new();
    let binding = stdin.as_fd();
    readfds.insert(&binding);

    let mut timeout = TimeVal::new(0, 0);

    match select(
        None,                 // ⚠️ modern nix infers nfds
        Some(&mut readfds),
        None,
        None,
        Some(&mut timeout),
    ) {
        Ok(n) => n > 0,
        Err(_) => false,
    }
}

#[derive(Debug, PartialEq, Eq)]
#[repr(u16)]
#[allow(dead_code)]
pub enum Opcodes
{
    BR = 0, /* branch */
    ADD = 1,    /* add  */
    LD = 2,     /* load */
    ST = 3,     /* store */
    JSR = 4,    /* jump register */
    AND = 5,    /* bitwise and */
    LDR = 6,    /* load register */
    STR = 7,    /* store register */
    RTI = 8,    /* unused */
    NOT = 9,    /* bitwise not */
    LDI = 10,    /* load indirect */
    STI = 11,    /* store indirect */
    JMP = 12,    /* jump */
    RES = 13,    /* reserved (unused) */
    LEA = 14,    /* load effective address */
    TRAP = 15   /* execute trap */
}

impl From<u16> for Opcodes {
    fn from(value: u16) -> Self {
        match value {
            0 => Opcodes::BR,
            1 => Opcodes::ADD,
            2 => Opcodes::LD,
            3 => Opcodes::ST,
            4 => Opcodes::JSR,
            5 => Opcodes::AND,
            6 => Opcodes::LDR,
            7 => Opcodes::STR,
            8 => Opcodes::RTI,
            9 => Opcodes::NOT,
            10 => Opcodes::LDI,
            11 => Opcodes::STI,
            12 => Opcodes::JMP,
            13 => Opcodes::RES,
            14 => Opcodes::LEA,
            15 => Opcodes::TRAP,
            _ => panic!("Invalid opcode"),
        }
    }
}

fn main() {
    disable_input_buffering();
    //Load arguments
    let args: Vec<String> = env::args().collect();

    let mut vm = VM::new();
    let Some(filename) = args.get(1) else {
        println!("You must provide an obj file");
        std::process::exit(1);
    };

    vm.read_image(filename);


    if args.len() < 2 {
        println!("lc3 [image-file] ... \n");
        std::process::exit(2);
    }

    vm.turn_on();

    while vm.is_running() {

        let curr_pc = vm.get_pc();
        let instruction: u16 = vm.mem_read(curr_pc);
        let operation = Opcodes::from(instruction >> 12);
        
        vm.advance_pc();
        // For debugging purposes:
        // println!(
        //     "PC: {:04X}  INSTR: {:04X}  OP: {:X}",
        //     vm.get_pc(),
        //     instruction,
        //     instruction >> 12
        // );

        match operation {
            Opcodes::ADD => {
                vm.add(instruction);
            }
            Opcodes::AND => {
                vm.and(instruction);
            }
            Opcodes::NOT => {
                vm.not(instruction);
            }
            Opcodes::BR => {
                vm.branch(instruction);
            }
            Opcodes::JMP => {
                vm.jmp(instruction);
            }
            Opcodes::JSR => {
                vm.jump(instruction);
            }
            Opcodes::LD => {
                vm.load(instruction);
            }
            Opcodes::LDI => {
                vm.ldi(instruction);
            }
            Opcodes::LDR => {
                vm.ldr(instruction);
            }
            Opcodes::LEA => {
                vm.lea(instruction);
            }
            Opcodes::ST => {
                vm.store(instruction);
            }
            Opcodes::STI => {
                vm.store_indirect(instruction);
            }
            Opcodes::STR => {
                vm.store_register(instruction);
            }
            Opcodes::TRAP => {
                let curr_pc = vm.get_pc();
                vm.set_reg(Register::R7 as usize, curr_pc);
                vm.execute_trap_routine(instruction);
            }
            Opcodes::RES => {}
            _ => {
                println!("Unknown operation code");
                std::process::abort();
            }
        }

        restore_input_buffering();
}
}