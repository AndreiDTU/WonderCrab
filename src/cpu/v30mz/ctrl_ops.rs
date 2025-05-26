use crate::{bus::mem_bus::MemBusConnection, cpu::{Mode, Operand}};

use super::{CpuStatus, V30MZ};

impl V30MZ {
    pub fn branch_op(&mut self, op: Operand, mode: Mode, extra: u8) {
        match op {
            Operand::IMMEDIATE => (self.PC, self.PS) = (self.expect_extra_word(), self.expect_extra_word()),
            Operand::IMMEDIATE_S => {
                match mode {
                    Mode::M8 => self.branch(true),
                    Mode::M16 => {
                        let displacement = self.expect_extra_word() as i16;
                        self.PC = self.PC.wrapping_add(self.pc_displacement);
                        self.pc_displacement = 0;
                        self.PC = (self.PC as i16).wrapping_add(displacement) as u16;
                    }
                    _ => unreachable!()
                }
            }
            Operand::MEMORY => {
                match mode {
                    Mode::M16 => {
                        self.expect_op_bytes(2);
                        self.PC = self.resolve_mem_src_16(self.current_op[1], extra);
                    }
                    Mode::M32 => {
                        self.expect_op_bytes(2);
                        (self.PC, self.PS) = self.resolve_mem_src_32(self.current_op[1], extra);
                    }
                    _ => unreachable!(),
                }
            }
            _ => unreachable!()
        }
    }

    pub fn brk(&mut self, op: Operand) {
        let vector = match op {
            Operand::IMMEDIATE => self.expect_extra_byte(),
            Operand::NONE => 3,
            _ => unreachable!(),
        };

        self.raise_exception(vector);
    }

    pub fn brkv(&mut self) {
        if self.PSW.contains(CpuStatus::OVERFLOW) {self.raise_exception(4)}
    }

    pub fn call(&mut self, op: Operand, mode: Mode, extra: u8) {
        if mode == Mode::M32 {
            self.push(self.PS);
        }
        self.push(self.PC);
        
        self.branch_op(op, mode, extra);
    }

    pub fn chkind(&mut self, extra: u8) {
        self.expect_op_bytes(2);
        let reg = self.resolve_src_16(Operand::REGISTER, extra);
        let (lo, hi) = self.resolve_mem_src_32(self.current_op[1], extra);
        if !(reg >= lo && reg < hi) {self.raise_exception(5)}
    }

    pub fn dispose(&mut self) {
        self.SP = self.BP;
        self.BP = self.pop();
    }

    pub fn prepare(&mut self) {
        let (imm16, imm5) = (self.expect_extra_word(), self.expect_extra_byte() & 0x1F);
        self.push(self.BP);
        let temp = self.SP;
        if imm5 > 0 {
            for _ in 0..(imm5 - 1) {
                self.BP = self.BP.wrapping_sub(2);
                let addr = self.get_physical_address(self.BP, self.SS);
                let word = self.read_mem_16(addr);
                self.push(word);
            }
        }
        self.BP = temp;
        self.SP = self.SP.wrapping_sub(imm16);
    }

    pub fn retn(&mut self, op: Operand) {
        self.PC = self.pop();
        let dest = match op {
            Operand::IMMEDIATE => self.expect_extra_word(),
            Operand::NONE => 0,
            _ => unreachable!(),
        };
        self.SP = self.SP.wrapping_add(dest);
    }

    pub fn retf(&mut self, op: Operand) {
        self.PC = self.pop();
        self.PS = self.pop();
        let dest = match op {
            Operand::IMMEDIATE => self.expect_extra_word(),
            Operand::NONE => 0,
            _ => unreachable!(),
        };
        self.SP = self.SP.wrapping_add(dest);
    }

    pub fn reti(&mut self) {
        self.PC = self.pop();
        self.PS = self.pop();
        self.PSW = CpuStatus::from_bits_truncate(self.pop());
    }
}