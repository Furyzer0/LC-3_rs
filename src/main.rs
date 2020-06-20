mod opcode;
mod register;
mod memory;

use libc;
use std::io::{self, Write};
use std::env;
use std::fs::File;
use self::opcode::{OpCode, Trap};
use self::register::{Registers, R_7, PC, COND};
use self::memory::Memory;

fn read_image(path: &str, memory: &mut Memory) -> io::Result<()> {
    let mut file = File::open(path)?;
    memory.load_file(&mut file)
}

extern "C" {
    fn disable_input_buffering();
    fn restore_input_buffering();
    fn connect();
}

// Prevent overflow
fn signed_sum(a: u16, b: u16) -> u16 {
    ((a as i32) + (b as i32)) as u16
}

fn main() {
    let mut regs = Registers::new();
    let mut memory = Memory::new();
    let mut running = true;

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("lc3 [image-file1] ...");
        std::process::exit(1);
    }

    for arg in args.iter().skip(1) {
        read_image(arg, &mut memory).unwrap()
    }
    unsafe {
        connect();
        disable_input_buffering();
    }

    while running {
        let instr = memory.read(regs[PC]);
        let opcode = OpCode::parse(instr);
        //println!("PC: {:x}, opcode: {:?}", regs[PC], opcode);
        *regs.pc_reg() += 1;
        match opcode {
            OpCode::AddReg {
                dest_reg,
                src_reg1,
                src_reg2,
            } => { 
                regs[dest_reg] = signed_sum(regs[src_reg1], regs[src_reg2]);
                regs.update_cond_flags(dest_reg);
            },
            OpCode::AddImm {
                dest_reg,
                src_reg,
                imm_value,
            } => { 
                regs[dest_reg] = signed_sum(regs[src_reg], imm_value);
                regs.update_cond_flags(dest_reg);
            },
            OpCode::AndReg {
                dest_reg,
                src_reg1,
                src_reg2,
            } => {
                regs[dest_reg] = regs[src_reg1] & regs[src_reg2];
                regs.update_cond_flags(dest_reg);
            },
            OpCode::AndImm {
                dest_reg,
                src_reg,
                imm_value,
            } => {
                regs[dest_reg] = regs[src_reg] & imm_value;
                regs.update_cond_flags(dest_reg);
            },
            OpCode::Br {
                flags,
                offset,
            } => if ((flags as u16) & regs[COND]) != 0 {
                regs[PC] = signed_sum(regs[PC], offset);
            },
            OpCode::Jmp {
                reg
            } => regs[PC] = regs[reg],
            OpCode::Jsr {
                offset
            } => {
                regs[R_7] = regs[PC];
                regs[PC] = signed_sum(regs[PC], offset);
            }, 
            OpCode::Jsrr {
                reg
            } => {
                regs[R_7] = regs[PC];
                regs[PC] = regs[reg];
            }, 
            OpCode::Ld {
                reg,
                offset,
            } => {
                regs[reg] = memory.read(signed_sum(regs[PC], offset));
                regs.update_cond_flags(reg);
            },
            OpCode::Ldi {
                reg,
                offset,
            } => {
                let adress = memory.read(signed_sum(regs[PC], offset));
                regs[reg] = memory.read(adress);
                regs.update_cond_flags(reg);
            },
            OpCode::Ldr {
                dest_reg,
                src_reg,
                offset,
            } => {
                regs[dest_reg] = memory.read(signed_sum(regs[src_reg], offset));
                regs.update_cond_flags(dest_reg);
            },
            OpCode::Lea {
                reg,
                offset
            } => {
                regs[reg] = signed_sum(regs[PC], offset);
                regs.update_cond_flags(reg);
            },
            OpCode::Not {
                dest_reg,
                src_reg,
            } => {
                regs[dest_reg] = !regs[src_reg];
                regs.update_cond_flags(dest_reg);
            },
            OpCode::St {
                reg, 
                offset,
            } => memory.write(signed_sum(regs[PC], offset), regs[reg]),
            OpCode::Sti {
                reg,
                offset,
            } => {
                let adress = memory.read(signed_sum(regs[PC], offset));
                memory.write(adress, regs[reg]);
            }, 
            OpCode::Str {
                src_reg,
                base_reg,
                offset,
            } => { 
                let adress = signed_sum(regs[base_reg], offset);
                memory.write(adress, regs[src_reg]);
            },
            OpCode::Trap(trap) => match trap {
                Trap::GETC => unsafe {
                    regs[0] = libc::getchar() as u16;
                },
                Trap::OUT => {
                    print!("{}", regs[0] as u8 as char);
                    io::stdout().flush().unwrap();
                },
                Trap::PUTS => {
                    let mut c = regs[0] as usize;
                    while memory[c] != 0 {
                        print!("{}", memory[c] as u8 as char);
                        c += 1;
                    }
                    io::stdout().flush().unwrap();
                },
                Trap::IN => unsafe {
                    print!("Enter a character: ");
                    io::stdout().flush().unwrap();
                    regs[0] = libc::getchar() as u16;
                },
                Trap::PUTSP => {
                    let mut c = regs[0] as usize;
                    while memory[c] != 0 {
                        let c1 = memory[c] & 0xff;
                        print!("{}", c1 as u8 as char);
                        let c2 = memory[c] >> 8;
                        if c2 != 0 { print!("{}", c2 as u8 as char) }
                        c += 1;
                        io::stdout().flush().unwrap();
                    }
                },
                Trap::HALT => {
                    println!("HALT");
                    running = false;
                }
            },
            opcode => println!("wrong instruction: {:?}, {:x}", opcode, instr),
        }
        //thread::sleep(time::Duration::from_millis(250));
    }
    unsafe {
        restore_input_buffering();
    }
}