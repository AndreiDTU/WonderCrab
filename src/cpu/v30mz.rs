use std::collections::HashMap;

use bitflags::bitflags;

use crate::soc::{IOBus, MemBus};

use super::{opcode::{OpCode, CPU_OP_CODES}, swap_h, swap_l, MemOperand, Mode, Operand, RegisterType};

bitflags! {
    // http://perfectkiosk.net/stsws.html
    pub struct CpuStatus: u16 {
        const FIXED_ON_1  = 0x8000;
        const FIXED_ON_2  = 0x4000;
        const FIXED_ON_3  = 0x2000;
        const FIXED_ON_4  = 0x1000;

        const OVERFLOW    = 0x0800; // Set when the result of an operation is too large
        const DIRECTION   = 0x0400; // Specifies the direction of a memory or block operation
        const INTERRUPT   = 0x0200; // When set, interrupts will be processed
        const BREAK       = 0x0100; // When set, after each instruction executed, an exception is raised with vector 1

        const SIGN        = 0x0080; // Set when the result of an operation is negative
        const ZERO        = 0x0040; // Set when the result of an operation is zero
        const FIXED_OFF_1 = 0x0020;
        const AUX_CARRY   = 0x0010; // Similar to CY, but applies with respect to the lowest 4 bits of the operation.

        const FIXED_OFF_2 = 0x0008;
        const PARITY      = 0x0004; // Set when the number of set bits in the lower 8 bits of an operation is even, or cleared if odd.
        const FIXED_ON_5  = 0x0002;
        const CARRY       = 0x0001; // Set when an operation produces a carry or borrows.
    }
}

pub struct V30MZ {
    // REGISTERS

    // GENERAL-PURPOSE
    AW: u16, // Names ending W refer to the whole register
    BW: u16, // Names ending in L or H refer to the low
    CW: u16, // and high byte respectively
    DW: u16,

    // SEGMENT
    DS0: u16, // DATA SEGMENT 0
    DS1: u16, // DATA SEGMENT 1
    PS: u16,  // PROGRAM SEGMENT
    SS: u16,  // STACK SEGMENT

    // INDEX
    IX: u16,
    IY: u16,

    // POINTERS
    SP: u16, // STACK POINTER
    BP: u16, // BASE POINTER

    PC: u16, // PROGRAM COUNTER

    PSW: CpuStatus, // PROGRAM STATUS WORD


    // COMMUNICATION

    // MEMORY BUS

    // PROGRAM
    pub current_op: Vec<u8>,
    pub op_request: bool,

    // OPERAND
    segment_override: Option<u16>,
    pub read_requests: Vec<u32>,
    pub read_responses: HashMap<u32, u8>,
    pub write_requests: HashMap<u32, u8>,

    // I/O BUS
    pub io_responses: HashMap<u16, u8>,
    pub io_requests: Vec<u16>,
}

impl MemBus for V30MZ {
    fn read_mem(&mut self, addr: u32) -> Result<u8, ()> {
        match self.read_responses.get(&addr) {
            None => {
                self.read_requests.push(addr);
                Err(())
            }
            Some(byte) => {
                Ok(*byte)
            }
        }
    }

    fn write_mem(&mut self, addr: u32, data: u8) {
        self.write_requests.insert(addr, data);
    }
}

impl IOBus for V30MZ {
    fn read_io(&mut self, addr: u16) -> Result<u8, ()> {
        match self.io_responses.get(&addr) {
            None => {
                self.io_requests.push(addr);
                Err(())
            }
            Some(byte) => {
                Ok(*byte)
            } 
        }
    }

    fn write_io(&mut self, addr: u16, byte: u8) {
        todo!()
    }
}

impl V30MZ {
    pub fn new() -> Self {
        Self {
            AW: 0, BW: 0, CW: 0, DW: 0,
            DS0: 0, DS1: 0, PS: 0, SS: 0,
            IX: 0, IY: 0,
            SP: 0, BP: 0, 
            PC: 0,
            
            PSW: CpuStatus::from_bits_truncate(0),

            current_op: Vec::with_capacity(5), op_request: false,
            segment_override: None, read_requests: Vec::new(), read_responses: HashMap::new(), write_requests: HashMap::new(),

            io_responses: HashMap::with_capacity(2), io_requests: Vec::with_capacity(2),
        }
    }

    pub fn tick(&mut self) {
        self.write_requests.clear();
        let _ = self.execute();
    }

    pub fn execute(&mut self) -> Result<(), ()> {
        // CPU requires at least one byte of instruction code to execute
        self.op_request = self.current_op.len() == 0;
        if self.op_request {return Err(())}

        let op = &CPU_OP_CODES[self.current_op[0] as usize];

        // This will return OK only if there are no pending requests to SoC
        match op.code {
            // MOV
            0x9E => {
                let AH = (self.AW >> 8) as u8;
                let mut psw = self.PSW.bits();
                psw = swap_l(psw, AH);
                self.PSW = CpuStatus::from_bits_truncate(psw);
                Ok(())
            }
            0x9F => {
                self.AW = swap_h(self.AW, self.PSW.bits() as u8);
                Ok(())
            }
            0x88..=0x8C | 0x8E | 0xA0..=0xA3 | 0xB0..=0xBF | 0xC4..=0xC7 => self.mov(op),

            // LDEA
            0x8D => self.ldea(op.mode),

            // CVTBW
            0x98 => self.cvtbw(),

            // CVTWL
            0x99 => self.cvtwl(),

            // IN
            0xE4 | 0xE5 | 0xEC | 0xED => self.in_op(op.mode, op.op2),
                
            _ => todo!(),
        }?;

        self.finish_op();
        Ok(())
    }

    pub fn get_pc_address(&mut self) -> u32 {
        let addr = self.apply_segment(self.PC, self.PS);
        self.PC = self.PC.wrapping_add(1);
        addr
    }

    // Instructions

    fn mov(&mut self, operation: &OpCode) -> Result<(), ()> {
        // Copies the value of op2 to op1
        // or reads two u16s from op3 and copies their values to op1 and op2
        let (mode, op1, op2, op3) = (operation.mode, operation.op1, operation.op2, operation.op3);

        if (op1, op2) == (Operand::REGISTER, Operand::IMMEDIATE) {
            return self.load_register_immediate(mode);
        }

        if (op1, op2) == (Operand::MEMORY, Operand::IMMEDIATE) {
            self.expect_op_bytes(2)?;
            let byte = self.current_op[1];
            self.resolve_mem_operand(byte, mode)?;
            let imm_addr = self.get_pc_address();
            
            if mode == Mode::M8 {
                let src = self.read_mem(imm_addr)?;
                self.write_mem_operand_8(src)?;
            } else {
                let src = self.read_mem_16(imm_addr)?;
                self.PC = self.PC.wrapping_add(1);
                self.write_mem_operand_16(src)?;
            }
            return Ok(());
        }

        match op3 {
            None => {
                match mode {
                    Mode::M8 => {
                        let src = self.resolve_src_8(op2)?;
                        self.write_src_to_dest_8(op1, src)?;
                    }
                    Mode::M16 => {
                        let src = self.resolve_src_16(op2)?;
                        self.write_src_to_dest_16(op1, src)?;
                    }
                    Mode::M32 => panic!("32-bit move only valid when op3 exists"),
                }
            }
            Some(_) => {
                self.expect_op_bytes(2)?;
                let byte = self.current_op[1];
                let src = self.resolve_mem_src_32(byte)?;

                self.write_reg_operand_16(src.0)?;
                match operation.code {
                    0xC4 => self.DS1 = src.1,
                    0xC5 => self.DS0 = src.1,
                    code => panic!("Not a valid 3-term move opcode: {:02X}", code),
                }
            }
        }

        Ok(())
    }

    fn ldea(&mut self, mode: Mode) -> Result<(), ()> {
        // Calculates the offset of a memory operand and stores
        // the result into a 16-bit register.

        // LDEA requires at least one byte of operand code
        self.expect_op_bytes(2)?;

        let byte = self.current_op[1];
        let address = match self.resolve_mem_operand(byte, mode) {
            Err(_) => return Err(()),
            Ok((op, _)) => {
                op
            }
        };

        match address {
            MemOperand::Offset(offset) => self.AW = offset,
            MemOperand::Register(RegisterType::RW(r)) => self.AW = *r,
            MemOperand::Register(RegisterType::RH(rh)) => {
                let AH = *rh as u8;
                self.AW = swap_h(self.AW, AH);
            }
            MemOperand::Register(RegisterType::RL(rl)) => {
                let AL = *rl as u8;
                self.AW = swap_l(self.AW, AL);
            }
        }

        Ok(())
    }

    fn cvtbw(&mut self) -> Result<(), ()> {
        // Sign-extends AL into AW. If the highest bit of AL is clear,
        // stores 0x00 into AH. Otherwise, stores 0xFF into AH.

        let sign = self.AW & 0x0080 != 0;
        if sign {
            self.AW |= 0xFF00;
        } else {
            self.AW &= 0x00FF;
        }

        Ok(())
    }

    fn cvtwl(&mut self) -> Result<(), ()> {
        // Sign-extends AW into DW,AW. If the highest bit of AW is clear,
        // stores 0x0000 into DW. Otherwise, stores 0xFFFF into DW.

        let sign = self.AW & 0x8000 != 0;
        self.DW = if sign {0xFFFF} else {0x0000};

        Ok(())
    }

    fn in_op(&mut self, mode: Mode, src: Operand) -> Result<(), ()> {
        // Inputs the value from the I/O port pointed to by src and stores it into AL.
        // If 16-bit, inputs the value from the I/O port pointed to by src + 1 and stores it into AH.

        let addr = self.get_io_address(src)?;

        // Request either one byte to be loaded into AL
        // or two bytes to be loaded into AL and AH respectively
        match mode {
            Mode::M8 => {
                let AL = self.read_io(addr)?;

                self.AW = swap_l(self.AW, AL);
            }
            Mode::M16 => {
                let (AL, AH) = self.read_io_16(addr)?;

                self.AW = swap_l(self.AW, AL);
                self.AW = swap_h(self.AW, AH);
            }
            Mode::M32 => panic!("Unsuported mode"),
        }

        Ok(())
    }

    // Utility functions

    fn load_register_immediate(&mut self, mode: Mode) -> Result<(), ()> {
        match mode {
            Mode::M8 => {
                self.expect_op_bytes(2)?;
                let src = self.current_op[1];

                let r_bits = (self.current_op[0] & 0b111) >> 3;
                let dest = self.resolve_register_operand(r_bits, mode);
                match dest {
                    RegisterType::RH(rh) => *rh = swap_h(*rh, src),
                    RegisterType::RL(rl) => *rl = swap_l(*rl, src),
                    RegisterType::RW(_) => unreachable!(),
                }
            }
            Mode::M16 => {
                self.expect_op_bytes(3)?;
                let src = u16::from_le_bytes([self.current_op[1], self.current_op[2]]);

                let r_bits = (self.current_op[0] & 0b111) >> 3;
                let dest = self.resolve_register_operand(r_bits, mode);
                match dest {
                    RegisterType::RW(r) => *r = src,
                    _ => unreachable!(),
                }
            }
            Mode::M32 => panic!("Mode not supported for immediate values!"),
        }

        Ok(())
    }

    fn resolve_src_16(&mut self, op: Operand) -> Result<u16, ()> {
        let src = match op {
            Operand::MEMORY => {
                self.expect_op_bytes(2)?;
                let byte = self.current_op[1];

                self.resolve_mem_src_16(byte)?
            },
            Operand::REGISTER => {
                self.expect_op_bytes(2)?;

                let r_bits = (self.current_op[1] & 0b0011_1000) >> 3;
                self.resolve_register_operand(r_bits, Mode::M16).try_into()?
            }
            Operand::ACCUMULATOR => self.AW,
            Operand::IMMEDIATE => {
                self.expect_op_bytes(3)?;
                u16::from_le_bytes([self.current_op[1], self.current_op[2]])
            },
            Operand::SEGMENT => {
                self.expect_op_bytes(2)?;
                let s_bits = (self.current_op[1] & 0b0001_1000) >> 3;
                *self.resolve_segment(s_bits)
            },
            Operand::DIRECT => todo!(),
            Operand::NONE => panic!("None src not supported"),
        };

        Ok(src)
    }

    fn resolve_src_8(&mut self, op: Operand) -> Result<u8, ()> {
        match op {
            Operand::MEMORY => {
                self.expect_op_bytes(2)?;

                let src = self.resolve_mem_src_8(self.current_op[1])?;
                Ok(src)
            }
            Operand::REGISTER => {
                self.expect_op_bytes(2)?;

                let r_bits = (self.current_op[1] & 0b0011_1000) >> 3;
                self.resolve_register_operand(r_bits, Mode::M8).try_into()
            }
            Operand::ACCUMULATOR => Ok(self.AW as u8),
            Operand::IMMEDIATE => {
                self.expect_op_bytes(2)?;

                Ok(self.current_op[1])
            }
            Operand::DIRECT => todo!(),
            _ => panic!("Unsuported 8-bit source type"),
        }
    }

    fn resolve_mem_src_32(&mut self, byte: u8) -> Result<(u16, u16), ()> {
        let (mem_operand, default_segment) = self.resolve_mem_operand(byte, Mode::M16)?;

        match mem_operand {
            MemOperand::Offset(offset) => {
                let addr = self.get_physical_address(offset, default_segment);
                Ok(self.read_mem_32(addr)?)
            }
            MemOperand::Register(_) => unimplemented!()
        }
    }

    fn resolve_mem_src_16(&mut self, byte: u8) -> Result<u16, ()> {
        let (mem_operand, default_segment) = self.resolve_mem_operand(byte, Mode::M16)?;

        match mem_operand {
            MemOperand::Offset(offset) => {
                let addr = self.get_physical_address(offset, default_segment);
                self.read_mem_16(addr)
            }
            MemOperand::Register(register_type) => register_type.try_into()
        }
    }

    fn resolve_mem_src_8(&mut self, byte: u8) -> Result<u8, ()> {
        let (mem_operand, default_segment) = self.resolve_mem_operand(byte, Mode::M8)?;

        match mem_operand {
            MemOperand::Offset(offset) => {
                let addr = self.get_physical_address(offset, default_segment);
                self.read_mem(addr)
            }
            MemOperand::Register(register_type) => register_type.try_into()
        }
    }

    fn write_src_to_dest_16(&mut self, op: Operand, src: u16) -> Result<(), ()> {
        match op {
            Operand::MEMORY => self.write_mem_operand_16(src)?,
            Operand::REGISTER => self.write_reg_operand_16(src)?,
            Operand::ACCUMULATOR => self.AW = src,
            Operand::SEGMENT => self.write_to_seg_operand(src)?,
            Operand::DIRECT => todo!(),
            _ => panic!("Unsupported 16-bit destination")
        }

        Ok(())
    }

    fn write_src_to_dest_8(&mut self, op: Operand, src: u8) -> Result<(), ()> {
        match op {
            Operand::MEMORY => self.write_mem_operand_8(src)?,
            Operand::REGISTER => self.write_reg_operand_8(src)?,
            Operand::ACCUMULATOR => self.AW = swap_l(self.AW, src),
            Operand::DIRECT => todo!(),
            _ => panic!("Unsupported 8-bit destination type"),
        };

        Ok(())
    }

    fn write_mem_operand_16(&mut self, src: u16) -> Result<(), ()> {
        self.expect_op_bytes(2)?;
        let byte = self.current_op[1];
        let (mem_operand, default_segment) = self.resolve_mem_operand(byte, Mode::M16)?;

        match mem_operand {
            MemOperand::Offset(offset) => {
                let addr = self.get_physical_address(offset, default_segment);
                self.write_mem_16(addr, src);
            }
            MemOperand::Register(register_type) => match register_type {
                RegisterType::RW(r) => *r = src,
                _ => unreachable!()
            }
        }

        Ok(())
    }

    fn write_mem_operand_8(&mut self, src: u8) -> Result<(), ()> {
        self.expect_op_bytes(2)?;
        let byte = self.current_op[1];
        let (mem_operand, default_segment) = self.resolve_mem_operand(byte, Mode::M8)?;

        match mem_operand {
            MemOperand::Offset(offset) => {
                let addr = self.get_physical_address(offset, default_segment);
                self.write_mem(addr, src);
            }
            MemOperand::Register(register_type) => match register_type {
                RegisterType::RW(_) => unreachable!(),
                RegisterType::RH(rh) => *rh = swap_h(*rh, src),
                RegisterType::RL(rl) => *rl = swap_l(*rl, src),
            }
        }

        Ok(())
    }

    fn write_reg_operand_16(&mut self, src: u16) -> Result<(), ()> {
        self.expect_op_bytes(2)?;
        let r_bits = (self.current_op[1] & 0b0011_1000) >> 3;

        match self.resolve_register_operand(r_bits, Mode::M16) {
            RegisterType::RW(r) => *r = src,
            _ => unreachable!(),
        }

        Ok(())
    }

    fn write_reg_operand_8(&mut self, src: u8) -> Result<(), ()> {
        self.expect_op_bytes(2)?;
        let r_bits = (self.current_op[1] & 0b0011_1000) >> 3;

        match self.resolve_register_operand(r_bits, Mode::M8) {
            RegisterType::RW(_) => unreachable!(),
            RegisterType::RH(rh) => *rh = swap_h(*rh, src),
            RegisterType::RL(rl) => *rl = swap_l(*rl, src),
        }

        Ok(())
    }

    fn write_to_seg_operand(&mut self, src: u16) -> Result<(), ()> {
        self.expect_op_bytes(2)?;
        let s_bits = (self.current_op[1] & 0b0001_1000) >> 3;

        *self.resolve_segment(s_bits) = src;

        Ok(())
    }

    fn resolve_register_operand(&mut self, bits: u8, mode: Mode) -> RegisterType<'_> {
        match mode {
            Mode::M8 => match bits {
                0 => RegisterType::RL(&mut self.AW),
                1 => RegisterType::RL(&mut self.CW),
                2 => RegisterType::RL(&mut self.DW),
                3 => RegisterType::RL(&mut self.BW),
                4 => RegisterType::RH(&mut self.AW),
                5 => RegisterType::RH(&mut self.CW),
                6 => RegisterType::RH(&mut self.DW),
                7 => RegisterType::RH(&mut self.BW),
                e => panic!("Invalid register index: {}", e),
            }
            Mode::M16 => match bits {
                0 => RegisterType::RW(&mut self.AW),
                1 => RegisterType::RW(&mut self.CW),
                2 => RegisterType::RW(&mut self.DW),
                3 => RegisterType::RW(&mut self.BW),
                4 => RegisterType::RW(&mut self.SP),
                5 => RegisterType::RW(&mut self.BP),
                6 => RegisterType::RW(&mut self.IX),
                7 => RegisterType::RW(&mut self.IY),
                e => panic!("Invalid register index: {}", e),
            }
            _ => panic!("Invalid register addressing mode"),
        }
    }

    // Returns the operand and its default segment's value
    fn resolve_mem_operand(&mut self, byte: u8, mode: Mode) -> Result<(MemOperand, u16), ()> {
        let segment = self.DS0;
        let a = byte >> 6;
        let m = byte & 0b111;

        // When a is 3, m specifies the index of the register containing the operand's value.
        if a == 3 {return Ok((MemOperand::Register(self.resolve_register_operand(m, mode)), segment))};

        // When a is 0 and m is 6, the operand's memory offset is not given by an expression.
        // Instead, the literal 16-bit offset is present as two additional bytes of program code (low byte first).
        if a == 0 && m == 6 {
            self.op_request = self.current_op.len() < 4;
            if self.op_request {return Err(())};

            let offset = u16::from_le_bytes([self.current_op[2], self.current_op[3]]);
            return Ok((MemOperand::Offset(offset), segment));
        }

        // When a is not 3, m specifies the base of the expression to use to calculate a memory offset.
        // If BP is present, the default segment register is SS. If BP is not present, the defaut segment register is DS0.
        let (base, result_segment) = match m {
            0 => (self.BW.wrapping_add(self.IX), segment),
            1 => (self.BW.wrapping_add(self.IY), segment),
            2 => (self.BW.wrapping_add(self.IX), self.SS),
            3 => (self.BW.wrapping_add(self.IY), self.SS),
            4 => (self.IX, segment),
            5 => (self.IY, segment),
            6 => (self.BP, self.SS),
            7 => (self.BW, segment),
            _ => unreachable!()
        };

        // The offset portion of the operand's physical address is calculated by evaluating the expression base
        // and optionally adding a signed displacement offset to it.
        let displacement = match a {
            0 => 0,
            1 => {
                self.op_request = self.current_op.len() < 3;
                if self.op_request {return Err(())}

                ((self.current_op[2] as i8) as i16) as u16
            }
            2 => {
                self.op_request = self.current_op.len() < 4;
                if self.op_request {return Err(())};

                u16::from_le_bytes([self.current_op[2], self.current_op[3]])
            }
            _ => unreachable!(),
        };

        Ok((MemOperand::Offset(base.wrapping_add(displacement)), result_segment))
    }

    fn resolve_segment(&mut self, bits: u8) -> &mut u16 {
        match bits {
            0 => &mut self.DS1,
            1 => &mut self.PS,
            2 => &mut self.SS,
            3 => &mut self.DS0,
            e => panic!("Invalid segment index: {}", e),
        }
    }

    // Returns Err if the current_op is shorter than the amount of bytes
    fn expect_op_bytes(&mut self, bytes: usize) -> Result<(), ()> {
        self.op_request = self.current_op.len() < bytes;
        if self.op_request {return Err(())} else {Ok(())}
    }

    fn get_physical_address(&self, offset: u16, default_segment: u16) -> u32 {
        let segment = match self.segment_override {
            None => default_segment,
            Some(s) => s,
        };

        self.apply_segment(offset, segment)
    }

    fn apply_segment(&self, offset: u16, segment: u16) -> u32 {
        let segment = (segment as u32) << 4;
        let offset = offset as u32;
        (offset + segment) & 0xFFFFF
    }

    fn get_io_address(&mut self, src: Operand) -> Result<u16, ()> {
        // Use either the next byte padded with 0s or DW as the io_address
        match src {
            Operand::IMMEDIATE => {
                // Need at least one operand byte to access immediate value
                self.expect_op_bytes(2)?;

                Ok(self.current_op[1] as u16)
            }
            Operand::NONE => {
                Ok(self.DW)
            }
            _ => panic!("Unsupported src operand for I/O Port"),
        }
    }

    fn finish_op(&mut self) {
        self.current_op.clear();
        self.io_responses.clear();
        self.read_requests.clear();
        self.read_responses.clear();
    }
}

#[cfg(test)]
mod v30mz_test;