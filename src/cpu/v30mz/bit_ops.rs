use crate::cpu::parity;

use super::*;

impl V30MZ {
    pub fn and(&mut self, op1: Operand, op2: Operand, mode: Mode) {
        match mode {
            Mode::M8 => {
                let a = self.resolve_src_8(op1);
                let b = self.resolve_src_8(op2);
                let res = a & b;

                self.update_flags_bitwise_8(res);

                self.write_src_to_dest_8(op1, res);
            }
            Mode::M16 => {
                let a = self.resolve_src_16(op1);
                let b = self.resolve_src_16(op2);
                let res = a & b;

                self.update_flags_bitwise_16(res);

                self.write_src_to_dest_16(op1, res);
            }
            _ => unreachable!(),
        }
    }

    pub fn not(&mut self, mode: Mode) {
        match mode {
            Mode::M8 => {
                let src = self.resolve_src_8(Operand::MEMORY);
                self.write_src_to_dest_8(Operand::MEMORY, !src);
            }
            Mode::M16 => {
                let src = self.resolve_src_16(Operand::MEMORY);
                self.write_src_to_dest_16(Operand::MEMORY, !src);
            }
            Mode::M32 => unreachable!()
        }
    }

    pub fn or(&mut self, op1: Operand, op2: Operand, mode: Mode) {
        match mode {
            Mode::M8 => {
                let a = self.resolve_src_8(op1);
                let b = self.resolve_src_8(op2);
                let res = a | b;

                self.update_flags_bitwise_8(res);

                self.write_src_to_dest_8(op1, res);
            }
            Mode::M16 => {
                let a = self.resolve_src_16(op1);
                let b = self.resolve_src_16(op2);
                let res = a | b;

                self.update_flags_bitwise_16(res);

                self.write_src_to_dest_16(op1, res);
            }
            _ => unreachable!(),
        }
    }

    pub fn rol(&mut self, code: u8, mode: Mode) {
        let src = self.get_rot_src(code);

        match mode {
            Mode::M8 => {
                let dest = self.resolve_src_8(Operand::MEMORY);
                let res = dest.rotate_left(src as u32);

                if src != 0 {
                    self.PSW.set(CpuStatus::CARRY, res & 1 != 0);
                }

                self.PSW.set(CpuStatus::OVERFLOW, res >> 7 != res & 1);

                self.write_src_to_dest_8(Operand::MEMORY, res);
            }
            Mode::M16 => {
                let dest = self.resolve_src_16(Operand::MEMORY);
                let res = dest.rotate_left(src as u32);

                if src != 0 {
                    self.PSW.set(CpuStatus::CARRY, res & 1 != 0);
                }

                self.PSW.set(CpuStatus::OVERFLOW, res >> 15 != res & 1);

                self.write_src_to_dest_16(Operand::MEMORY, res);
            }
            _ => unreachable!()
        }
    }

    pub fn rolc(&mut self, code: u8, mode: Mode) {
        let src = self.get_rot_src(code);

        match mode {
            Mode::M8 => {
                let dest = self.resolve_src_8(Operand::MEMORY);
                let mut res = dest;
                let mut old_msb = res >> 7;

                for _ in 0..src {
                    let old_carry = self.PSW.contains(CpuStatus::CARRY) as u8;
                    self.PSW.set(CpuStatus::CARRY, res & 0x80 != 0);
                    old_msb = res >> 7;
                    res = (res << 1) | old_carry;
                }

                self.PSW.set(CpuStatus::OVERFLOW, res >> 7 != old_msb);

                self.write_src_to_dest_8(Operand::MEMORY, res);
            }
            Mode::M16 => {
                let dest = self.resolve_src_16(Operand::MEMORY);
                let mut res = dest;
                let mut old_msb = res >> 15;

                for _ in 0..src {
                    let old_carry = self.PSW.contains(CpuStatus::CARRY) as u16;
                    self.PSW.set(CpuStatus::CARRY, res & 0x8000 != 0);
                    old_msb = res >> 15;
                    res = (res << 1) | old_carry;
                }

                self.PSW.set(CpuStatus::OVERFLOW, res >> 15 != old_msb);

                self.write_src_to_dest_16(Operand::MEMORY, res);
            }
            _ => unreachable!()
        }
    }

    pub fn ror(&mut self, code: u8, mode: Mode) {
        let src = self.get_rot_src(code);

        match mode {
            Mode::M8 => {
                let dest = self.resolve_src_8(Operand::MEMORY);
                let res = dest.rotate_right(src as u32);

                if src != 0 {
                    self.PSW.set(CpuStatus::CARRY, res & 0x80 != 0);
                }

                self.PSW.set(CpuStatus::OVERFLOW, res >> 7 != res & 1);

                self.write_src_to_dest_8(Operand::MEMORY, res);
            }
            Mode::M16 => {
                let dest = self.resolve_src_16(Operand::MEMORY);
                let res = dest.rotate_right(src as u32);

                if src != 0 {
                    self.PSW.set(CpuStatus::CARRY, res & 0x8000 != 0);
                }

                self.PSW.set(CpuStatus::OVERFLOW, res >> 15 != res & 1);

                self.write_src_to_dest_16(Operand::MEMORY, res);
            }
            _ => unreachable!()
        }
    }

    pub fn rorc(&mut self, code: u8, mode: Mode) {
        let src = self.get_rot_src(code);

        match mode {
            Mode::M8 => {
                let dest = self.resolve_src_8(Operand::MEMORY);
                let mut res = dest;
                let mut old_msb = res >> 7;

                for _ in 0..src {
                    let old_carry = self.PSW.contains(CpuStatus::CARRY) as u8;
                    self.PSW.set(CpuStatus::CARRY, res & 1 != 0);
                    old_msb = res >> 7;
                    res = (old_carry << 7) | (res >> 1);
                }

                self.PSW.set(CpuStatus::OVERFLOW, res >> 7 != old_msb);

                self.write_src_to_dest_8(Operand::MEMORY, res);
            }
            Mode::M16 => {
                let dest = self.resolve_src_16(Operand::MEMORY);
                let mut res = dest;
                let mut old_msb = res >> 15;

                for _ in 0..src {
                    let old_carry = self.PSW.contains(CpuStatus::CARRY) as u16;
                    self.PSW.set(CpuStatus::CARRY, res & 1 != 0);
                    old_msb = res >> 15;
                    res = (old_carry << 15) | (res >> 1);
                }

                self.PSW.set(CpuStatus::OVERFLOW, res >> 15 != old_msb);

                self.write_src_to_dest_16(Operand::MEMORY, res);
            }
            _ => unreachable!()
        }
    }

    pub fn shl(&mut self, code: u8, mode: Mode) {
        let src = self.get_rot_src(code);

        match mode {
            Mode::M8 => {
                let dest = self.resolve_src_8(Operand::MEMORY);
                let res = dest << src;

                self.PSW.set(CpuStatus::ZERO, res == 0);
                self.PSW.set(CpuStatus::SIGN, res & 0x80 != 0);
                self.PSW.set(CpuStatus::OVERFLOW, dest & 0x80 != res & 0x80);
                self.PSW.set(CpuStatus::CARRY, dest << (src - 1) != 0);
                self.PSW.set(CpuStatus::PARITY, parity(res));

                self.write_src_to_dest_8(Operand::MEMORY, res);
            }
            Mode::M16 => {
                let dest = self.resolve_src_16(Operand::MEMORY);
                let res = dest << src;

                self.PSW.set(CpuStatus::ZERO, res == 0);
                self.PSW.set(CpuStatus::SIGN, res & 0x8000 != 0);
                self.PSW.set(CpuStatus::OVERFLOW, dest & 0x8000 != res & 0x8000);
                self.PSW.set(CpuStatus::CARRY, dest << (src - 1) != 0);
                self.PSW.set(CpuStatus::PARITY, parity(res as u8));

                self.write_src_to_dest_16(Operand::MEMORY, res);
            }
            _ => unreachable!()
        }
    }

    pub fn shr(&mut self, code: u8, mode: Mode) {
        let src = self.get_rot_src(code);

        match mode {
            Mode::M8 => {
                let dest = self.resolve_src_8(Operand::MEMORY);
                let res = dest >> src;

                self.PSW.set(CpuStatus::ZERO, res == 0);
                self.PSW.set(CpuStatus::SIGN, res & 0x80 != 0);
                self.PSW.set(CpuStatus::OVERFLOW, dest & 0x80 != res & 0x80);
                self.PSW.set(CpuStatus::CARRY, dest >> (src - 1) != 0);
                self.PSW.set(CpuStatus::PARITY, parity(res));

                self.write_src_to_dest_8(Operand::MEMORY, res);
            }
            Mode::M16 => {
                let dest = self.resolve_src_16(Operand::MEMORY);
                let res = dest >> src;

                self.PSW.set(CpuStatus::ZERO, res == 0);
                self.PSW.set(CpuStatus::SIGN, res & 0x8000 != 0);
                self.PSW.set(CpuStatus::OVERFLOW, dest & 0x8000 != res & 0x8000);
                self.PSW.set(CpuStatus::CARRY, dest >> (src - 1) != 0);
                self.PSW.set(CpuStatus::PARITY, parity(res as u8));

                self.write_src_to_dest_16(Operand::MEMORY, res);
            }
            _ => unreachable!()
        }
    }

    pub fn shra(&mut self, code: u8, mode: Mode) {
        let src = self.get_rot_src(code);

        match mode {
            Mode::M8 => {
                let dest = self.resolve_src_8(Operand::MEMORY);
                let res = (dest as i8 >> src) as u8;

                self.PSW.set(CpuStatus::ZERO, res == 0);
                self.PSW.set(CpuStatus::SIGN, res & 0x80 != 0);
                self.PSW.set(CpuStatus::OVERFLOW, dest & 0x80 != res & 0x80);
                self.PSW.set(CpuStatus::CARRY, dest >> (src - 1) != 0);
                self.PSW.set(CpuStatus::PARITY, parity(res));

                self.write_src_to_dest_8(Operand::MEMORY, res);
            }
            Mode::M16 => {
                let dest = self.resolve_src_16(Operand::MEMORY);
                let res = (dest as i16 >> src) as u16;

                self.PSW.set(CpuStatus::ZERO, res == 0);
                self.PSW.set(CpuStatus::SIGN, res & 0x8000 != 0);
                self.PSW.set(CpuStatus::OVERFLOW, dest & 0x8000 != res & 0x8000);
                self.PSW.set(CpuStatus::CARRY, dest >> (src - 1) != 0);
                self.PSW.set(CpuStatus::PARITY, parity(res as u8));

                self.write_src_to_dest_16(Operand::MEMORY, res);
            }
            _ => unreachable!()
        }
    }

    pub fn test(&mut self, op1: Operand, op2: Operand, mode: Mode) {
        match mode {
            Mode::M8 => {
                let a = self.resolve_src_8(op1);
                let b = self.resolve_src_8(op2);
                let res = a & b;

                self.update_flags_bitwise_8(res);
            }
            Mode::M16 => {
                let a = self.resolve_src_16(op1);
                let b = self.resolve_src_16(op2);
                let res = a & b;

                self.update_flags_bitwise_16(res);
            }
            _ => unreachable!(),
        }
    }

    pub fn xor(&mut self, op1: Operand, op2: Operand, mode: Mode) {
        match mode {
            Mode::M8 => {
                let a = self.resolve_src_8(op1);
                let b = self.resolve_src_8(op2);
                let res = a ^ b;

                self.update_flags_bitwise_8(res);

                self.write_src_to_dest_8(op1, res);
            }
            Mode::M16 => {
                let a = self.resolve_src_16(op1);
                let b = self.resolve_src_16(op2);
                let res = a ^ b;

                self.update_flags_bitwise_16(res);

                self.write_src_to_dest_16(op1, res);
            }
            _ => unreachable!(),
        }
    }
}