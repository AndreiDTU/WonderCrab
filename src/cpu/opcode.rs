use once_cell::sync::Lazy;

use super::*;

pub struct OpCode {
    code: u8,      // First byte
    name: String,  // Mnemonic
    op1:  Operand, // Destination
    op2:  Operand, // Source
    mode: Mode,    // Amount of bits to be read

    op3:  Option<Operand>, // Third source
}

pub struct SubOpCode {
    code: u8,
    name: String,
    mode: Option<Mode>
}

impl OpCode {
    pub fn one_byte(code: u8, name: &str, op1: Operand, op2: Operand, mode: Mode) -> Self {
        Self {code, name: name.to_string(), op1, op2, mode, op3: None}
    }

    pub fn three_term(code: u8, name: &str, op1: Operand, op2: Operand, op3: Operand, mode: Mode) -> Self {
        Self {code, name: name.to_string(), op1, op2, mode, op3: Some(op3)}
    }

    pub fn invalid(code: u8) -> Self {
        Self {code, name: "INV".to_string(), op1: Operand::NONE, op2: Operand::NONE, mode: Mode::M16, op3: None}
    }

    pub fn two_byte(code: u8, op1: Operand, op2: Operand, mode: Mode) -> Self {
        Self {code, name: "IMM".to_string(), op1, op2, mode, op3: None}
    }

    pub fn fpo1(code: u8) -> Self {
        Self {code, name: "FPO1".to_string(), op1: Operand::NONE, op2: Operand::MEMORY, mode: Mode::M16, op3: None}
    }
}

impl SubOpCode {
    pub fn normal(code: u8, name: &str) -> Self {
        Self {code, name: name.to_string(), mode: None}
    }

    pub fn overwrite(code: u8, name: &str, mode: Mode) -> Self {
        Self {code, name: name.to_string(), mode: Some(mode)}
    }
}

pub static CPU_OP_CODES: Lazy<Vec<OpCode>> = Lazy::new(|| {
    vec![
        OpCode::one_byte(0x00, "ADD",     Operand::MEMORY,      Operand::REGISTER,    Mode::M8 ),
        OpCode::one_byte(0x01, "ADD",     Operand::MEMORY,      Operand::REGISTER,    Mode::M16),
        OpCode::one_byte(0x02, "ADD",     Operand::REGISTER,    Operand::MEMORY,      Mode::M8 ),
        OpCode::one_byte(0x03, "ADD",     Operand::REGISTER,    Operand::MEMORY,      Mode::M16),
        OpCode::one_byte(0x04, "ADD",     Operand::ACCUMULATOR, Operand::IMMEDIATE,   Mode::M8 ),
        OpCode::one_byte(0x05, "ADD",     Operand::ACCUMULATOR, Operand::IMMEDIATE,   Mode::M16),
        OpCode::one_byte(0x06, "PUSH",    Operand::NONE,        Operand::SEGMENT,     Mode::M16),
        OpCode::one_byte(0x07, "POP",     Operand::NONE,        Operand::SEGMENT,     Mode::M16),

        OpCode::one_byte(0x08, "OR",      Operand::MEMORY,      Operand::REGISTER,    Mode::M8 ),
        OpCode::one_byte(0x09, "OR",      Operand::MEMORY,      Operand::REGISTER,    Mode::M16),
        OpCode::one_byte(0x0A, "OR",      Operand::REGISTER,    Operand::MEMORY,      Mode::M8 ),
        OpCode::one_byte(0x0B, "OR",      Operand::REGISTER,    Operand::MEMORY,      Mode::M16),
        OpCode::one_byte(0x0C, "OR",      Operand::ACCUMULATOR, Operand::IMMEDIATE,   Mode::M8 ),
        OpCode::one_byte(0x0D, "OR",      Operand::ACCUMULATOR, Operand::IMMEDIATE,   Mode::M16),
        OpCode::one_byte(0x0E, "PUSH",    Operand::NONE,        Operand::SEGMENT,     Mode::M16),
        OpCode::invalid(0x0F),

        OpCode::one_byte(0x10, "ADDC",    Operand::MEMORY,      Operand::REGISTER,    Mode::M8 ),
        OpCode::one_byte(0x11, "ADDC",    Operand::MEMORY,      Operand::REGISTER,    Mode::M16),
        OpCode::one_byte(0x12, "ADDC",    Operand::REGISTER,    Operand::MEMORY,      Mode::M8 ),
        OpCode::one_byte(0x13, "ADDC",    Operand::REGISTER,    Operand::MEMORY,      Mode::M16),
        OpCode::one_byte(0x14, "ADDC",    Operand::ACCUMULATOR, Operand::IMMEDIATE,   Mode::M8 ),
        OpCode::one_byte(0x15, "ADDC",    Operand::ACCUMULATOR, Operand::IMMEDIATE,   Mode::M16),
        OpCode::one_byte(0x16, "PUSH",    Operand::NONE,        Operand::SEGMENT,     Mode::M16),
        OpCode::one_byte(0x17, "POP",     Operand::NONE,        Operand::SEGMENT,     Mode::M16),

        OpCode::one_byte(0x18, "SUBC",    Operand::MEMORY,      Operand::REGISTER,    Mode::M8 ),
        OpCode::one_byte(0x19, "SUBC",    Operand::MEMORY,      Operand::REGISTER,    Mode::M16),
        OpCode::one_byte(0x1A, "SUBC",    Operand::REGISTER,    Operand::MEMORY,      Mode::M8 ),
        OpCode::one_byte(0x1B, "SUBC",    Operand::REGISTER,    Operand::MEMORY,      Mode::M16),
        OpCode::one_byte(0x1C, "SUBC",    Operand::ACCUMULATOR, Operand::IMMEDIATE,   Mode::M8 ),
        OpCode::one_byte(0x1D, "SUBC",    Operand::ACCUMULATOR, Operand::IMMEDIATE,   Mode::M16),
        OpCode::one_byte(0x1E, "PUSH",    Operand::NONE,        Operand::SEGMENT,     Mode::M16),
        OpCode::one_byte(0x1F, "POP",     Operand::NONE,        Operand::SEGMENT,     Mode::M16),

        OpCode::one_byte(0x20, "AND",     Operand::MEMORY,      Operand::REGISTER,    Mode::M8 ),
        OpCode::one_byte(0x21, "AND",     Operand::MEMORY,      Operand::REGISTER,    Mode::M16),
        OpCode::one_byte(0x22, "AND",     Operand::REGISTER,    Operand::MEMORY,      Mode::M8 ),
        OpCode::one_byte(0x23, "AND",     Operand::REGISTER,    Operand::MEMORY,      Mode::M16),
        OpCode::one_byte(0x24, "AND",     Operand::ACCUMULATOR, Operand::IMMEDIATE,   Mode::M8 ),
        OpCode::one_byte(0x25, "AND",     Operand::ACCUMULATOR, Operand::IMMEDIATE,   Mode::M16),
        OpCode::one_byte(0x26, "DS1:",    Operand::NONE,        Operand::NONE,        Mode::M16),
        OpCode::one_byte(0x27, "ADJ4A",   Operand::NONE,        Operand::NONE,        Mode::M16),

        OpCode::one_byte(0x28, "SUB",     Operand::MEMORY,      Operand::REGISTER,    Mode::M8 ),
        OpCode::one_byte(0x29, "SUB",     Operand::MEMORY,      Operand::REGISTER,    Mode::M16),
        OpCode::one_byte(0x2A, "SUB",     Operand::REGISTER,    Operand::MEMORY,      Mode::M8 ),
        OpCode::one_byte(0x2B, "SUB",     Operand::REGISTER,    Operand::MEMORY,      Mode::M16),
        OpCode::one_byte(0x2C, "SUB",     Operand::ACCUMULATOR, Operand::IMMEDIATE,   Mode::M8 ),
        OpCode::one_byte(0x2D, "SUB",     Operand::ACCUMULATOR, Operand::IMMEDIATE,   Mode::M16),
        OpCode::one_byte(0x2E, "PS:",     Operand::NONE,        Operand::NONE,        Mode::M16),
        OpCode::one_byte(0x2F, "ADJ4S",   Operand::NONE,        Operand::NONE,        Mode::M16),

        OpCode::one_byte(0x30, "XOR",     Operand::MEMORY,      Operand::REGISTER,    Mode::M8 ),
        OpCode::one_byte(0x31, "XOR",     Operand::MEMORY,      Operand::REGISTER,    Mode::M16),
        OpCode::one_byte(0x32, "XOR",     Operand::REGISTER,    Operand::MEMORY,      Mode::M8 ),
        OpCode::one_byte(0x33, "XOR",     Operand::REGISTER,    Operand::MEMORY,      Mode::M16),
        OpCode::one_byte(0x34, "XOR",     Operand::ACCUMULATOR, Operand::IMMEDIATE,   Mode::M8 ),
        OpCode::one_byte(0x35, "XOR",     Operand::ACCUMULATOR, Operand::IMMEDIATE,   Mode::M16),
        OpCode::one_byte(0x36, "SS:",     Operand::NONE,        Operand::NONE,        Mode::M16),
        OpCode::one_byte(0x37, "ADJBA",   Operand::NONE,        Operand::NONE,        Mode::M16),

        OpCode::one_byte(0x38, "CMP",     Operand::MEMORY,      Operand::REGISTER,    Mode::M8 ),
        OpCode::one_byte(0x39, "CMP",     Operand::MEMORY,      Operand::REGISTER,    Mode::M16),
        OpCode::one_byte(0x3A, "CMP",     Operand::REGISTER,    Operand::MEMORY,      Mode::M8 ),
        OpCode::one_byte(0x3B, "CMP",     Operand::REGISTER,    Operand::MEMORY,      Mode::M16),
        OpCode::one_byte(0x3C, "CMP",     Operand::ACCUMULATOR, Operand::IMMEDIATE,   Mode::M8 ),
        OpCode::one_byte(0x3D, "CMP",     Operand::ACCUMULATOR, Operand::IMMEDIATE,   Mode::M16),
        OpCode::one_byte(0x3E, "DS0:",    Operand::NONE,        Operand::NONE,        Mode::M16),
        OpCode::one_byte(0x3F, "ADJBS",   Operand::NONE,        Operand::NONE,        Mode::M16),

        OpCode::one_byte(0x40, "INC",     Operand::REGISTER,    Operand::NONE,        Mode::M16),
        OpCode::one_byte(0x41, "INC",     Operand::REGISTER,    Operand::NONE,        Mode::M16),
        OpCode::one_byte(0x42, "INC",     Operand::REGISTER,    Operand::NONE,        Mode::M16),
        OpCode::one_byte(0x43, "INC",     Operand::REGISTER,    Operand::NONE,        Mode::M16),
        OpCode::one_byte(0x44, "INC",     Operand::REGISTER,    Operand::NONE,        Mode::M16),
        OpCode::one_byte(0x45, "INC",     Operand::REGISTER,    Operand::NONE,        Mode::M16),
        OpCode::one_byte(0x46, "INC",     Operand::REGISTER,    Operand::NONE,        Mode::M16),
        OpCode::one_byte(0x47, "INC",     Operand::REGISTER,    Operand::NONE,        Mode::M16),

        OpCode::one_byte(0x48, "DEC",     Operand::REGISTER,    Operand::NONE,        Mode::M16),
        OpCode::one_byte(0x49, "DEC",     Operand::REGISTER,    Operand::NONE,        Mode::M16),
        OpCode::one_byte(0x4A, "DEC",     Operand::REGISTER,    Operand::NONE,        Mode::M16),
        OpCode::one_byte(0x4B, "DEC",     Operand::REGISTER,    Operand::NONE,        Mode::M16),
        OpCode::one_byte(0x4C, "DEC",     Operand::REGISTER,    Operand::NONE,        Mode::M16),
        OpCode::one_byte(0x4D, "DEC",     Operand::REGISTER,    Operand::NONE,        Mode::M16),
        OpCode::one_byte(0x4E, "DEC",     Operand::REGISTER,    Operand::NONE,        Mode::M16),
        OpCode::one_byte(0x4F, "DEC",     Operand::REGISTER,    Operand::NONE,        Mode::M16),

        OpCode::one_byte(0x50, "PUSH",    Operand::NONE,        Operand::REGISTER,    Mode::M16),
        OpCode::one_byte(0x51, "PUSH",    Operand::NONE,        Operand::REGISTER,    Mode::M16),
        OpCode::one_byte(0x52, "PUSH",    Operand::NONE,        Operand::REGISTER,    Mode::M16),
        OpCode::one_byte(0x53, "PUSH",    Operand::NONE,        Operand::REGISTER,    Mode::M16),
        OpCode::one_byte(0x54, "PUSH",    Operand::NONE,        Operand::REGISTER,    Mode::M16),
        OpCode::one_byte(0x55, "PUSH",    Operand::NONE,        Operand::REGISTER,    Mode::M16),
        OpCode::one_byte(0x56, "PUSH",    Operand::NONE,        Operand::REGISTER,    Mode::M16),
        OpCode::one_byte(0x57, "PUSH",    Operand::NONE,        Operand::REGISTER,    Mode::M16),

        OpCode::one_byte(0x58, "POP",     Operand::NONE,        Operand::REGISTER,    Mode::M16),
        OpCode::one_byte(0x59, "POP",     Operand::NONE,        Operand::REGISTER,    Mode::M16),
        OpCode::one_byte(0x5A, "POP",     Operand::NONE,        Operand::REGISTER,    Mode::M16),
        OpCode::one_byte(0x5B, "POP",     Operand::NONE,        Operand::REGISTER,    Mode::M16),
        OpCode::one_byte(0x5C, "POP",     Operand::NONE,        Operand::REGISTER,    Mode::M16),
        OpCode::one_byte(0x5D, "POP",     Operand::NONE,        Operand::REGISTER,    Mode::M16),
        OpCode::one_byte(0x5E, "POP",     Operand::NONE,        Operand::REGISTER,    Mode::M16),
        OpCode::one_byte(0x5F, "POP",     Operand::NONE,        Operand::REGISTER,    Mode::M16),

        OpCode::one_byte(0x60, "PUSH",    Operand::NONE,        Operand::NONE,        Mode::M16),
        OpCode::one_byte(0x61, "POP",     Operand::NONE,        Operand::NONE,        Mode::M16),
        OpCode::one_byte(0x62, "CHKIND",  Operand::REGISTER,    Operand::MEMORY,      Mode::M32),
        OpCode::invalid(0x63),
        OpCode::invalid(0x64),
        OpCode::invalid(0x65),
        OpCode::invalid(0x66),
        OpCode::invalid(0x67),

        OpCode::one_byte(0x68, "PUSH",    Operand::NONE,        Operand::IMMEDIATE,   Mode::M16),
        OpCode::three_term(0x69, "MUL", Operand::REGISTER, Operand::MEMORY, Operand::IMMEDIATE, Mode::M16),
        OpCode::one_byte(0x6A, "PUSH",    Operand::NONE,        Operand::IMMEDIATE,   Mode::M16),
        OpCode::three_term(0x6B, "MUL", Operand::REGISTER, Operand::MEMORY, Operand::IMMEDIATE, Mode::M8),
        OpCode::one_byte(0x6C, "INMB",    Operand::NONE,        Operand::NONE,        Mode::M8 ),
        OpCode::one_byte(0x6D, "INMW",    Operand::NONE,        Operand::NONE,        Mode::M16),
        OpCode::one_byte(0x6E, "OUTMB",   Operand::NONE,        Operand::NONE,        Mode::M8 ),
        OpCode::one_byte(0x6F, "OUTMW",   Operand::NONE,        Operand::NONE,        Mode::M16),

        OpCode::one_byte(0x70, "BV",      Operand::NONE,        Operand::NONE,        Mode::M8 ),
        OpCode::one_byte(0x71, "BNV",     Operand::NONE,        Operand::NONE,        Mode::M8 ),
        OpCode::one_byte(0x72, "BC",      Operand::NONE,        Operand::NONE,        Mode::M8 ),
        OpCode::one_byte(0x73, "BNC",     Operand::NONE,        Operand::NONE,        Mode::M8 ),
        OpCode::one_byte(0x74, "BE",      Operand::NONE,        Operand::NONE,        Mode::M8 ),
        OpCode::one_byte(0x75, "BNE",     Operand::NONE,        Operand::NONE,        Mode::M8 ),
        OpCode::one_byte(0x76, "BNH",     Operand::NONE,        Operand::NONE,        Mode::M8 ),
        OpCode::one_byte(0x77, "BH",      Operand::NONE,        Operand::NONE,        Mode::M8 ),

        OpCode::one_byte(0x78, "BN",      Operand::NONE,        Operand::NONE,        Mode::M8 ),
        OpCode::one_byte(0x79, "BP",      Operand::NONE,        Operand::NONE,        Mode::M8 ),
        OpCode::one_byte(0x7A, "BPE",     Operand::NONE,        Operand::NONE,        Mode::M8 ),
        OpCode::one_byte(0x7B, "BPO",     Operand::NONE,        Operand::NONE,        Mode::M8 ),
        OpCode::one_byte(0x7C, "BLT",     Operand::NONE,        Operand::NONE,        Mode::M8 ),
        OpCode::one_byte(0x7D, "BGE",     Operand::NONE,        Operand::NONE,        Mode::M8 ),
        OpCode::one_byte(0x7E, "BLE",     Operand::NONE,        Operand::NONE,        Mode::M8 ),
        OpCode::one_byte(0x7F, "BGT",     Operand::NONE,        Operand::NONE,        Mode::M8 ),

        OpCode::two_byte(0x80, Operand::MEMORY, Operand::IMMEDIATE, Mode::M8),
        OpCode::two_byte(0x81, Operand::MEMORY, Operand::IMMEDIATE, Mode::M8),
        OpCode::two_byte(0x82, Operand::MEMORY, Operand::IMMEDIATE, Mode::M8),
        OpCode::two_byte(0x83, Operand::MEMORY, Operand::IMMEDIATE, Mode::M8),
        OpCode::one_byte(0x84, "TEST",    Operand::MEMORY,      Operand::REGISTER,    Mode::M8 ),
        OpCode::one_byte(0x85, "TEST",    Operand::MEMORY,      Operand::REGISTER,    Mode::M16),
        OpCode::one_byte(0x86, "XCH",     Operand::MEMORY,      Operand::REGISTER,    Mode::M8 ),
        OpCode::one_byte(0x87, "XCH",     Operand::MEMORY,      Operand::REGISTER,    Mode::M16),

        OpCode::one_byte(0x88, "MOV",     Operand::MEMORY,      Operand::REGISTER,    Mode::M8 ),
        OpCode::one_byte(0x89, "MOV",     Operand::MEMORY,      Operand::REGISTER,    Mode::M16),
        OpCode::one_byte(0x8A, "MOV",     Operand::REGISTER,    Operand::MEMORY,      Mode::M8 ),
        OpCode::one_byte(0x8B, "MOV",     Operand::REGISTER,    Operand::MEMORY,      Mode::M16),
        OpCode::one_byte(0x8C, "MOV",     Operand::MEMORY,      Operand::SEGMENT,     Mode::M16),
        OpCode::one_byte(0x8D, "LDEA",    Operand::REGISTER,    Operand::MEMORY,      Mode::M16),
        OpCode::one_byte(0x8E, "MOV",     Operand::SEGMENT,     Operand::MEMORY,      Mode::M16),
        OpCode::one_byte(0x8F, "POP",     Operand::MEMORY,      Operand::NONE,        Mode::M16),
        
        OpCode::one_byte(0x90, "NOP",     Operand::NONE,        Operand::NONE,        Mode::M16),
        OpCode::one_byte(0x91, "XCH",     Operand::ACCUMULATOR, Operand::REGISTER,    Mode::M16),
        OpCode::one_byte(0x92, "XCH",     Operand::ACCUMULATOR, Operand::REGISTER,    Mode::M16),
        OpCode::one_byte(0x93, "XCH",     Operand::ACCUMULATOR, Operand::REGISTER,    Mode::M16),
        OpCode::one_byte(0x94, "XCH",     Operand::ACCUMULATOR, Operand::REGISTER,    Mode::M16),
        OpCode::one_byte(0x95, "XCH",     Operand::ACCUMULATOR, Operand::REGISTER,    Mode::M16),
        OpCode::one_byte(0x96, "XCH",     Operand::ACCUMULATOR, Operand::REGISTER,    Mode::M16),
        OpCode::one_byte(0x97, "XCH",     Operand::ACCUMULATOR, Operand::REGISTER,    Mode::M16),

        OpCode::one_byte(0x98, "CVTBW",   Operand::NONE,        Operand::NONE,        Mode::M16),
        OpCode::one_byte(0x99, "CVTWL",   Operand::NONE,        Operand::NONE,        Mode::M16),
        OpCode::one_byte(0x9A, "CALL",    Operand::NONE,        Operand::IMMEDIATE,   Mode::M32),
        OpCode::one_byte(0x9B, "POLL",    Operand::NONE,        Operand::NONE,        Mode::M16),
        OpCode::one_byte(0x9C, "PUSH",    Operand::NONE,        Operand::NONE,        Mode::M16),
        OpCode::one_byte(0x9D, "POP",     Operand::NONE,        Operand::NONE,        Mode::M16),
        OpCode::one_byte(0x9E, "MOV",     Operand::NONE,        Operand::ACCUMULATOR, Mode::M8 ),
        OpCode::one_byte(0x9F, "MOV",     Operand::ACCUMULATOR, Operand::NONE,        Mode::M8 ),

        OpCode::one_byte(0xA0, "MOV",     Operand::ACCUMULATOR, Operand::DIRECT,      Mode::M8 ),
        OpCode::one_byte(0xA1, "MOV",     Operand::ACCUMULATOR, Operand::DIRECT,      Mode::M16),
        OpCode::one_byte(0xA2, "MOV",     Operand::DIRECT,      Operand::ACCUMULATOR, Mode::M8 ),
        OpCode::one_byte(0xA3, "MOV",     Operand::DIRECT,      Operand::ACCUMULATOR, Mode::M16),
        OpCode::one_byte(0xA4, "MOVBKB",  Operand::NONE,        Operand::NONE,        Mode::M8 ),
        OpCode::one_byte(0xA5, "MOVBKW",  Operand::NONE,        Operand::NONE,        Mode::M16),
        OpCode::one_byte(0xA6, "CMPBKB",  Operand::NONE,        Operand::NONE,        Mode::M8 ),
        OpCode::one_byte(0xA7, "CMPBKW",  Operand::NONE,        Operand::NONE,        Mode::M16),

        OpCode::one_byte(0xA8, "TEST",    Operand::ACCUMULATOR, Operand::IMMEDIATE,   Mode::M8 ),
        OpCode::one_byte(0xA9, "TEST",    Operand::ACCUMULATOR, Operand::IMMEDIATE,   Mode::M16),
        OpCode::one_byte(0xAA, "STMB",    Operand::NONE,        Operand::ACCUMULATOR, Mode::M8 ),
        OpCode::one_byte(0xAB, "STMW",    Operand::NONE,        Operand::ACCUMULATOR, Mode::M16),
        OpCode::one_byte(0xAC, "LDMB",    Operand::ACCUMULATOR, Operand::NONE,        Mode::M8 ),
        OpCode::one_byte(0xAD, "LDMW",    Operand::ACCUMULATOR, Operand::NONE,        Mode::M16),
        OpCode::one_byte(0xAE, "CMPMB",   Operand::NONE,        Operand::ACCUMULATOR, Mode::M8 ),
        OpCode::one_byte(0xAF, "CMPMW",   Operand::NONE,        Operand::ACCUMULATOR, Mode::M16),

        OpCode::one_byte(0xB0, "MOV",     Operand::REGISTER,    Operand::IMMEDIATE,   Mode::M8 ),
        OpCode::one_byte(0xB1, "MOV",     Operand::REGISTER,    Operand::IMMEDIATE,   Mode::M8 ),
        OpCode::one_byte(0xB2, "MOV",     Operand::REGISTER,    Operand::IMMEDIATE,   Mode::M8 ),
        OpCode::one_byte(0xB3, "MOV",     Operand::REGISTER,    Operand::IMMEDIATE,   Mode::M8 ),
        OpCode::one_byte(0xB4, "MOV",     Operand::REGISTER,    Operand::IMMEDIATE,   Mode::M8 ),
        OpCode::one_byte(0xB5, "MOV",     Operand::REGISTER,    Operand::IMMEDIATE,   Mode::M8 ),
        OpCode::one_byte(0xB6, "MOV",     Operand::REGISTER,    Operand::IMMEDIATE,   Mode::M8 ),
        OpCode::one_byte(0xB7, "MOV",     Operand::REGISTER,    Operand::IMMEDIATE,   Mode::M8 ),

        OpCode::one_byte(0xB8, "MOV",     Operand::REGISTER,    Operand::IMMEDIATE,   Mode::M16),
        OpCode::one_byte(0xB9, "MOV",     Operand::REGISTER,    Operand::IMMEDIATE,   Mode::M16),
        OpCode::one_byte(0xBA, "MOV",     Operand::REGISTER,    Operand::IMMEDIATE,   Mode::M16),
        OpCode::one_byte(0xBB, "MOV",     Operand::REGISTER,    Operand::IMMEDIATE,   Mode::M16),
        OpCode::one_byte(0xBC, "MOV",     Operand::REGISTER,    Operand::IMMEDIATE,   Mode::M16),
        OpCode::one_byte(0xBD, "MOV",     Operand::REGISTER,    Operand::IMMEDIATE,   Mode::M16),
        OpCode::one_byte(0xBE, "MOV",     Operand::REGISTER,    Operand::IMMEDIATE,   Mode::M16),
        OpCode::one_byte(0xBF, "MOV",     Operand::REGISTER,    Operand::IMMEDIATE,   Mode::M16),

        OpCode::two_byte(0xC0, Operand::MEMORY, Operand::IMMEDIATE, Mode::M8),
        OpCode::two_byte(0xC1, Operand::MEMORY, Operand::IMMEDIATE, Mode::M16),
        OpCode::one_byte(0xC2, "RETN",    Operand::NONE,        Operand::IMMEDIATE,   Mode::M16),
        OpCode::one_byte(0xC3, "RETN",    Operand::NONE,        Operand::NONE,        Mode::M16),
        OpCode::three_term(0xC4, "MOV", Operand::SEGMENT, Operand::REGISTER, Operand::MEMORY, Mode::M32),
        OpCode::three_term(0xC5, "MOV", Operand::SEGMENT, Operand::REGISTER, Operand::MEMORY, Mode::M32),
        OpCode::one_byte(0xC6, "MOV",     Operand::MEMORY,      Operand::IMMEDIATE,   Mode::M8 ),
        OpCode::one_byte(0xC7, "MOV",     Operand::MEMORY,      Operand::IMMEDIATE,   Mode::M16),

        OpCode::one_byte(0xC8, "PREPARE", Operand::IMMEDIATE,   Operand::IMMEDIATE,   Mode::M16),
        OpCode::one_byte(0xC9, "DISPOSE", Operand::NONE,        Operand::NONE,        Mode::M16),
        OpCode::one_byte(0xCA, "RETF",    Operand::NONE,        Operand::IMMEDIATE,   Mode::M16),
        OpCode::one_byte(0xCB, "RETF",    Operand::NONE,        Operand::NONE,        Mode::M16),
        OpCode::one_byte(0xCC, "BRK",     Operand::NONE,        Operand::NONE,        Mode::M16),
        OpCode::one_byte(0xCD, "BRK",     Operand::NONE,        Operand::IMMEDIATE,   Mode::M8 ),
        OpCode::one_byte(0xCE, "BRKV",    Operand::NONE,        Operand::NONE,        Mode::M16),
        OpCode::one_byte(0xCF, "RETI",    Operand::NONE,        Operand::NONE,        Mode::M16),

        OpCode::two_byte(0xD0, Operand::MEMORY, Operand::NONE, Mode::M8 ),
        OpCode::two_byte(0xD1, Operand::MEMORY, Operand::NONE, Mode::M16),
        OpCode::two_byte(0xD2, Operand::MEMORY, Operand::NONE, Mode::M8 ),
        OpCode::two_byte(0xD3, Operand::MEMORY, Operand::NONE, Mode::M16),
        OpCode::one_byte(0xD4, "CVTBD",   Operand::ACCUMULATOR, Operand::IMMEDIATE,   Mode::M8 ),
        OpCode::one_byte(0xD5, "CVTDB",   Operand::ACCUMULATOR, Operand::IMMEDIATE,   Mode::M8 ),
        OpCode::one_byte(0xD6, "SALC",    Operand::NONE,        Operand::NONE,        Mode::M16),
        OpCode::one_byte(0xD7, "TRANS",   Operand::NONE,        Operand::NONE,        Mode::M16),

        OpCode::fpo1(0xD8),
        OpCode::fpo1(0xD9),
        OpCode::fpo1(0xDA),
        OpCode::fpo1(0xDB),
        OpCode::fpo1(0xDC),
        OpCode::fpo1(0xDD),
        OpCode::fpo1(0xDE),
        OpCode::fpo1(0xDF),

        OpCode::one_byte(0xE0, "DBNZNE",  Operand::NONE,        Operand::NONE,        Mode::M8 ),
        OpCode::one_byte(0xE1, "DBNZE",   Operand::NONE,        Operand::NONE,        Mode::M8 ),
        OpCode::one_byte(0xE2, "DBNZ",    Operand::NONE,        Operand::NONE,        Mode::M8 ),
        OpCode::one_byte(0xE3, "BCWZ",    Operand::NONE,        Operand::NONE,        Mode::M8 ),
        OpCode::one_byte(0xE4, "IN",      Operand::ACCUMULATOR, Operand::IMMEDIATE,   Mode::M8 ),
        OpCode::one_byte(0xE5, "IN",      Operand::ACCUMULATOR, Operand::IMMEDIATE,   Mode::M8 ),
        OpCode::one_byte(0xE6, "OUT",     Operand::IMMEDIATE,   Operand::ACCUMULATOR, Mode::M8 ),
        OpCode::one_byte(0xE7, "OUT",     Operand::IMMEDIATE,   Operand::ACCUMULATOR, Mode::M8 ),
        
        OpCode::one_byte(0xE8, "CALL",    Operand::NONE,        Operand::NONE,        Mode::M16),
        OpCode::one_byte(0xE9, "BR",      Operand::NONE,        Operand::NONE,        Mode::M16),
        OpCode::one_byte(0xEA, "BR",      Operand::NONE,        Operand::IMMEDIATE,   Mode::M32),
        OpCode::one_byte(0xEB, "BR",      Operand::NONE,        Operand::NONE,        Mode::M8 ),
        OpCode::one_byte(0xEC, "IN",      Operand::NONE,        Operand::NONE,        Mode::M16),
        OpCode::one_byte(0xED, "IN",      Operand::NONE,        Operand::NONE,        Mode::M16),
        OpCode::one_byte(0xEE, "OUT",     Operand::NONE,        Operand::NONE,        Mode::M16),
        OpCode::one_byte(0xEF, "OUT",     Operand::NONE,        Operand::NONE,        Mode::M16),

        OpCode::one_byte(0xF0, "BUSLOCK", Operand::NONE,        Operand::NONE,        Mode::M16),
        OpCode::invalid(0xF1),
        OpCode::one_byte(0xF2, "REPNE",   Operand::NONE,        Operand::NONE,        Mode::M16),
        OpCode::one_byte(0xF3, "REP",     Operand::NONE,        Operand::NONE,        Mode::M16),
        OpCode::one_byte(0xF4, "HALT",    Operand::NONE,        Operand::NONE,        Mode::M16),
        OpCode::one_byte(0xF5, "NOT1",    Operand::NONE,        Operand::NONE,        Mode::M16),
        OpCode::two_byte(0xF6, Operand::MEMORY, Operand::IMMEDIATE, Mode::M8 ),
        OpCode::two_byte(0xF7, Operand::MEMORY, Operand::IMMEDIATE, Mode::M16),

        OpCode::one_byte(0xF8, "CLR1",    Operand::NONE,        Operand::NONE,        Mode::M16),
        OpCode::one_byte(0xF9, "SET1",    Operand::NONE,        Operand::NONE,        Mode::M16),
        OpCode::one_byte(0xFA, "DI",      Operand::NONE,        Operand::NONE,        Mode::M16),
        OpCode::one_byte(0xFB, "EI",      Operand::NONE,        Operand::NONE,        Mode::M16),
        OpCode::one_byte(0xFC, "CLR1",    Operand::NONE,        Operand::NONE,        Mode::M16),
        OpCode::one_byte(0xFD, "SET1",    Operand::NONE,        Operand::NONE,        Mode::M16),
        OpCode::two_byte(0xFE, Operand::MEMORY, Operand::NONE, Mode::M8 ),
        OpCode::two_byte(0xFF, Operand::MEMORY, Operand::NONE, Mode::M16),
    ]
});

pub static IMMEDIATE_GROUP: Lazy<Vec<SubOpCode>> = Lazy::new(|| {
    vec![
        SubOpCode::normal(0b000, "ADD" ),
        SubOpCode::normal(0b001, "OR"  ),
        SubOpCode::normal(0b010, "ADDC"),
        SubOpCode::normal(0b011, "SUBC"),
        SubOpCode::normal(0b100, "AND" ),
        SubOpCode::normal(0b101, "SUB" ),
        SubOpCode::normal(0b110, "XOR" ),
        SubOpCode::normal(0b111, "CMP" ),
    ]
});

pub static SHIFT_GROUP: Lazy<Vec<SubOpCode>> = Lazy::new(|| {
    vec![
        SubOpCode::normal(0b000, "ROL" ),
        SubOpCode::normal(0b001, "ROR" ),
        SubOpCode::normal(0b010, "ROLC"),
        SubOpCode::normal(0b011, "RORC"),
        SubOpCode::normal(0b100, "SHL" ),
        SubOpCode::normal(0b101, "SHR" ),
        SubOpCode::normal(0b110, "INV" ),
        SubOpCode::normal(0b111, "SHRA"),
    ]
});

pub static GROUP_1: Lazy<Vec<SubOpCode>> = Lazy::new(|| {
    vec![
        SubOpCode::normal(0b000, "TEST"),
        SubOpCode::normal(0b001, "INV" ),
        SubOpCode::normal(0b010, "NOT" ),
        SubOpCode::normal(0b011, "NEG" ),
        SubOpCode::normal(0b100, "MULU"),
        SubOpCode::normal(0b101, "MUL" ),
        SubOpCode::normal(0b110, "DIVU"),
        SubOpCode::normal(0b111, "DIV" ),
    ]
});

pub static GROUP_2: Lazy<Vec<SubOpCode>> = Lazy::new(|| {
    vec![
        SubOpCode::normal(0b000, "INC"),
        SubOpCode::normal(0b001, "DEC" ),
        SubOpCode::overwrite(0b010, "CALL", Mode::M16),
        SubOpCode::overwrite(0b011, "CALL", Mode::M32),
        SubOpCode::overwrite(0b100, "BR",   Mode::M16),
        SubOpCode::overwrite(0b101, "BR",   Mode::M32),
        SubOpCode::overwrite(0b110, "PUSH", Mode::M16),
        SubOpCode::normal(0b111, "INV" ),
    ]
});