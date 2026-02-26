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
pub enum Opcodes
{
    OP_BR = 0, /* branch */
    OP_ADD = 1,    /* add  */
    OP_LD = 2,     /* load */
    OP_ST = 3,     /* store */
    OP_JSR = 4,    /* jump register */
    OP_AND = 5,    /* bitwise and */
    OP_LDR = 6,    /* load register */
    OP_STR = 7,    /* store register */
    OP_RTI = 8,    /* unused */
    OP_NOT = 9,    /* bitwise not */
    OP_LDI = 10,    /* load indirect */
    OP_STI = 11,    /* store indirect */
    OP_JMP = 12,    /* jump */
    OP_RES = 13,    /* reserved (unused) */
    OP_LEA = 14,    /* load effective address */
    OP_TRAP = 15   /* execute trap */
}

impl From<u16> for Opcodes {
    fn from(value: u16) -> Self {
        match value {
            0 => Opcodes::OP_BR,
            1 => Opcodes::OP_ADD,
            2 => Opcodes::OP_LD,
            3 => Opcodes::OP_ST,
            4 => Opcodes::OP_JSR,
            5 => Opcodes::OP_AND,
            6 => Opcodes::OP_LDR,
            7 => Opcodes::OP_STR,
            8 => Opcodes::OP_RTI,
            9 => Opcodes::OP_NOT,
            10 => Opcodes::OP_LDI,
            11 => Opcodes::OP_STI,
            12 => Opcodes::OP_JMP,
            13 => Opcodes::OP_RES,
            14 => Opcodes::OP_LEA,
            15 => Opcodes::OP_TRAP,
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


    if args.len() < 2  {
        println!("lc3 [image-file] ... \n");
        std::process::exit(2);
    }

    for i in 0..args.len()  {
        if Some(vm.read_image(&args[i])).is_some()  {
            println!("Failed to load image {}", args[i]);
            std::process::exit(1);
        }
    }

    vm.turn_on();

    while vm.is_running() {
        let instruction: u16 = vm.mem_read(Register::R_PC as u16);
        let operation = Opcodes::from(instruction >> 12);

        match operation {
            Opcodes::OP_ADD => {
                vm.add(instruction);
            }
            Opcodes::OP_AND => {
                vm.and(instruction);
            }
            Opcodes::OP_NOT => {
                vm.not(instruction);
            }
            Opcodes::OP_BR => {
                vm.branch(instruction);
            }
            Opcodes::OP_JMP => {
                vm.jump(instruction);
            }
            Opcodes::OP_JSR => {
                vm.jump(instruction);
            }
            Opcodes::OP_LD => {
                vm.load(instruction);
            }
            Opcodes::OP_LDI => {
                vm.ldi(instruction);
            }
            Opcodes::OP_LDR => {
                vm.ldr(instruction);
            }
            Opcodes::OP_LEA => {
                vm.lea(instruction);
            }
            Opcodes::OP_ST => {
                vm.store(instruction);
            }
            Opcodes::OP_STI => {
                vm.store_indirect(instruction);
            }
            Opcodes::OP_STR => {
                vm.store_register(instruction);
            }
            Opcodes::OP_TRAP => {
                vm.execute_trap_routine(instruction);
            }
            Opcodes::OP_RES => {}
            _ => {
                println!("Unknown operation code");
                std::process::abort();
            }
        }

        restore_input_buffering();
    }
}
