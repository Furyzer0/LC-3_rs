mod opcode;
mod register;

use enum_primitive_derive::Primitive;
use libc;
use num_traits::ToPrimitive;
use std::io::{self, Read, Write};
use std::env;
use std::fs::File;
use self::opcode::{OpCode, Trap};
use self::register::Registers;

const R_7: usize = 7;
const PC: usize = 8;
const COND: usize = 9;
const MAX_MEMORY: usize = std::u16::MAX as usize;
const MR_KBSR: usize = 0xFE00;
const MR_KBDR: usize = 0xFE02;

// TODO: struct Memory { ... }

fn mem_read(memory: &mut [u16; MAX_MEMORY], adress: u16) -> u16 {
    unsafe {
        if adress == MR_KBSR as u16 {
            if check_key() != 0 {
                memory[MR_KBSR] = 1 << 15;
                memory[MR_KBDR] = libc::getchar() as u16;
            } else {
                memory[MR_KBSR] = 0;
            }
        }
        memory[adress as usize]
    }
} 

fn mem_write(memory: &mut [u16; MAX_MEMORY], adress: u16, value: u16) {
    memory[adress as usize] = value
}

// Because LC-3 files are little endian
fn swap16(arr: &[u8]) -> u16 {
    ((arr[0] as u16) << 8) | (arr[1] as u16)
}

// TODO be sure that is working
fn read_image_file(file: &mut File, buffer: &mut [u16]) -> io::Result<()> {
    let mut origin = [0u8; 2];
    file.read_exact(&mut origin)?;
    let origin: u16 = swap16(&origin);
    println!("origin: {:X}", origin);
    let mut vec = Vec::new();
    file.read_to_end(&mut vec)?;
    for i in 0..vec.len() / 2 {
        buffer[origin as usize + i] = swap16(&vec[2 * i..]);
    }
    Ok(())
}

fn read_image(path: &str, buffer: &mut [u16]) -> io::Result<()> {
    let mut file = File::open(path)?;
    read_image_file(&mut file, buffer)
}

extern "C" {
    fn check_key() -> libc::uint16_t;
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
    let mut memory = [0u16; MAX_MEMORY];
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
        let instr = mem_read(&mut memory, regs[PC]);
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
                regs[reg] = mem_read(&mut memory, signed_sum(regs[PC], offset));
                regs.update_cond_flags(reg);
            },
            OpCode::Ldi {
                reg,
                offset,
            } => {
                let adress = mem_read(&mut memory, signed_sum(regs[PC], offset));
                regs[reg] = mem_read(&mut memory, adress);
                regs.update_cond_flags(reg);
            },
            OpCode::Ldr {
                dest_reg,
                src_reg,
                offset,
            } => {
                regs[dest_reg] = mem_read(&mut memory, signed_sum(regs[src_reg], offset));
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
            } => mem_write(&mut memory, signed_sum(regs[PC], offset), regs[reg]),
            OpCode::Sti {
                reg,
                offset,
            } => {
                let adress = mem_read(&mut memory, signed_sum(regs[PC], offset));
                mem_write(&mut memory, adress, regs[reg]);
            }, 
            OpCode::Str {
                src_reg,
                base_reg,
                offset,
            } => { 
                let adress = signed_sum(regs[base_reg], offset);
                mem_write(&mut memory, adress, regs[src_reg]);
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