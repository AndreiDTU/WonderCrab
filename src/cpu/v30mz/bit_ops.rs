use crate::cpu::parity;

use super::*;

impl V30MZ {
    pub fn and(&mut self, op1: Operand, op2: Operand, mode: Mode, extra: u8) {
        match mode {
            Mode::M8 => {
                let a = self.resolve_src_8(op1, extra);
                let b = if op2 == Operand::IMMEDIATE && op1 == Operand::MEMORY {
                    self.expect_extra_byte()
                } else {
                    self.resolve_src_8(op2, extra)
                };
                let res = a & b;

                self.write_src_to_dest_8(op1, res, extra);

                self.update_flags_bitwise_8(res);
            }
            Mode::M16 => {
                let a = self.resolve_src_16(op1, extra);
                let b = if op2 == Operand::IMMEDIATE_S {
                    self.expect_extra_byte() as i8 as i16 as u16
                } else if op2 == Operand::IMMEDIATE && op1 == Operand::MEMORY {
                    self.expect_extra_word()
                } else {
                    self.resolve_src_16(op2, extra)
                };
                let res = a & b;

                self.write_src_to_dest_16(op1, res, extra);

                self.update_flags_bitwise_16(res);
            }
            _ => unreachable!(),
        }
    }

    pub fn not(&mut self, mode: Mode, extra: u8) {
        match mode {
            Mode::M8 => {
                let src = self.resolve_src_8(Operand::MEMORY, extra);
                self.write_src_to_dest_8(Operand::MEMORY, !src, extra);
            }
            Mode::M16 => {
                let src = self.resolve_src_16(Operand::MEMORY, extra);
                self.write_src_to_dest_16(Operand::MEMORY, !src, extra);
            }
            Mode::M32 => unreachable!()
        }
    }

    pub fn or(&mut self, op1: Operand, op2: Operand, mode: Mode, extra: u8) {
        match mode {
            Mode::M8 => {
                let a = self.resolve_src_8(op1, extra);
                let b = if op2 == Operand::IMMEDIATE && op1 == Operand::MEMORY {
                    self.expect_extra_byte()
                } else {
                    self.resolve_src_8(op2, extra)
                };
                let res = a | b;

                self.update_flags_bitwise_8(res);

                self.write_src_to_dest_8(op1, res, extra);
            }
            Mode::M16 => {
                let a = self.resolve_src_16(op1, extra);
                let b = if op2 == Operand::IMMEDIATE_S {
                    self.expect_extra_byte() as i8 as i16 as u16
                } else if op2 == Operand::IMMEDIATE && op1 == Operand::MEMORY {
                    self.expect_extra_word()
                } else {
                    self.resolve_src_16(op2, extra)
                };
                let res = a | b;

                self.update_flags_bitwise_16(res);

                self.write_src_to_dest_16(op1, res, extra);
            }
            _ => unreachable!(),
        }
    }

    pub fn rol(&mut self, code: u8, mode: Mode, extra: u8) {
        let src = self.get_rot_src(code);

        match mode {
            Mode::M8 => {
                let dest = self.resolve_src_8(Operand::MEMORY, extra);
                let res = dest.rotate_left(src as u32);

                if src != 0 {
                    self.PSW.set(CpuStatus::CARRY, res & 1 != 0);
                }

                self.PSW.set(CpuStatus::OVERFLOW, (res >> 7 ^ self.PSW.contains(CpuStatus::CARRY) as u8) != 0);

                self.write_src_to_dest_8(Operand::MEMORY, res, extra);
            }
            Mode::M16 => {
                let dest = self.resolve_src_16(Operand::MEMORY, extra);
                let res = dest.rotate_left(src as u32);

                if src != 0 {
                    self.PSW.set(CpuStatus::CARRY, res & 1 != 0);
                }

                self.PSW.set(CpuStatus::OVERFLOW, (res >> 15 ^ self.PSW.contains(CpuStatus::CARRY) as u16) != 0);

                self.write_src_to_dest_16(Operand::MEMORY, res, extra);
            }
            _ => unreachable!()
        }
    }

    pub fn rolc(&mut self, code: u8, mode: Mode, extra: u8) {
        let src = self.get_rot_src(code);

        match mode {
            Mode::M8 => {
                let dest = self.resolve_src_8(Operand::MEMORY, extra);
                let mut res = dest;

                for _ in 0..src {
                    let old_carry = self.PSW.contains(CpuStatus::CARRY) as u8;
                    self.PSW.set(CpuStatus::CARRY, res & 0x80 != 0);
                    res = (res << 1) | old_carry;
                }

                self.PSW.set(CpuStatus::OVERFLOW, (res >> 7) & 1 != self.PSW.contains(CpuStatus::CARRY) as u8);

                self.write_src_to_dest_8(Operand::MEMORY, res, extra);
            }
            Mode::M16 => {
                let dest = self.resolve_src_16(Operand::MEMORY, extra);
                let mut res = dest;
                let mut old_msb = res >> 15;

                for _ in 0..src {
                    let old_carry = self.PSW.contains(CpuStatus::CARRY) as u16;
                    self.PSW.set(CpuStatus::CARRY, res & 0x8000 != 0);
                    old_msb = res >> 15;
                    res = (res << 1) | old_carry;
                }

                self.PSW.set(CpuStatus::OVERFLOW, (res >> 15) & 1 != self.PSW.contains(CpuStatus::CARRY) as u16);

                self.write_src_to_dest_16(Operand::MEMORY, res, extra);
            }
            _ => unreachable!()
        }
    }

    pub fn ror(&mut self, code: u8, mode: Mode, extra: u8) {
        let src = self.get_rot_src(code);

        match mode {
            Mode::M8 => {
                let dest = self.resolve_src_8(Operand::MEMORY, extra);
                let res = dest.rotate_right(src as u32);

                if src != 0 {
                    self.PSW.set(CpuStatus::CARRY, res & 0x80 != 0);
                }

                self.PSW.set(CpuStatus::OVERFLOW, ((res >> 6) ^ (res >> 7)) & 1 != 0);

                self.write_src_to_dest_8(Operand::MEMORY, res, extra);
            }
            Mode::M16 => {
                let dest = self.resolve_src_16(Operand::MEMORY, extra);
                let res = dest.rotate_right(src as u32);

                if src != 0 {
                    self.PSW.set(CpuStatus::CARRY, res & 0x8000 != 0);
                }

                self.PSW.set(CpuStatus::OVERFLOW, ((res >> 14) ^ (res >> 15)) & 1 != 0);

                self.write_src_to_dest_16(Operand::MEMORY, res, extra);
            }
            _ => unreachable!()
        }
    }

    pub fn rorc(&mut self, code: u8, mode: Mode, extra: u8) {
        let src = self.get_rot_src(code);

        match mode {
            Mode::M8 => {
                let dest = self.resolve_src_8(Operand::MEMORY, extra);
                let mut res = dest;

                for _ in 0..src {
                    let old_carry = self.PSW.contains(CpuStatus::CARRY) as u8;
                    self.PSW.set(CpuStatus::CARRY, res & 1 != 0);
                    res = (old_carry << 7) | (res >> 1);
                }

                self.PSW.set(CpuStatus::OVERFLOW, ((res >> 6) ^ (res >> 7)) & 1 != 0);

                self.write_src_to_dest_8(Operand::MEMORY, res, extra);
            }
            Mode::M16 => {
                let dest = self.resolve_src_16(Operand::MEMORY, extra);
                let mut res = dest;

                for _ in 0..src {
                    let old_carry = self.PSW.contains(CpuStatus::CARRY) as u16;
                    self.PSW.set(CpuStatus::CARRY, res & 1 != 0);
                    res = (old_carry << 15) | (res >> 1);
                }

                self.PSW.set(CpuStatus::OVERFLOW, ((res >> 14) ^ (res >> 15)) & 1 != 0);

                self.write_src_to_dest_16(Operand::MEMORY, res, extra);
            }
            _ => unreachable!()
        }
    }

    pub fn shl(&mut self, code: u8, mode: Mode, extra: u8) {
        let src = self.get_rot_src(code);

        match mode {
            Mode::M8 => {
                let dest = self.resolve_src_8(Operand::MEMORY, extra);
                let res = dest << src;

                if src != 0 {
                    self.PSW.set(CpuStatus::CARRY, dest << (src - 1) != 0);
                }

                self.PSW.set(CpuStatus::ZERO, res == 0);
                self.PSW.set(CpuStatus::SIGN, res & 0x80 != 0);
                self.PSW.set(CpuStatus::OVERFLOW, (res >> 7 ^ self.PSW.contains(CpuStatus::CARRY) as u8) != 0);
                self.PSW.set(CpuStatus::PARITY, parity(res));

                self.write_src_to_dest_8(Operand::MEMORY, res, extra);
            }
            Mode::M16 => {
                let dest = self.resolve_src_16(Operand::MEMORY, extra);
                let res = dest << src;

                if src != 0 {
                    self.PSW.set(CpuStatus::CARRY, dest << (src - 1) != 0);
                }

                self.PSW.set(CpuStatus::ZERO, res == 0);
                self.PSW.set(CpuStatus::SIGN, res & 0x8000 != 0);
                self.PSW.set(CpuStatus::OVERFLOW, (res >> 15 ^ self.PSW.contains(CpuStatus::CARRY) as u16) != 0);
                self.PSW.set(CpuStatus::PARITY, parity(res as u8));

                self.write_src_to_dest_16(Operand::MEMORY, res, extra);
            }
            _ => unreachable!()
        }

        self.PSW.remove(CpuStatus::AUX_CARRY);
    }

    pub fn shr(&mut self, code: u8, mode: Mode, extra: u8) {
        let src = self.get_rot_src(code);

        match mode {
            Mode::M8 => {
                let dest = self.resolve_src_8(Operand::MEMORY, extra);
                let res = dest >> src;

                self.PSW.set(CpuStatus::ZERO, res == 0);
                self.PSW.set(CpuStatus::SIGN, res & 0x80 != 0);
                self.PSW.set(CpuStatus::OVERFLOW, dest & 0x80 != res & 0x80);
                self.PSW.set(CpuStatus::CARRY, dest >> (src - 1) != 0);
                self.PSW.set(CpuStatus::PARITY, parity(res));

                self.write_src_to_dest_8(Operand::MEMORY, res, extra);
            }
            Mode::M16 => {
                let dest = self.resolve_src_16(Operand::MEMORY, extra);
                let res = dest >> src;

                self.PSW.set(CpuStatus::ZERO, res == 0);
                self.PSW.set(CpuStatus::SIGN, res & 0x8000 != 0);
                self.PSW.set(CpuStatus::OVERFLOW, dest & 0x8000 != res & 0x8000);
                self.PSW.set(CpuStatus::CARRY, dest >> (src - 1) != 0);
                self.PSW.set(CpuStatus::PARITY, parity(res as u8));

                self.write_src_to_dest_16(Operand::MEMORY, res, extra);
            }
            _ => unreachable!()
        }
    }

    pub fn shra(&mut self, code: u8, mode: Mode, extra: u8) {
        let src = self.get_rot_src(code);

        match mode {
            Mode::M8 => {
                let dest = self.resolve_src_8(Operand::MEMORY, extra);
                let res = (dest as i8 >> src) as u8;

                self.PSW.set(CpuStatus::ZERO, res == 0);
                self.PSW.set(CpuStatus::SIGN, res & 0x80 != 0);
                self.PSW.set(CpuStatus::OVERFLOW, dest & 0x80 != res & 0x80);
                self.PSW.set(CpuStatus::CARRY, dest >> (src - 1) != 0);
                self.PSW.set(CpuStatus::PARITY, parity(res));

                self.write_src_to_dest_8(Operand::MEMORY, res, extra);
            }
            Mode::M16 => {
                let dest = self.resolve_src_16(Operand::MEMORY, extra);
                let res = (dest as i16 >> src) as u16;

                self.PSW.set(CpuStatus::ZERO, res == 0);
                self.PSW.set(CpuStatus::SIGN, res & 0x8000 != 0);
                self.PSW.set(CpuStatus::OVERFLOW, dest & 0x8000 != res & 0x8000);
                self.PSW.set(CpuStatus::CARRY, dest >> (src - 1) != 0);
                self.PSW.set(CpuStatus::PARITY, parity(res as u8));

                self.write_src_to_dest_16(Operand::MEMORY, res, extra);
            }
            _ => unreachable!()
        }
    }

    pub fn test(&mut self, op1: Operand, op2: Operand, mode: Mode, extra: u8) {
        match mode {
            Mode::M8 => {
                let a = self.resolve_src_8(op1, extra);
                let b = if op2 == Operand::IMMEDIATE && op1 == Operand::MEMORY {
                    self.expect_extra_byte()
                } else {
                    self.resolve_src_8(op2, extra)
                };
                let res = a & b;

                self.update_flags_bitwise_8(res);
            }
            Mode::M16 => {
                let a = self.resolve_src_16(op1, extra);
                let b = if op2 == Operand::IMMEDIATE_S {
                    self.expect_extra_byte() as i8 as i16 as u16
                } else if op2 == Operand::IMMEDIATE && op1 == Operand::MEMORY {
                    self.expect_extra_word()
                } else {
                    self.resolve_src_16(op2, extra)
                };
                let res = a & b;

                self.update_flags_bitwise_16(res);
            }
            _ => unreachable!(),
        }
    }

    pub fn xor(&mut self, op1: Operand, op2: Operand, mode: Mode, extra: u8) {
        match mode {
            Mode::M8 => {
                let a = self.resolve_src_8(op1, extra);
                let b = if op2 == Operand::IMMEDIATE && op1 == Operand::MEMORY {
                    self.expect_extra_byte()
                } else {
                    self.resolve_src_8(op2, extra)
                };
                let res = a ^ b;

                self.update_flags_bitwise_8(res);

                self.write_src_to_dest_8(op1, res, extra);
            }
            Mode::M16 => {
                let a = self.resolve_src_16(op1, extra);
                let b = if op2 == Operand::IMMEDIATE_S {
                    self.expect_extra_byte() as i8 as i16 as u16
                } else if op2 == Operand::IMMEDIATE && op1 == Operand::MEMORY {
                    self.expect_extra_word()
                } else {
                    self.resolve_src_16(op2, extra)
                };
                let res = a ^ b;

                self.update_flags_bitwise_16(res);

                self.write_src_to_dest_16(op1, res, extra);
            }
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::cpu::v30mz::CpuStatus;
    use crate::soc::SoC;
    use crate::assert_eq_hex;
    
    #[test]
    fn test_0x08_or_memory_register_8() {
        let mut soc = SoC::test_build();
        soc.set_wram(vec![
            0x08, 0xC1, // CL <- AL & CL
        ]);
        soc.get_cpu().AW = 0xFE;
        soc.get_cpu().CW = 0xEF;

        soc.tick_cpu_no_cycles();
        assert_eq_hex!(soc.get_cpu().CW, 0xFF);
    }

    #[test]
    fn test_0x09_and_memory_register_16() {
        let mut soc = SoC::test_build();
        soc.set_wram(vec![
            0x09, 0xC1, // CW <- AW & CW
        ]);
        soc.get_cpu().AW = 0xFFEE;
        soc.get_cpu().CW = 0xEEFF;

        soc.tick_cpu_no_cycles();
        assert_eq_hex!(soc.get_cpu().CW, 0xFFFF);
    }
    
    #[test]
    fn test_0x20_and_memory_register_8() {
        let mut soc = SoC::test_build();
        soc.set_wram(vec![
            0x20, 0xC1, // CL <- AL & CL
        ]);
        soc.get_cpu().AW = 0xFE;
        soc.get_cpu().CW = 0xEF;

        soc.tick_cpu_no_cycles();
        assert_eq_hex!(soc.get_cpu().CW, 0xEE);
    }

    #[test]
    fn test_0x21_and_memory_register_16() {
        let mut soc = SoC::test_build();
        soc.set_wram(vec![
            0x21, 0xC1, // CW <- AW & CW
        ]);
        soc.get_cpu().AW = 0xFFEE;
        soc.get_cpu().CW = 0xEEFF;

        soc.tick_cpu_no_cycles();
        assert_eq_hex!(soc.get_cpu().CW, 0xEEEE);
    }

    #[test]
    fn test_0xc0_2_rolc_mem_imm_8() {
        let mut soc = SoC::test_build();
        soc.set_wram(vec![
            0xC0, 0xD0, 0x01,
        ]);
        soc.get_cpu().AW = 0xFE;

        soc.tick_cpu_no_cycles();
        assert_eq_hex!(soc.get_cpu().AW, 0xFC);
        assert!(soc.get_cpu().PSW.contains(CpuStatus::CARRY))
    }

    #[test]
    fn test_0xc1_2_rolc_mem_imm_16() {
        let mut soc = SoC::test_build();
        soc.set_wram(vec![
            0xC1, 0xD0, 0x01,
        ]);
        soc.get_cpu().AW = 0xFFFE;

        soc.tick_cpu_no_cycles();
        assert_eq_hex!(soc.get_cpu().AW, 0xFFFC);
        assert!(soc.get_cpu().PSW.contains(CpuStatus::CARRY))
    }

    #[test]
    fn test_0xc0_3_rorc_mem_imm_8() {
        let mut soc = SoC::test_build();
        soc.set_wram(vec![
            0xC0, 0xD8, 0x01,
        ]);
        soc.get_cpu().AW = 0xFF;

        soc.tick_cpu_no_cycles();
        assert_eq_hex!(soc.get_cpu().AW, 0x7F);
        assert!(soc.get_cpu().PSW.contains(CpuStatus::CARRY))
    }

    #[test]
    fn test_0xc1_3_rorc_mem_imm_16() {
        let mut soc = SoC::test_build();
        soc.set_wram(vec![
            0xC1, 0xD8, 0x01,
        ]);
        soc.get_cpu().AW = 0xFFFF;

        soc.tick_cpu_no_cycles();
        assert_eq_hex!(soc.get_cpu().AW, 0x7FFF);
        assert!(soc.get_cpu().PSW.contains(CpuStatus::CARRY))
    }

    #[test]
    fn test_0xc0_4_shl_mem_imm_8() {
        let mut soc = SoC::test_build();
        soc.set_wram(vec![
            0xC0, 0xE0, 0x01,
        ]);
        soc.get_cpu().AW = 0xFE;

        soc.tick_cpu_no_cycles();
        assert_eq_hex!(soc.get_cpu().AW, 0xFC);
        assert!(soc.get_cpu().PSW.contains(CpuStatus::CARRY))
    }

    #[test]
    fn test_0xc1_4_shl_mem_imm_16() {
        let mut soc = SoC::test_build();
        soc.set_wram(vec![
            0xC1, 0xE0, 0x01,
        ]);
        soc.get_cpu().AW = 0xFFFE;

        soc.tick_cpu_no_cycles();
        assert_eq_hex!(soc.get_cpu().AW, 0xFFFC);
        assert!(soc.get_cpu().PSW.contains(CpuStatus::CARRY))
    }

    #[test]
    fn test_0xd0_0_rol_1_8() {
        let mut soc = SoC::test_build();
        soc.set_wram(vec![
            0xD0, 0xC0,
        ]);
        soc.get_cpu().AW = 0xFE;

        soc.tick_cpu_no_cycles();
        assert_eq_hex!(soc.get_cpu().AW, 0xFD);
    }

    #[test]
    fn test_0xd1_0_rol_1_16() {
        let mut soc = SoC::test_build();
        soc.set_wram(vec![
            0xD1, 0xC0,
        ]);
        soc.get_cpu().AW = 0xFFFE;

        soc.tick_cpu_no_cycles();
        assert_eq_hex!(soc.get_cpu().AW, 0xFFFD);
    }

    #[test]
    fn test_0xd0_1_ror_1_8() {
        let mut soc = SoC::test_build();
        soc.set_wram(vec![
            0xD0, 0xC8,
        ]);
        soc.get_cpu().AW = 0x7F;

        soc.tick_cpu_no_cycles();
        assert_eq_hex!(soc.get_cpu().AW, 0xBF);
    }

    #[test]
    fn test_0xd1_1_ror_1_16() {
        let mut soc = SoC::test_build();
        soc.set_wram(vec![
            0xD1, 0xC8,
        ]);
        soc.get_cpu().AW = 0x7FFF;

        soc.tick_cpu_no_cycles();
        assert_eq_hex!(soc.get_cpu().AW, 0xBFFF);
    }

    #[test]
    fn test_0xf6_2_not_8() {
        let mut soc = SoC::test_build();
        soc.set_wram(vec![
            0xF6, 0xD0,
        ]);
        soc.get_cpu().AW = 0xFE;

        soc.tick_cpu_no_cycles();
        assert_eq_hex!(soc.get_cpu().AW, 0x01);
    }

    #[test]
    fn test_0xf7_2_not_16() {
        let mut soc = SoC::test_build();
        soc.set_wram(vec![
            0xF7, 0xD0,
        ]);
        soc.get_cpu().AW = 0xFFFE;

        soc.tick_cpu_no_cycles();
        assert_eq_hex!(soc.get_cpu().AW, 0x0001);
    }
}