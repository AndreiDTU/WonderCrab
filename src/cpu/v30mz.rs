use std::collections::HashMap;

use bitflags::bitflags;

use crate::soc::{IOBus, MemBus};

use super::{opcode::{OpCode, CPU_OP_CODES}, swap_h, swap_l, MemOperand, Mode, Operand, RegisterType};

mod util;
mod mem_ops;
mod alu_ops;

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
    pub io_read_requests: Vec<u16>,
    pub io_write_requests: HashMap<u16, u8>,
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
                self.io_read_requests.push(addr);
                Err(())
            }
            Some(byte) => {
                Ok(*byte)
            } 
        }
    }

    fn write_io(&mut self, addr: u16, byte: u8) {
        self.io_write_requests.insert(addr, byte);
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

            io_responses: HashMap::with_capacity(2), io_read_requests: Vec::with_capacity(2), io_write_requests: HashMap::with_capacity(2),
        }
    }

    pub fn tick(&mut self) {
        self.write_requests.clear();
        let _ = self.execute();
    }

    pub fn execute(&mut self) -> Result<(), ()> {
        // CPU requires at least one byte of instruction code to execute
        self.expect_op_bytes(1)?;

        let op = &CPU_OP_CODES[self.current_op[0] as usize];

        // This will return OK only if there are no pending requests to SoC
        match op.code {
            // ADD
            0x00..=0x05 => self.add(op.op1, op.op2, op.mode),
            
            // PUSH
            0x06 | 0x0E | 0x16 | 0x1E | 0x50..=0x57 | 0x68 | 0x9C => self.push_op(op.op2),
            0x60 => Ok(self.push_r()),
            0x6A => self.push_s(),

            // POP
            0x07 | 0x17 | 0x1F | 0x58..=0x5F | 0x8F | 0x9D => self.pop_op(op.op1),
            0x61 => self.pop_r(),

            // ADDC
            0x10..=0x15 => self.addc(op.op1, op.op2, op.mode),

            // ADJ4A
            0x27 => Ok(self.adj4a()),

            // ADJ4S
            0x2F => Ok(self.adj4s()),

            // XCH
            0x86 | 0x87 | 0x91..=0x97 => self.xch(op.mode, op.op1, op.op2),

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

            // SALC
            0xD6 => Ok(self.salc()),

            // TRANS
            0xD7 => self.trans(),

            // IN
            0xE4 | 0xE5 | 0xEC | 0xED => self.in_op(op.mode, op.op2),

            // OUT
            0xE6 | 0xE7 | 0xEE | 0xEF => self.out_op(op.mode, op.op2),
                
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

    fn finish_op(&mut self) {
        self.current_op.clear();
        self.io_responses.clear();
        self.read_requests.clear();
        self.read_responses.clear();
    }
}

#[cfg(test)]
mod test;