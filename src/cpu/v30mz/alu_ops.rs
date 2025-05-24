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

                self.update_flags_add_8(old_dest, src, result);

                self.write_src_to_dest_8(op1, result as u8)
            }
            Mode::M16 => {
                let old_dest = self.resolve_src_16(op1)? as u32;
                let src = self.resolve_src_16(op2)? as u32;

                let result = old_dest.wrapping_add(src);

                self.update_flags_add_16(old_dest, src, result);

                self.write_src_to_dest_16(op1, result as u16)
            }
            Mode::M32 => unreachable!(),
        }
    }
    
    pub fn addc(&mut self, op1: Operand, op2: Operand, mode: Mode) -> Result<(), ()> {
        // Adds the two operands, plus 1 more if the carry flag (CY) was set.
        // The result is stored in the left operand. 
        match mode {
            Mode::M8 => {
                let old_dest = self.resolve_src_8(op1)? as u16;
                let src = self.resolve_src_8(op2)? as u16;

                let carry = self.PSW.contains(CpuStatus::CARRY) as u16;

                let result = old_dest.wrapping_add(src).wrapping_add(carry);

                self.update_flags_add_8(old_dest, src, result);

                self.write_src_to_dest_8(op1, result as u8)
            }
            Mode::M16 => {
                let old_dest = self.resolve_src_16(op1)? as u32;
                let src = self.resolve_src_16(op2)? as u32;

                let carry = self.PSW.contains(CpuStatus::CARRY) as u32;

                let result = old_dest.wrapping_add(src).wrapping_add(carry);

                self.update_flags_add_16(old_dest, src, result);

                self.write_src_to_dest_16(op1, result as u16)
            }
            Mode::M32 => unreachable!(),
        }
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
        self.AW = swap_l(self.AW, AL);
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
        self.AW = swap_l(self.AW, AL);
    }

    pub fn adjba(&mut self) {
        let mut AL = self.AW as u8;
        if AL & 0x0F > 0x0F || self.PSW.contains(CpuStatus::AUX_CARRY) {
            AL = AL.wrapping_add(0x06) & 0x0F;
            self.AW = self.AW.wrapping_add(0x0100);
            self.AW = swap_l(self.AW, AL);
            self.PSW.insert(CpuStatus::AUX_CARRY);
            self.PSW.insert(CpuStatus::CARRY);
            self.PSW.remove(CpuStatus::SIGN);
            self.PSW.insert(CpuStatus::ZERO);
        } else {
            AL &= 0x0F;
            self.PSW.remove(CpuStatus::AUX_CARRY);
            self.PSW.remove(CpuStatus::CARRY);
            self.PSW.insert(CpuStatus::SIGN);
            self.PSW.remove(CpuStatus::ZERO);
        }
        self.PSW.remove(CpuStatus::OVERFLOW);
        self.PSW.insert(CpuStatus::PARITY);
        self.AW = swap_l(self.AW, AL);
    }

    pub fn adjbs(&mut self) {
        let mut AL = self.AW as u8;
        if AL & 0x0F > 0x0F || self.PSW.contains(CpuStatus::AUX_CARRY) {
            AL = AL.wrapping_sub(0x06) & 0x0F;
            self.AW = swap_h(self.AW, ((self.AW >> 8) as u8).wrapping_sub(1));
            self.AW = swap_l(self.AW, AL);
            self.PSW.insert(CpuStatus::AUX_CARRY);
            self.PSW.insert(CpuStatus::CARRY);
            self.PSW.remove(CpuStatus::SIGN);
            self.PSW.insert(CpuStatus::ZERO);
        } else {
            AL &= 0x0F;
            self.PSW.remove(CpuStatus::AUX_CARRY);
            self.PSW.remove(CpuStatus::CARRY);
            self.PSW.insert(CpuStatus::SIGN);
            self.PSW.remove(CpuStatus::ZERO);
        }
        self.PSW.remove(CpuStatus::OVERFLOW);
        self.PSW.insert(CpuStatus::PARITY);
        self.AW = swap_l(self.AW, AL);
    }

    pub fn cmp(&mut self, op1: Operand, op2: Operand, mode: Mode) -> Result<(), ()> {
        match mode {
            Mode::M8 => {
                let old_dest = self.resolve_src_8(op1)?;
                let src = self.resolve_src_8(op2)?;

                let result = old_dest.wrapping_sub(src);

                self.update_flags_sub_8(old_dest, src, result);
            }
            Mode::M16 => {
                let old_dest = self.resolve_src_16(op1)?;
                let src = self.resolve_src_16(op2)?;

                let result = old_dest.wrapping_sub(src);

                self.update_flags_sub_16(old_dest, src, result);
            }
            Mode::M32 => unreachable!(),
        }

        Ok(())
    }

    pub fn dec(&mut self, op: Operand, mode: Mode) -> Result<(), ()> {
        match op {
            Operand::REGISTER => {
                let bits = self.current_op[0] & 0b111;
                let RegisterType::RW(r) = self.resolve_register_operand(bits, Mode::M16) else {unreachable!()};
                let (o, a) = (*r == 0xFFFF, *r & 0xF == 0);
                *r = r.wrapping_sub(1);
                let (z, s) = (*r == 0, *r & 0x8000 == 1);
                self.PSW.set(CpuStatus::OVERFLOW, o);
                self.PSW.set(CpuStatus::AUX_CARRY, a);
                self.PSW.set(CpuStatus::ZERO, z);
                self.PSW.set(CpuStatus::SIGN, s);
            }
            Operand::MEMORY => {
                match mode {
                    Mode::M8 => {
                        self.expect_op_bytes(1)?;
                        let src = self.resolve_mem_src_8(self.current_op[1])?;
                        self.PSW.set(CpuStatus::OVERFLOW, src == 0xFF);
                        self.PSW.set(CpuStatus::AUX_CARRY, src & 0xF == 0);
                        let res = src.wrapping_sub(1);
                        self.PSW.set(CpuStatus::ZERO, src == 0);
                        self.PSW.set(CpuStatus::SIGN, src & 0x80 == 1);
                        self.write_src_to_dest_8(Operand::MEMORY, res)?;
                    }
                    Mode::M16 => {
                        self.expect_op_bytes(1)?;
                        let src = self.resolve_mem_src_16(self.current_op[1])?;
                        self.PSW.set(CpuStatus::OVERFLOW, src == 0xFFFF);
                        self.PSW.set(CpuStatus::AUX_CARRY, src & 0xF == 0);
                        let res = src.wrapping_sub(1);
                        self.PSW.set(CpuStatus::ZERO, src == 0);
                        self.PSW.set(CpuStatus::SIGN, src & 0x8000 == 1);
                        self.write_src_to_dest_16(Operand::MEMORY, res)?;
                    }
                    _ => unreachable!(),
                }
            }
            _ => unreachable!()
        }

        Ok(())
    }

    pub fn cvtbd(&mut self) -> Result<(), ()> {
        self.expect_op_bytes(2)?;
        let src = self.current_op[1];
        if src == 0 {
            return self.raise_exception(0);
        }
        let (AL, AH) = (self.AW as u8 / src, self.AW as u8 % src);
        self.PSW.set(CpuStatus::ZERO, AL == 0);
        self.PSW.set(CpuStatus::SIGN, AL & 0x80 != 0);
        self.PSW.set(CpuStatus::PARITY, parity(AL));
        self.AW = swap_h(self.AW, AH);
        self.AW = swap_l(self.AW, AL);

        Ok(())
    }

    pub fn cvtdb(&mut self) -> Result<(), ()> {
        self.expect_op_bytes(2)?;
        let src = self.current_op[1] as u16;

        let (AH, AL) = ((self.AW >> 8) as u8, self.AW as u8);

        let result = AH as u16 * src + AL as u16;

        self.AW = swap_l(self.AW, result as u8);
        self.AW &= 0xFF;
        self.update_flags_add_8(AL as u16, AH as u16, result);

        Ok(())
    }

    pub fn sub(&mut self, op1: Operand, op2: Operand, mode: Mode) -> Result<(), ()> {
        // Subtracts the two operands. The result is stored in the left operand.
        match mode {
            Mode::M8 => {
                let old_dest = self.resolve_src_8(op1)?;
                let src = self.resolve_src_8(op2)?;

                let result = old_dest.wrapping_sub(src);

                self.update_flags_sub_8(old_dest, src, result);
                self.write_src_to_dest_8(op1, result)
            }
            Mode::M16 => {
                let old_dest = self.resolve_src_16(op1)?;
                let src = self.resolve_src_16(op2)?;

                let result = old_dest.wrapping_sub(src);

                self.update_flags_sub_16(old_dest, src, result);
                self.write_src_to_dest_16(op1, result)
            }
            Mode::M32 => unreachable!(),
        }
    }

    pub fn subc(&mut self, op1: Operand, op2: Operand, mode: Mode) -> Result<(), ()> {
        // Subtracts the two operands. The result is stored in the left operand.
        match mode {
            Mode::M8 => {
                let old_dest = self.resolve_src_8(op1)?;
                let src = self.resolve_src_8(op2)?;

                let carry = self.PSW.contains(CpuStatus::CARRY) as u8;

                let result = old_dest.wrapping_sub(src).wrapping_add(carry);

                self.update_flags_sub_8(old_dest, src, result);
                self.write_src_to_dest_8(op1, result)
            }
            Mode::M16 => {
                let old_dest = self.resolve_src_16(op1)?;
                let src = self.resolve_src_16(op2)?;

                let carry = self.PSW.contains(CpuStatus::CARRY) as u16;

                let result = old_dest.wrapping_sub(src).wrapping_add(carry);

                self.update_flags_sub_16(old_dest, src, result);
                self.write_src_to_dest_16(op1, result)
            }
            Mode::M32 => unreachable!(),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::soc::SoC;
    use crate::assert_eq_hex;

    use super::*;

    #[test]
    fn test_0x00_add_register_to_memory_8() {
        let mut soc = SoC::new();
        soc.set_wram(vec![
            0x00, 0x06, 0xFE, 0x00, // [0x00FE] <- [0x00FE] + AL
        ]);
        soc.get_cpu().AW = 0x1234;
        soc.get_wram()[0x00FE] = 0x01;

        soc.tick();
        assert_eq_hex!(soc.get_cpu().PC, 0x0004);
        assert_eq_hex!(soc.get_wram()[0x00FE], 0x35);
        assert!(soc.get_cpu().PSW.contains(CpuStatus::PARITY));
    }

    #[test]
    fn test_0x01_add_register_to_memory_16() {
        let mut soc = SoC::new();
        soc.set_wram(vec![
            0x01, 0x06, 0xFE, 0x00, // [0x00FE] <- [0x00FE] + AW
        ]);
        soc.get_cpu().AW = 0x1234;
        soc.get_wram()[0x00FE] = 0xFF;
        soc.get_wram()[0x00FF] = 0x01;

        soc.tick();
        assert_eq_hex!(soc.get_cpu().PC, 0x0004);
        assert_eq_hex!(soc.get_wram()[0x00FE], 0x33);
        assert_eq_hex!(soc.get_wram()[0x00FF], 0x14);
        assert!(soc.get_cpu().PSW.contains(CpuStatus::AUX_CARRY));
        assert!(soc.get_cpu().PSW.contains(CpuStatus::PARITY));
    }

    #[test]
    fn test_0x02_add_memory_to_register_8() {
        let mut soc = SoC::new();
        soc.set_wram(vec![
            0x02, 0x06, 0xFE, 0x00,
        ]);
        soc.get_cpu().AW = 0x1234;
        soc.get_wram()[0x00FE] = 0x01;

        soc.tick();
        assert_eq_hex!(soc.get_cpu().PC, 0x0004);
        assert_eq_hex!(soc.get_cpu().AW, 0x1235);
        assert!(!soc.get_cpu().PSW.contains(CpuStatus::AUX_CARRY));
        assert!(soc.get_cpu().PSW.contains(CpuStatus::PARITY));
    }

    #[test]
    fn test_0x03_add_memory_to_register_16() {
        let mut soc = SoC::new();
        soc.set_wram(vec![
            0x03, 0x06, 0xFE, 0x00,
        ]);
        soc.get_cpu().AW = 0x1234;
        soc.get_wram()[0x00FE] = 0xFF;
        soc.get_wram()[0x00FF] = 0x01;

        soc.tick();
        assert_eq_hex!(soc.get_cpu().PC, 0x0004);
        assert_eq_hex!(soc.get_cpu().AW, 0x1433);
        assert!(soc.get_cpu().PSW.contains(CpuStatus::AUX_CARRY));
        assert!(soc.get_cpu().PSW.contains(CpuStatus::PARITY));
    }

    #[test]
    fn test_0x04_add_immediate_to_accumulator_8() {
        let mut soc = SoC::new();
        soc.set_wram(vec![
            0x04, 0xFF,
        ]);
        soc.get_cpu().AW = 0x12FF;

        soc.tick();
        assert_eq_hex!(soc.get_cpu().PC, 0x0002);
        assert_eq_hex!(soc.get_cpu().AW, 0x12FE);
        assert!(soc.get_cpu().PSW.contains(CpuStatus::AUX_CARRY));
        assert!(soc.get_cpu().PSW.contains(CpuStatus::SIGN));
        assert!(soc.get_cpu().PSW.contains(CpuStatus::CARRY));
        assert!(!soc.get_cpu().PSW.contains(CpuStatus::OVERFLOW));
    }

    #[test]
    fn test_0x05_add_immediate_to_accumulator_16() {
        let mut soc = SoC::new();
        soc.set_wram(vec![
            0x05, 0xFF, 0xFF
        ]);
        soc.get_cpu().AW = 0x12FF;

        soc.tick();
        assert_eq_hex!(soc.get_cpu().PC, 0x0003);
        assert_eq_hex!(soc.get_cpu().AW, 0x12FE);
        assert!(soc.get_cpu().PSW.contains(CpuStatus::AUX_CARRY));
        assert!(!soc.get_cpu().PSW.contains(CpuStatus::SIGN));
        assert!(soc.get_cpu().PSW.contains(CpuStatus::CARRY));
        assert!(!soc.get_cpu().PSW.contains(CpuStatus::OVERFLOW));
    }

    #[test]
    fn test_0x10_addc_register_to_memory_8() {
        let mut soc = SoC::new();
        soc.set_wram(vec![
            0x10, 0x06, 0xFE, 0x00, // [0x00FE] <- [0x00FE] + AL + carry
        ]);
        soc.get_cpu().AW = 0x1234;
        soc.get_cpu().PSW.insert(CpuStatus::CARRY);
        soc.get_wram()[0x00FE] = 0x01;

        soc.tick();
        assert_eq_hex!(soc.get_cpu().PC, 0x0004);
        assert_eq_hex!(soc.get_wram()[0x00FE], 0x36);
        assert!(soc.get_cpu().PSW.contains(CpuStatus::PARITY));
    }

    #[test]
    fn test_0x11_addc_register_to_memory_16() {
        let mut soc = SoC::new();
        soc.set_wram(vec![
            0x11, 0x06, 0xFE, 0x00, // [0x00FE] <- [0x00FE] + AW + carry
        ]);
        soc.get_cpu().AW = 0x1234;
        soc.get_cpu().PSW.insert(CpuStatus::CARRY);
        soc.get_wram()[0x00FE] = 0xFF;
        soc.get_wram()[0x00FF] = 0x01;

        soc.tick();
        assert_eq_hex!(soc.get_cpu().PC, 0x0004);
        assert_eq_hex!(soc.get_wram()[0x00FE], 0x34);
        assert_eq_hex!(soc.get_wram()[0x00FF], 0x14);
        assert!(soc.get_cpu().PSW.contains(CpuStatus::AUX_CARRY));
        assert!(!soc.get_cpu().PSW.contains(CpuStatus::PARITY));
    }

    #[test]
    fn test_0x12_addc_memory_to_register_8() {
        let mut soc = SoC::new();
        soc.set_wram(vec![
            0x12, 0x06, 0xFE, 0x00,
        ]);
        soc.get_cpu().AW = 0x1234;
        soc.get_cpu().PSW.insert(CpuStatus::CARRY);
        soc.get_wram()[0x00FE] = 0x01;

        soc.tick();
        assert_eq_hex!(soc.get_cpu().PC, 0x0004);
        assert_eq_hex!(soc.get_cpu().AW, 0x1236);
        assert!(!soc.get_cpu().PSW.contains(CpuStatus::AUX_CARRY));
        assert!(soc.get_cpu().PSW.contains(CpuStatus::PARITY));
    }

    #[test]
    fn test_0x13_addc_memory_to_register_16() {
        let mut soc = SoC::new();
        soc.set_wram(vec![
            0x13, 0x06, 0xFE, 0x00,
        ]);
        soc.get_cpu().AW = 0x1234;
        soc.get_cpu().PSW.insert(CpuStatus::CARRY);
        soc.get_wram()[0x00FE] = 0xFF;
        soc.get_wram()[0x00FF] = 0x01;

        soc.tick();
        assert_eq_hex!(soc.get_cpu().PC, 0x0004);
        assert_eq_hex!(soc.get_cpu().AW, 0x1434);
        assert!(soc.get_cpu().PSW.contains(CpuStatus::AUX_CARRY));
        assert!(!soc.get_cpu().PSW.contains(CpuStatus::PARITY));
    }

    #[test]
    fn test_0x14_addc_immediate_to_accumulator_8() {
        let mut soc = SoC::new();
        soc.set_wram(vec![
            0x14, 0xFF,
        ]);
        soc.get_cpu().AW = 0x12FF;
        soc.get_cpu().PSW.insert(CpuStatus::CARRY);

        soc.tick();
        assert_eq_hex!(soc.get_cpu().PC, 0x0002);
        assert_eq_hex!(soc.get_cpu().AW, 0x12FF); // 0xFF + 0xFF + 1 = 0x1FF
        assert!(soc.get_cpu().PSW.contains(CpuStatus::AUX_CARRY));
        assert!(soc.get_cpu().PSW.contains(CpuStatus::CARRY));
        assert!(soc.get_cpu().PSW.contains(CpuStatus::SIGN));
    }

    #[test]
    fn test_0x15_addc_immediate_to_accumulator_16() {
        let mut soc = SoC::new();
        soc.set_wram(vec![
            0x15, 0xFF, 0xFF
        ]);
        soc.get_cpu().AW = 0x12FF;
        soc.get_cpu().PSW.insert(CpuStatus::CARRY);

        soc.tick();
        assert_eq_hex!(soc.get_cpu().PC, 0x0003);
        assert_eq_hex!(soc.get_cpu().AW, 0x12FF); // 0x12FF + 0xFFFF + 1 = 0x12FF
        assert!(soc.get_cpu().PSW.contains(CpuStatus::CARRY));
        assert!(soc.get_cpu().PSW.contains(CpuStatus::AUX_CARRY));
        assert!(!soc.get_cpu().PSW.contains(CpuStatus::SIGN));
    }
}