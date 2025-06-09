/// Contains the WonderSwan's CPU
pub mod v30mz;

/// Contains large lists of opcodes and subopcodes
#[allow(unused)]
mod opcode;

/// Operands that the instruction uses
#[derive(Debug)]
#[derive(Clone, Copy, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum Operand {
    /// The instruction contains a mod/r/m byte
    MEMORY,
    /// The instruction uses a register operand
    REGISTER,
    /// The instruction somehow uses the AW register
    ACCUMULATOR,
    /// The instruction pulls bytes directly from the program memory as an argument
    IMMEDIATE,
    /// The instruction pulls a single immediate byte and sign extends it to word length
    IMMEDIATE_S,
    /// The instruction uses a segment operand
    SEGMENT,
    /// The instruction uses immediate bytes and interprets them as a memory offset
    DIRECT,
    /// The instruction either takes no argument, or has unusual or predetermined arguments
    NONE,
}

/// Certain operations can operate in 8, 16 or 32-bit modes
#[derive(Debug)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// 8-bit mode
    M8,
    /// 16-bit mode
    M16,
    /// 32-bit mode
    M32,
}

/// The result of interpreting a mod/r/m byte
#[derive(Debug)]
pub enum MemOperand<'a> {
    /// mod/r/m byte representing an offset
    Offset(u16),
    /// mod/r/m byte representing a register
    Register(RegisterType<'a>),
}

/// The type of register a mod/r/m byte can address
#[derive(Debug)]
pub enum RegisterType<'a> {
    /// mod/r/m byte addresses a word register
    RW(&'a mut u16),
    /// mod/r/m byte addresses the high byte of a register
    RH(&'a mut u16),
    /// mod/r/m byte addresses the low byte of a register
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

#[doc(hidden)]
fn swap_h(dest: u16, src: u8) -> u16 {
    (dest & 0x00FF) | ((src as u16) << 8)
}

#[doc(hidden)]
fn swap_l(dest: u16, src: u8) -> u16 {
    (dest & 0xFF00) | (src as u16)
}

#[doc(hidden)]
pub fn parity(x: u8) -> bool {
    let mut parity = 0;
    for i in 0..8 {
        parity += (x >> i) & 1;
    }
    parity % 2 == 0
}