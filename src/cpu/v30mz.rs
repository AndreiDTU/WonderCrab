use std::{cell::RefCell, rc::Rc};

use bitflags::bitflags;

use crate::bus::{io_bus::{IOBus, IOBusConnection}, mem_bus::{MemBus, MemBusConnection}};

use super::{opcode::{OpCode, CPU_OP_CODES, GROUP_2, IMMEDIATE_GROUP}, swap_h, swap_l, MemOperand, Mode, Operand, RegisterType};

mod util;
mod mem_ops;
mod alu_ops;
mod bit_ops;

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
    pc_displacement: u16,

    PSW: CpuStatus, // PROGRAM STATUS WORD

    // PROGRAM
    pub current_op: Vec<u8>,

    // MEMORY
    mem_bus: Rc<RefCell<MemBus>>,
    io_bus: Rc<RefCell<IOBus>>,
    
    segment_override: Option<u16>,
}

impl MemBusConnection for V30MZ {
    fn read_mem(&mut self, addr: u32) -> u8 {
        self.mem_bus.borrow_mut().read_mem(addr)
    }

    fn write_mem(&mut self, addr: u32, byte: u8) {
        self.mem_bus.borrow_mut().write_mem(addr, byte);
    }
}

impl IOBusConnection for V30MZ {
    fn read_io(&mut self, addr: u16) -> u8 {
        self.io_bus.borrow_mut().read_io(addr)
    }

    fn write_io(&mut self, addr: u16, byte: u8) {
        self.io_bus.borrow_mut().write_io(addr, byte);
    }
}

impl V30MZ {
    pub fn new(wram: Rc<RefCell<MemBus>>, io: Rc<RefCell<IOBus>>) -> Self {
        Self {
            AW: 0, BW: 0, CW: 0, DW: 0,
            DS0: 0, DS1: 0, PS: 0, SS: 0,
            IX: 0, IY: 0,
            SP: 0, BP: 0, 
            PC: 0, pc_displacement: 0,
            
            PSW: CpuStatus::from_bits_truncate(0xF022),

            current_op: Vec::with_capacity(8),
            segment_override: None,

            mem_bus: wram, io_bus: io,
        }
    }

    pub fn tick(&mut self) {
        self.execute();
    }

    pub fn execute(&mut self) {
        // CPU requires at least one byte of instruction code to execute
        self.expect_op_bytes(1);

        let op = &CPU_OP_CODES[self.current_op[0] as usize];

        // This will return OK only if there are no pending requests to SoC
        match op.code {
            // ADD
            0x00..=0x05 => self.add(op.op1, op.op2, op.mode),

            // PUSH
            0x06 | 0x0E | 0x16 | 0x1E | 0x50..=0x57 | 0x68 | 0x6A | 0x9C => self.push_op(op.op2),
            0x60 => self.push_r(),
            // POP
            0x07 | 0x17 | 0x1F | 0x58..=0x5F | 0x8F | 0x9D => self.pop_op(op.op2),
            0x61 => self.pop_r(),

            // ADDC
            0x10..=0x15 => self.addc(op.op1, op.op2, op.mode),

            // SUBC
            0x18..=0x1D => self.subc(op.op1, op.op2, op.mode),

            // ADJ4A
            0x27 => self.adj4a(),

            // SUB
            0x28..=0x2D => self.sub(op.op1, op.op2, op.mode),

            // ADJ4S
            0x2F => self.adj4s(),

            // ADJBA
            0x37 => self.adjba(),

            // CMP
            0x38..=0x3D => self.cmp(op.op1, op.op2, op.mode),

            // ADJBS
            0x3F => self.adjbs(),

            // Immediate Group
            0x80..=0x83 => {
                self.expect_op_bytes(2);
                let sub_op = &IMMEDIATE_GROUP[(self.current_op[1] & 0b0011_1000) as usize >> 3];
                match (op.code, sub_op.code) {
                    (_, 0) => self.add(op.op1, op.op2, op.mode),
                    (_, 2) => self.addc(op.op1, op.op2, op.mode),
                    (_, 3) => self.subc(op.op1, op.op2, op.mode),
                    (_, 5) => self.sub(op.op1, op.op2, op.mode),
                    (_, 7) => self.cmp(op.op1, op.op2, op.mode),
                    _ => todo!()
                }
            }

            // XCH
            0x86 | 0x87 | 0x91..=0x97 => self.xch(op.mode, op.op1, op.op2),

            // MOV
            0x9E => {
                let AH = (self.AW >> 8) as u8;
                let mut psw = self.PSW.bits();
                psw = swap_l(psw, AH);
                self.PSW = CpuStatus::from_bits_truncate(psw);
            }
            0x9F => {
                self.AW = swap_h(self.AW, self.PSW.bits() as u8);
            }
            0x88..=0x8C | 0x8E | 0xA0..=0xA3 | 0xB0..=0xBF | 0xC4..=0xC7 => self.mov(op),

            // LDEA
            0x8D => self.ldea(),

            // CVTBW
            0x98 => self.cvtbw(),

            // CVTWL
            0x99 => self.cvtwl(),

            // SALC
            0xD6 => self.salc(),

            // TRANS
            0xD7 => self.trans(),

            // IN
            0xE4 | 0xE5 | 0xEC | 0xED => self.in_op(op.mode, op.op2),

            // OUT
            0xE6 | 0xE7 | 0xEE | 0xEF => self.out_op(op.mode, op.op2),

            // Group 2
            0xFE | 0xFF => {
                self.expect_op_bytes(2);
                let sub_op = &GROUP_2[(self.current_op[1] & 0b0011_1000) as usize >> 3];
                match (op.code, sub_op.code) {
                    (_, 6) => self.push_op(Operand::MEMORY),
                    _ => todo!()
                }
            }
                
            code => panic!("Not yet implemented! Code: {:02X}", code),
        };

        self.finish_op();
    }

    pub fn get_pc_address(&mut self) -> u32 {
        self.apply_segment(self.PC, self.PS)
    }

    fn finish_op(&mut self) {
        self.current_op.clear();
        self.PC = self.PC.wrapping_add(self.pc_displacement);
        self.pc_displacement = 0;
    }

    fn raise_exception(&mut self, vector: u8) {
        self.PC = self.PC.wrapping_add(self.pc_displacement);

        self.push(self.PSW.bits());
        self.PSW.remove(CpuStatus::INTERRUPT);
        self.PSW.remove(CpuStatus::BREAK);
        self.push(self.PS);
        self.push(self.PC);

        (self.PS, self.PC) = self.read_mem_32(vector as u32);
    }
}