mod opcode;
mod v30mz;

// Operands that the instruction applies to

// An operand of NONE may also indicate that the operand
// is better detected by means other than checking the enum
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
enum Mode {
    M8,
    M16,
    M32,
}