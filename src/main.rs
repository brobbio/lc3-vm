use std::{env, fs}
use std::io;
use std::io::{Read, Write};
use std::process;
use std::alloc;
use std::fs;
use libc::{read, write, close, fork};
use std::fs::OpenOptions;
use libc::{termios, tcgetattr, tcsetattr};
use std::time::{Duration, Instant, SystemTime};
use libc::timeval;
use libc::{pid_t, size_t};
use libc::{termios, tcgetattr, tcsetattr};
use nix::sys::termios;



mod memory;
mod vm;




fn main() {



    let vm = VM::new();

    //Load arguments
    let args: Vec<String> = env::args().collect()

    if (args.len() < 2 ) {
        printf("lc3 [image-file] ... \n");
        std::process::exit(2);
    }

    for ( i in 0..args.len() ) {
        if ( not read_image[args[i]] ) {
            println!("Failed to load image {}", args[i]);
            std::process::exit(1);
        }
    }


    int running = true;

    while (running) {
        let instruction: u16 = mem_read(Registers::R_PC);
        let operation: u16 = instruction >> 12;

        match operation 
            {
                Opcodes::OP_ADD {

                }
                    ////@{ADD}
                    break;
                Opcodes::OP_AND:
                    //@{AND}
                    break;
                Opcodes::OP_NOT:
                    //@{NOT}
                    break;
                Opcodes::OP_BR:
                    //@{BR}
                    break;
                Opcodes::OP_JMP:
                    //@{JMP}
                    break;
                Opcodes::OP_JSR:
                    //@{JSR}
                    break;
                Opcodes::OP_LD:
                    //@{LD}
                    break;
                Opcodes::OP_LDI:
                    //@{LDI}
                    break;
                Opcodes::OP_LDR:
                    //@{LDR}
                    break;
                Opcodes::OP_LEA:
                    //@{LEA}
                    break;
                Opcodes::OP_ST:
                    //@{ST}
                    break;
                Opcodes::OP_STI:
                    //@{STI}
                    break;
                Opcodes::OP_STR:
                    //@{STR}
                    break;
                Opcodes::OP_TRAP:
                    vm.execute_trap_routine();
                    break;
                Opcodes::OP_RES:
                Opcodes::OP_RTI:
                default:
                    //@{BAD OPCODE}
                    break;
            }
        }
    }
