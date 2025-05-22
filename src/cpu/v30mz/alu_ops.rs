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

                let carry = self.PSW.contains(CpuStatus::CARRY) as u16;

                let result = old_dest.wrapping_add(src).wrapping_add(carry);

                self.update_flags_8(old_dest, src, result);

                self.write_src_to_dest_8(op1, result as u8)
            }
            Mode::M16 => {
                let old_dest = self.resolve_src_16(op1)? as u32;
                let src = self.resolve_src_16(op2)? as u32;

                let carry = self.PSW.contains(CpuStatus::CARRY) as u32;

                let result = old_dest.wrapping_add(src).wrapping_add(carry);

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

        let carry = self.PSW.contains(CpuStatus::CARRY) as u32;

        let result = old_dest.wrapping_add(src).wrapping_add(carry);

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