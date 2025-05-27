use std::{cell::RefCell, collections::HashMap, rc::Rc};

use bitflags::bitflags;

use crate::bus::{io_bus::{IOBus, IOBusConnection}, mem_bus::{MemBus, MemBusConnection, Owner}};

use super::{opcode::{OpCode, CPU_OP_CODES, GROUP_1, GROUP_2, IMMEDIATE_GROUP, SHIFT_GROUP}, swap_h, swap_l, MemOperand, Mode, Operand, RegisterType};

mod util;
mod mem_ops;
mod alu_ops;
mod bit_ops;
mod ctrl_ops;
mod block_ops;

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
    segment_override: Option<u16>,
    halt: bool, rep: bool, rep_z: bool,

    // MEMORY
    mem_bus: Rc<RefCell<MemBus>>,
    io_bus: Rc<RefCell<IOBus>>,

    // MEMORY BUFFER
    mem_buffer: HashMap<u32, u8>,
    io_buffer: HashMap<u16, u8>,

    // TIMING
    cycles: u8,
    base: u8,
}

impl MemBusConnection for V30MZ {
    fn read_mem(&mut self, addr: u32) -> u8 {
        self.mem_bus.borrow_mut().read_mem(addr)
    }

    fn write_mem(&mut self, addr: u32, byte: u8) {
        self.mem_buffer.insert(addr, byte);
    }
}

impl IOBusConnection for V30MZ {
    fn read_io(&mut self, addr: u16) -> u8 {
        self.io_bus.borrow_mut().read_io(addr)
    }

    fn write_io(&mut self, addr: u16, byte: u8) {
        self.io_buffer.insert(addr, byte);
    }
}

impl V30MZ {
    pub fn new(mem_bus: Rc<RefCell<MemBus>>, io_bus: Rc<RefCell<IOBus>>) -> Self {
        Self {
            AW: 0, BW: 0, CW: 0, DW: 0,
            DS0: 0, DS1: 0, PS: 0, SS: 0,
            IX: 0, IY: 0,
            SP: 0, BP: 0, 
            PC: 0, pc_displacement: 0,
            
            PSW: CpuStatus::from_bits_truncate(0xF002),

            current_op: Vec::with_capacity(8),
            segment_override: None,
            halt: false, rep: false, rep_z: false,

            mem_bus, io_bus,
            mem_buffer: HashMap::new(),
            io_buffer: HashMap::new(),

            cycles: 0, base: 0,
        }
    }

    pub fn tick(&mut self) {
        if self.cycles == 0 {
            if !self.rep {self.poll_interrupts()};
            if !self.halt {self.execute()};
        } else {
            self.cycles -= 1;
            if self.cycles == 0 {self.commit_writes()};
        }
    }

    pub fn execute(&mut self) {
        // CPU requires at least one byte of instruction code to execute
        self.expect_op_bytes(1);

        let op = &CPU_OP_CODES[self.current_op[0] as usize];

        // println!("{:05X} {:02X} {}", self.get_pc_address(), op.code, op.name);

        if !((op.code >= 0xA4 && op.code <= 0xA7) || (op.code >= 0x6C && op.code <= 0x6F) || (op.code >= 0xAA && op.code <= 0xAF)) {
            self.rep = false;
        }

        self.base = op.cycles;
        self.cycles = self.base;

        // This will return OK only if there are no pending requests to SoC
        match op.code {
            // PREFIXES

            0x26 => {
                self.segment_override = Some(self.DS1);
                self.finish_prefix();
                return;
            }

            0x2E => {
                self.segment_override = Some(self.PS);
                self.finish_prefix();
                return;
            }

            0x36 => {
                self.segment_override = Some(self.SS);
                self.finish_prefix();
                return;
            }

            0x3E => {
                self.segment_override = Some(self.DS0);
                self.finish_prefix();
                return;
            }

            // BUSLOCK
            0xF0 => {
                self.mem_bus.borrow_mut().owner = Owner::CPU;
                self.finish_prefix();
                return;
            }

            // REPNE
            0xF2 => {
                self.rep = true;
                self.rep_z = false;
                self.finish_prefix();
            }

            // REP
            0xF3 => {
                self.rep = true;
                self.rep_z = true;
                self.finish_prefix();
                return;
            }

            // FULL INSTRUCTIONS

            // ADD
            0x00..=0x05 => self.add(op.op1, op.op2, op.mode, op.extra),

            // PUSH
            0x06 | 0x0E | 0x16 | 0x1E | 0x50..=0x57 | 0x68 | 0x6A | 0x9C => self.push_op(op.op2, op.extra),
            0x60 => self.push_r(),
            // POP
            0x07 | 0x17 | 0x1F | 0x58..=0x5F | 0x8F | 0x9D => self.pop_op(op.op2, op.extra),
            0x61 => self.pop_r(),

            // OR
            0x08..=0x0D => self.or(op.op1, op.op2, op.mode, op.extra),

            // ADDC
            0x10..=0x15 => self.addc(op.op1, op.op2, op.mode, op.extra),

            // SUBC
            0x18..=0x1D => self.subc(op.op1, op.op2, op.mode, op.extra),

            // AND
            0x20..=0x25 => self.and(op.op1, op.op2, op.mode, op.extra),

            // ADJ4A
            0x27 => self.adj4a(),

            // SUB
            0x28..=0x2D => self.sub(op.op1, op.op2, op.mode, op.extra),

            // ADJ4S
            0x2F => self.adj4s(),

            // XOR
            0x30..=0x35 => self.xor(op.op1, op.op2, op.mode, op.extra),

            // ADJBA
            0x37 => self.adjba(),

            // CMP
            0x38..=0x3D => self.cmp(op.op1, op.op2, op.mode, op.extra),

            // ADJBS
            0x3F => self.adjbs(),

            // INC
            0x40..=0x47 => self.inc(op.op1, op.mode, op.extra),

            // DEC
            0x48..=0x4F => self.dec(op.op1, op.mode, op.extra),

            // CHKIND
            0x62 => self.chkind(op.extra),

            // MUL
            0x69 | 0x6B => self.mul(op.op3, op.mode, op.extra),

            // INM
            0x6C | 0x6D => self.inm(op.mode, op.cycles, op.extra),

            // OUTM
            0x6E | 0x6F => self.outm(op.mode, op.cycles, op.extra),

            // Branch ops
            0x70 => self.branch(self.PSW.contains(CpuStatus::OVERFLOW)),
            0x71 => self.branch(!self.PSW.contains(CpuStatus::OVERFLOW)),
            0x72 => self.branch(self.PSW.contains(CpuStatus::CARRY)),
            0x73 => self.branch(!self.PSW.contains(CpuStatus::CARRY)),
            0x74 => self.branch(self.PSW.contains(CpuStatus::ZERO)),
            0x75 => self.branch(!self.PSW.contains(CpuStatus::ZERO)),
            0x76 => self.branch(self.PSW.contains(CpuStatus::ZERO) || self.PSW.contains(CpuStatus::ZERO)),
            0x77 => self.branch(!(self.PSW.contains(CpuStatus::ZERO) || self.PSW.contains(CpuStatus::ZERO))),
            0x78 => self.branch(self.PSW.contains(CpuStatus::SIGN)),
            0x79 => self.branch(!self.PSW.contains(CpuStatus::SIGN)),
            0x7A => self.branch(self.PSW.contains(CpuStatus::PARITY)),
            0x7B => self.branch(!self.PSW.contains(CpuStatus::PARITY)),
            0x7C => self.branch(self.PSW.contains(CpuStatus::SIGN) ^ self.PSW.contains(CpuStatus::OVERFLOW)),
            0x7D => self.branch(!(self.PSW.contains(CpuStatus::SIGN) ^ self.PSW.contains(CpuStatus::OVERFLOW))),
            0x7E => self.branch((self.PSW.contains(CpuStatus::SIGN) ^ self.PSW.contains(CpuStatus::OVERFLOW)) || self.PSW.contains(CpuStatus::ZERO)),
            0x7F => self.branch(!((self.PSW.contains(CpuStatus::SIGN) ^ self.PSW.contains(CpuStatus::OVERFLOW)) || self.PSW.contains(CpuStatus::ZERO))),

            0xE0 => {
                self.CW = self.CW.wrapping_sub(1);
                self.branch(self.CW != 0 && !self.PSW.contains(CpuStatus::ZERO));
            }
            0xE1 => {
                self.CW = self.CW.wrapping_sub(1);
                self.branch(self.CW != 0 && self.PSW.contains(CpuStatus::ZERO));
            }
            0xE2 => {
                self.CW = self.CW.wrapping_sub(1);
                self.branch(self.CW != 0);
            }
            0xE3 => self.branch(self.CW == 0),

            // Immediate Group
            0x80..=0x83 => {
                self.expect_op_bytes(2);
                let sub_op = &IMMEDIATE_GROUP[(self.current_op[1] & 0b0011_1000) as usize >> 3];
                self.base = sub_op.cycles;
                self.cycles = self.base;
                match sub_op.code {
                    0 => self.add(op.op1, op.op2, op.mode, sub_op.extra),
                    1 => self.or(op.op1, op.op2, op.mode, sub_op.extra),
                    2 => self.addc(op.op1, op.op2, op.mode, sub_op.extra),
                    3 => self.subc(op.op1, op.op2, op.mode, sub_op.extra),
                    4 => self.and(op.op1, op.op2, op.mode, sub_op.extra),
                    5 => self.sub(op.op1, op.op2, op.mode, sub_op.extra),
                    6 => self.xor(op.op1, op.op2, op.mode, sub_op.extra),
                    7 => self.cmp(op.op1, op.op2, op.mode, sub_op.extra),
                    _ => unreachable!(),
                }
            }

            // TEST
            0x84 | 0x85 | 0xA8 | 0xA9 => self.test(op.op1, op.op2, op.mode, op.extra),

            // XCH
            0x86 | 0x87 | 0x91..=0x97 => self.xch(op.mode, op.op1, op.op2, op.extra),

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
            0x88..=0x8C | 0x8E | 0xA0..=0xA3 | 0xB0..=0xBF | 0xC4..=0xC7 => self.mov(op, op.extra),

            // LDEA
            0x8D => self.ldea(op.extra),

            // NOP
            0x90 => {}

            // CALL
            0x9A | 0x9B | 0xE8 => self.call(op.op1, op.mode, op.extra),

            // CVTBW
            0x98 => self.cvtbw(),

            // CVTWL
            0x99 => self.cvtwl(),

            // MOVBK
            0xA4 | 0xA5 => self.movbk(op.mode, op.cycles, op.extra),

            // CMPBK
            0xA6 | 0xA7 => self.cmpbk(op.mode, op.cycles, op.extra),

            // STM
            0xAA | 0xAB => self.stm(op.mode, op.cycles, op.extra),

            // LDM
            0xAC | 0xAD => self.ldm(op.mode, op.cycles, op.extra),

            // CMPM
            0xAE | 0xAF => self.cmpm(op.mode, op.cycles, op.extra),

            // Shift Group
            0xC0 | 0xC1 | 0xD0..=0xD3 => {
                self.expect_op_bytes(2);
                let sub_op = &SHIFT_GROUP[(self.current_op[1] & 0b0011_1000) as usize >> 3];
                match sub_op.code {
                    0 => self.rol(op.code, op.mode, op.extra),
                    1 => self.ror(op.code, op.mode, op.extra),
                    2 => self.rolc(op.code, op.mode, op.extra),
                    3 => self.rorc(op.code, op.mode, op.extra),
                    4 => self.shl(op.code, op.mode, op.extra),
                    5 => self.shr(op.code, op.mode, op.extra),
                    6 => todo!(),
                    7 => self.shra(op.code, op.mode, op.extra),
                    _ => unreachable!()
                }
            }

            // RETN
            0xC2 | 0xC3 => self.retn(op.op2),

            // PREPARE
            0xC8 => self.prepare(),

            // DISPOSE
            0xC9 => self.dispose(),

            // RETF
            0xCA | 0xCB => self.retf(op.op2),

            // BRK
            0xCC | 0xCD => self.brk(op.op2),

            // BRKV
            0xCE => self.brkv(),

            // RETI
            0xCF => self.reti(),

            // CVTBD
            0xD4 => self.cvtbd(),

            // CVTDB
            0xD5 => self.cvtdb(),

            // SALC
            0xD6 => self.salc(),

            // TRANS
            0xD7 => self.trans(),

            // FPO1
            0xD8..=0xDF => self.fpo1(),

            // IN
            0xE4 | 0xE5 | 0xEC | 0xED => self.in_op(op.mode, op.op2),

            // OUT
            0xE6 | 0xE7 | 0xEE | 0xEF => self.out_op(op.mode, op.op1),

            // BR
            0xE9..=0xEB => self.branch_op(op.op1, op.mode, op.extra),

            // HALT
            0xF4 => self.halt = true,

            // NOT1
            0xF5 => self.PSW.toggle(CpuStatus::CARRY),

            // Group 1
            0xF6 | 0xF7 => {
                self.expect_op_bytes(2);
                let sub_op = &GROUP_1[(self.current_op[1] & 0b0011_1000) as usize >> 3];
                self.base = sub_op.cycles;
                self.cycles = self.base;
                match sub_op.code {
                    0 => self.test(op.op1, op.op2, op.mode, sub_op.extra),
                    1 => todo!(),
                    2 => self.not(op.mode, sub_op.extra),
                    3 => self.neg(op.mode, sub_op.extra),
                    4 => self.mulu(op.mode, sub_op.extra),
                    5 => self.mul(op.op3, op.mode, sub_op.extra),
                    6 | 7 => match (op.code, sub_op.code) {
                        (0xF6, 6) => {
                            self.base = 15;
                            self.cycles = self.base;
                            self.divu(op.mode, 1);
                        }
                        (0xF7, 6) => {
                            self.base = 23;
                            self.cycles = self.base;
                            self.divu(op.mode, 1);
                        }
                        (0xF6, 7) => {
                            self.base = 17;
                            self.cycles = self.base;
                            self.div(op.mode, 1);
                        }
                        (0xF7, 7) => {
                            self.base = 24;
                            self.cycles = self.base;
                            self.div(op.mode, 1);
                        }
                        _ => unreachable!(),
                    }
                    _ => unreachable!(),
                }
            }

            // CLR1
            0xF8 => self.PSW.remove(CpuStatus::CARRY),
            0xFC => self.PSW.remove(CpuStatus::DIRECTION),

            // SET1
            0xF9 => self.PSW.insert(CpuStatus::CARRY),
            0xFD => self.PSW.insert(CpuStatus::DIRECTION),

            // DI
            0xFA => self.PSW.remove(CpuStatus::INTERRUPT),

            // EI
            0xFB => self.PSW.insert(CpuStatus::INTERRUPT),

            // Group 2
            0xFE | 0xFF => {
                self.expect_op_bytes(2);
                let sub_op = &GROUP_2[(self.current_op[1] & 0b0011_1000) as usize >> 3];
                self.base = sub_op.cycles;
                self.cycles = self.base;
                match sub_op.code {
                    0 => self.inc(op.op1, op.mode, sub_op.extra),
                    1 => self.dec(op.op1, op.mode, sub_op.extra),
                    2 => self.call(op.op1, op.mode, sub_op.extra),
                    3 => self.call(op.op1, op.mode, sub_op.extra),
                    4 => self.branch_op(op.op1, op.mode, sub_op.extra),
                    5 => self.branch_op(op.op1, op.mode, sub_op.extra),
                    6 => self.push_op(Operand::MEMORY, sub_op.extra),
                    7 => todo!(),
                    _ => unreachable!()
                }
            }

            // NOP
            0x0F | 0x63..=0x67 => {}
                
            code => panic!("Not yet implemented! Code: {:02X}", code),
        };

        self.finish_op();
    }

    pub fn reset(&mut self) {
        self.DS0 = 0x0000;
        self.DS1 = 0x0000;
        self.PC = 0x0000;
        self.PS = 0xFFFF;
        self.PSW = CpuStatus::from_bits_truncate(0xF002);
        self.SS = 0x0000;
    }

    pub fn get_pc_address(&mut self) -> u32 {
        self.apply_segment(self.PC, self.PS)
    }

    fn finish_op(&mut self) {
        if self.rep {
            self.CW = self.CW.wrapping_sub(1);
            self.pc_displacement = 0;
        }

        if !self.rep || self.CW == 0 {
            self.PC = self.PC.wrapping_add(self.pc_displacement);
            self.mem_bus.borrow_mut().owner = Owner::NONE;
            if self.PSW.contains(CpuStatus::BREAK) {self.raise_exception(1)}
        }

        self.current_op.clear();
        self.pc_displacement = 0;
        self.cycles -= 1;
        if self.cycles == 0 {
            self.commit_writes();
        }
    }

    fn finish_prefix(&mut self) {
        self.current_op.clear();
        self.PC = self.PC.wrapping_add(self.pc_displacement);
        self.pc_displacement = 0;
        self.cycles -= 1;
        if self.cycles == 0 {
            self.commit_writes();
        }
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

    fn poll_interrupts(&mut self) {
        let nmi = self.read_io(0xB7) != 0;
        let cause = self.read_io(0xB4);
        if (cause != 0 || nmi) && self.mem_bus.borrow().owner != Owner::CPU {
            self.halt = false;
            if (self.PSW.contains(CpuStatus::INTERRUPT)) || nmi {
                let source = cause.trailing_zeros() as u8;
                let vector = self.read_io(0xB0).wrapping_add(source);
                self.raise_exception(vector);
            }
        }
    }

    fn commit_writes(&mut self) {
        for (addr, byte) in &self.mem_buffer {
            self.mem_bus.borrow_mut().write_mem(*addr, *byte);
        }
        for (addr, byte) in &self.io_buffer {
            self.io_bus.borrow_mut().write_io(*addr, *byte);
        }
        self.mem_buffer.clear();
        self.io_buffer.clear();
    }

    #[cfg(test)]
    pub fn tick_ignore_cycles(&mut self) {
        if !self.rep {self.poll_interrupts()};
        if !self.halt {self.execute()};
        self.commit_writes();
    }
}