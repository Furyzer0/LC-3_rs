use enum_primitive_derive::Primitive;
use num_traits::FromPrimitive;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum OpCode {
    AddReg {
        dest_reg: u8,
        src_reg1: u8,
        src_reg2: u8,
    },
    AddImm {
        dest_reg: u8,
        src_reg: u8,
        imm_value: u16,
    },
    AndReg {
        dest_reg: u8,
        src_reg1: u8,
        src_reg2: u8,
    },
    AndImm {
        dest_reg: u8,
        src_reg: u8,
        imm_value: u16,
    },
    Br {
        flags: u8,
        offset: u16,
    },
    Jmp {
        reg: u8,
    },
    Jsr {
        offset: u16,
    },
    Jsrr {
        reg: u8,
    },
    Ld {
        reg: u8,
        offset: u16,
    },
    Ldi {
        reg: u8,
        offset: u16,
    },
    Ldr {
        dest_reg: u8,
        src_reg: u8,
        offset: u16,
    },
    Lea {
        reg: u8,
        offset: u16,
    },
    Not {
        dest_reg: u8,
        src_reg: u8,
    },
    St {
        reg: u8,
        offset: u16,
    },
    Sti {
        reg: u8,
        offset: u16,
    },
    Str {
        src_reg: u8,
        base_reg: u8,
        offset: u16,
    },
    Rti,
    Trap(Trap),
    Invalid
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Primitive)]
pub enum Trap {
    GETC = 0x20,  /* get character from keyboard */
    OUT = 0x21,   /* output a character */
    PUTS = 0x22,  /* output a word string */
    IN = 0x23,    /* input a string */
    PUTSP = 0x24, /* output a byte string */
    HALT = 0x25,   /* halt the program */
}

impl OpCode {
    pub fn parse(instr: u16) -> Self {
        match instr >> 12 {
            0x0 => OpCode::Br {
                flags: ((instr >> 9) & 0x7) as u8,
                offset: sign_extend(instr & 0x1ff, 9),
            },
            1 => match ((instr >> 5) & 0x1) == 0 {
                true => OpCode::AddReg {
                    dest_reg: ((instr >> 9) & 0x7) as u8,
                    src_reg1: ((instr >> 6) & 0x7) as u8,
                    src_reg2: (instr & 0x7) as u8,
                },
                false => OpCode::AddImm {
                    dest_reg: ((instr >> 9) & 0x7) as u8,
                    src_reg: ((instr >> 6) & 0x7) as u8,
                    imm_value: sign_extend(instr & 0x1f, 5),
                }
            },
            2 => OpCode::Ld {
                reg: ((instr >> 9) & 0x7) as u8,
                offset: sign_extend(instr & 0x1ff, 9),
            },
            3 => OpCode::St {
                reg: ((instr >> 9) & 0x7) as u8,
                offset: sign_extend(instr & 0x1ff, 9),
            },
            4 => match ((instr >> 11) & 0x1) == 0 {
                true => OpCode::Jsrr {
                    reg: ((instr >> 5) & 0x7) as u8,
                },
                false => OpCode::Jsr {
                    offset: sign_extend(instr & 0x3ff, 11),
                }
            },
            5 => match ((instr >> 5) & 0x1) == 0 {
                true => OpCode::AndReg {
                    dest_reg: ((instr >> 9) & 0x7) as u8,
                    src_reg1: ((instr >> 6) & 0x7) as u8,
                    src_reg2: (instr & 0x7) as u8,
                },
                false => OpCode::AndImm {
                    dest_reg: ((instr >> 9) & 0x7) as u8,
                    src_reg: ((instr >> 6) & 0x7) as u8,
                    imm_value: sign_extend(instr & 0x1f, 5),
                }
            },
            6 => OpCode::Ldr {
                dest_reg: ((instr >> 9) & 0x7) as u8,
                src_reg: ((instr >> 6) & 0x7) as u8,
                offset: sign_extend(instr & 0x3f, 6),
            },
            7 => OpCode::Str {
                src_reg: ((instr >> 9) & 0x7) as u8,
                base_reg: ((instr >> 6) & 0x7) as u8,
                offset: sign_extend(instr & 0x3f, 6),
            },
            8 => OpCode::Rti,
            9 => OpCode::Not {
                dest_reg: ((instr >> 9) & 0x7) as u8,
                src_reg: ((instr >> 6) & 0x7) as u8,  
            },
            10 => OpCode::Ldi {
                reg: ((instr >> 9) & 0x7) as u8,
                offset: sign_extend(instr & 0x1ff, 9),
            },
            11 => OpCode::Sti {
                reg: ((instr >> 9) & 0x7) as u8,
                offset: sign_extend(instr & 0x1ff, 9), 
            },
            12 => OpCode::Jmp {
                reg: ((instr >> 6) & 0x7) as u8,
            },
            14 => OpCode::Lea {
                reg: ((instr >> 9) & 0x7) as u8,
                offset: sign_extend(instr & 0x1ff, 9),
            },
            15 => match Trap::from_u16(instr & 0xff) {
                Some(trap) => OpCode::Trap(trap),
                None => OpCode::Invalid,
            },
            _ => OpCode::Invalid,
        }
    }
}

fn sign_extend(value: u16, length: usize) -> u16 {
    match value >> (length - 1) {
        1 => value | (0xffff << length),
        _ => value,
    }
}

fn sign_extend_signed(value: u16, length: usize) -> i16 {
    sign_extend(value, length) as i16
}

#[cfg(test)]
mod tests {
    use super::{sign_extend, OpCode};

    #[test]
    fn sign_extend_works() {
        assert!(sign_extend(7u16, 4) == 7u16);
        assert!(sign_extend(11u16, 4) == (-5i16 as u16));
    }

    #[test]
    fn opcode_parser_works() {
        assert!(OpCode::parse(0b_0101_001_010_1_01001) == OpCode::AndImm {
                dest_reg: 1,
                src_reg: 2,
                imm_value: 9,
            }
        );
        //TODO: more tests
    }
}