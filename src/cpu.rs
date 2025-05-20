pub mod v30mz;
mod opcode;

// Operands that the instruction applies to

// An operand of NONE may also indicate that the operand
// is better detected by means other than checking the enum
#[derive(Clone, Copy, PartialEq, Eq)]
enum Operand {
    MEMORY,
    REGISTER,
    ACCUMULATOR,
    IMMEDIATE,
    SEGMENT,
    DIRECT,
    NONE,
}

// Amount of bits to be read
#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    M8,
    M16,
    M32,
}

enum MemOperand<'a> {
    Register(&'a mut u16),
    HighReg(&'a mut u16),
    LowReg(&'a mut u16),
    Offset(u16),
}