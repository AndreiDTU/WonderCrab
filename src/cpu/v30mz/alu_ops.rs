use crate::cpu::parity;

use super::*;

impl V30MZ {
    pub fn add(&mut self, op1: Operand, op2: Operand, mode: Mode) -> Result<(), ()> {
        // Adds the two operands. The result is stored in the left operand.
        match mode {
            Mode::M8 => {
                let old_dest = self.resolve_src_8(op1)? as u16;
                let src = self.resolve_src_8(op2)? as u16;

                let result = old_dest.wrapping_add(src);

                self.update_flags_8(old_dest, src, result);

                self.write_src_to_dest_8(op1, result as u8)
            }
            Mode::M16 => {
                let old_dest = self.resolve_src_16(op1)? as u32;
                let src = self.resolve_src_16(op2)? as u32;

                let result = old_dest.wrapping_add(src);

                self.update_flags_16(old_dest, src, result);

                self.write_src_to_dest_16(op1, result as u16)
            }
            Mode::M32 => unreachable!(),
        }
    }

    pub fn add_s(&mut self) -> Result<(), ()> {
        self.expect_op_bytes(2)?;
        let old_dest = self.resolve_mem_src_16(self.current_op[2])? as u32;
        let src = self.expect_extra_byte() as u32;
        
        let result = old_dest.wrapping_add(src);

        self.update_flags_16(old_dest, src, result);

        self.write_src_to_dest_16(Operand::MEMORY, result as u16)
    }
    
    pub fn addc(&mut self, op1: Operand, op2: Operand, mode: Mode) -> Result<(), ()> {
        // Adds the two operands, plus 1 more if the carry flag (CY) was set.
        // The result is stored in the left operand. 
        match mode {
            Mode::M8 => {
                let old_dest = self.resolve_src_8(op1)? as u16;
                let src = self.resolve_src_8(op2)? as u16;

                let result = old_dest.wrapping_add(src).wrapping_add(1);

                self.update_flags_8(old_dest, src, result);

                self.write_src_to_dest_8(op1, result as u8)
            }
            Mode::M16 => {
                let old_dest = self.resolve_src_16(op1)? as u32;
                let src = self.resolve_src_16(op2)? as u32;

                let result = old_dest.wrapping_add(src);

                self.update_flags_16(old_dest, src, result);

                self.write_src_to_dest_16(op1, result as u16)
            }
            Mode::M32 => unreachable!(),
        }
    }

    pub fn addc_s(&mut self) -> Result<(), ()> {
        self.expect_op_bytes(2)?;
        let old_dest = self.resolve_mem_src_16(self.current_op[2])? as u32;
        let src = self.expect_extra_byte() as u32;
        
        let result = old_dest.wrapping_add(src).wrapping_add(1);

        self.update_flags_16(old_dest, src, result);

        self.write_src_to_dest_16(Operand::MEMORY, result as u16)
    }

    pub fn adj4a(&mut self) {
        let mut AL = self.AW as u8;
        if AL & 0x0F > 0x09 || self.PSW.contains(CpuStatus::AUX_CARRY) {
            AL = AL.wrapping_add(0x06);
            self.AW = swap_l(self.AW, AL);
            self.PSW.insert(CpuStatus::AUX_CARRY);
        }
        if AL > 0x9F || self.PSW.contains(CpuStatus::CARRY) {
            AL = AL.wrapping_add(0x60);
            self.AW = swap_l(self.AW, AL);
            self.PSW.insert(CpuStatus::CARRY);
        }

        self.PSW.set(CpuStatus::ZERO, AL == 0);
        self.PSW.set(CpuStatus::SIGN, AL & 0x80 != 0);
        self.PSW.set(CpuStatus::PARITY, parity(AL));
    }

    pub fn adj4s(&mut self) {
        let mut AL = self.AW as u8;
        if AL & 0x0F > 0x09 || self.PSW.contains(CpuStatus::AUX_CARRY) {
            AL = AL.wrapping_sub(0x06);
            self.AW = swap_l(self.AW, AL);
            self.PSW.insert(CpuStatus::AUX_CARRY);
        }
        if AL > 0x9F || self.PSW.contains(CpuStatus::CARRY) {
            AL = AL.wrapping_sub(0x60);
            self.AW = swap_l(self.AW, AL);
            self.PSW.insert(CpuStatus::CARRY);
        }

        self.PSW.set(CpuStatus::ZERO, AL == 0);
        self.PSW.set(CpuStatus::SIGN, AL & 0x80 != 0);
        self.PSW.set(CpuStatus::PARITY, parity(AL));
    }
}