pub mod v30mz;
#[allow(unused)]
mod opcode;

// Operands that the instruction applies to

// An operand of NONE may also indicate that the operand
// is better detected by means other than checking the enum
#[derive(Clone, Copy, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum Operand {
    MEMORY,
    REGISTER,
    ACCUMULATOR,
    IMMEDIATE,
    IMMEDIATE_S,
    SEGMENT,
    DIRECT,
    NONE,
}

// Amount of bits to be read
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    M8,
    M16,
    M32,
}

pub enum MemOperand<'a> {
    Offset(u16),
    Register(RegisterType<'a>),
}

pub enum RegisterType<'a> {
    RW(&'a mut u16),
    RH(&'a mut u16),
    RL(&'a mut u16),
}

impl TryFrom<RegisterType<'_>> for u8 {
    type Error = ();

    fn try_from(value: RegisterType) -> Result<Self, Self::Error> {
        match value {
            RegisterType::RW(_) => Err(()),
            RegisterType::RH(rh) => Ok((*rh >> 8) as u8),
            RegisterType::RL(rl) => Ok(*rl as u8),
        }
    }
}

impl TryFrom<RegisterType<'_>> for u16 {
    type Error = ();

    fn try_from(value: RegisterType<'_>) -> Result<Self, Self::Error> {
        match value {
            RegisterType::RW(r) => Ok(*r),
            _ => Err(()),
        }
    }
}

fn swap_h(dest: u16, src: u8) -> u16 {
    (dest & 0x00FF) | ((src as u16) << 8)
}

fn swap_l(dest: u16, src: u8) -> u16 {
    (dest & 0xFF00) | (src as u16)
}
pub fn parity(x: u8) -> bool {
    let mut parity = 0;
    for i in 0..8 {
        parity += (x >> i) & 1;
    }
    parity % 2 == 0
}