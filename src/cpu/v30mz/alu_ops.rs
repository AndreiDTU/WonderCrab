use crate::cpu::parity;

use super::*;

impl V30MZ {
    pub fn add(&mut self, op1: Operand, op2: Operand, mode: Mode, extra: u8) {
        // Adds the two operands. The result is stored in the left operand.
        match mode {
            Mode::M8 => {
                let old_dest = self.resolve_src_8(op1, extra) as u16;
                let src = if op2 == Operand::IMMEDIATE && op1 == Operand::MEMORY {
                    self.get_imm8()
                } else {
                    self.resolve_src_8(op2, extra)
                } as u16;

                let result = old_dest.wrapping_add(src);

                self.update_flags_add_8(old_dest, src, result, 0);

                self.write_src_to_dest_8(op1, result as u8, extra)
            }
            Mode::M16 => {
                let old_dest = self.resolve_src_16(op1, extra) as u32;
                let src = if op2 == Operand::IMMEDIATE_S {
                    self.get_imm8() as i8 as i16 as u16
                } else if op2 == Operand::IMMEDIATE && op1 == Operand::MEMORY {
                    self.get_imm16()
                } else {
                    self.resolve_src_16(op2, extra)
                } as u32;

                let result = old_dest.wrapping_add(src);

                self.update_flags_add_16(old_dest, src, result, 0);

                self.write_src_to_dest_16(op1, result as u16, extra)
            }
            Mode::M32 => unreachable!(),
        }
    }
    
    pub fn addc(&mut self, op1: Operand, op2: Operand, mode: Mode, extra: u8) {
        // Adds the two operands, plus 1 more if the carry flag (CY) was set.
        // The result is stored in the left operand. 
        match mode {
            Mode::M8 => {
                let old_dest = self.resolve_src_8(op1, extra) as u16;
                let src = if op2 == Operand::IMMEDIATE && op1 == Operand::MEMORY {
                    self.get_imm8()
                } else {
                    self.resolve_src_8(op2, extra)
                } as u16;

                let carry = self.PSW.contains(CpuStatus::CARRY) as u16;

                let result = old_dest.wrapping_add(src).wrapping_add(carry);

                self.update_flags_add_8(old_dest, src, result, carry);

                self.write_src_to_dest_8(op1, result as u8, extra)
            }
            Mode::M16 => {
                let old_dest = self.resolve_src_16(op1, extra) as u32;
                let src = if op2 == Operand::IMMEDIATE_S {
                    self.get_imm8() as i8 as i16 as u16
                } else if op2 == Operand::IMMEDIATE && op1 == Operand::MEMORY {
                    self.get_imm16()
                } else {
                    self.resolve_src_16(op2, extra)
                } as u32;

                let carry = self.PSW.contains(CpuStatus::CARRY) as u32;

                let result = old_dest.wrapping_add(src).wrapping_add(carry);

                self.update_flags_add_16(old_dest, src, result, carry);

                self.write_src_to_dest_16(op1, result as u16, extra)
            }
            Mode::M32 => unreachable!(),
        }
    }

    pub fn adj4a(&mut self) {
        let mut AL = self.AW as u8;
        if AL & 0x0F > 0x09 || self.PSW.contains(CpuStatus::AUX_CARRY) {
            AL = AL.wrapping_add(0x06);
            self.PSW.insert(CpuStatus::AUX_CARRY);
        }
        if AL > 0x9F || self.PSW.contains(CpuStatus::CARRY) {
            AL = AL.wrapping_add(0x60);
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
            self.PSW.insert(CpuStatus::AUX_CARRY);
        }
        if AL > 0x9F || self.PSW.contains(CpuStatus::CARRY) {
            AL = AL.wrapping_sub(0x60);
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

    pub fn cmp(&mut self, op1: Operand, op2: Operand, mode: Mode, extra: u8) {
        // println!("CMP address before: {:05X}", self.get_pc_address());
        match mode {
            Mode::M8 => {
                let (dest, src) = if op2 == Operand::IMMEDIATE && op1 == Operand::MEMORY {
                    (self.resolve_mem_src_8(self.current_op[1], extra), self.get_imm8())
                } else {
                    (self.resolve_src_8(op1, extra), self.resolve_src_8(op2, extra))
                };

                let result = dest.wrapping_sub(src);

                self.update_flags_sub_8(dest, src, result, 0);
            }
            Mode::M16 => {
                let dest = self.resolve_src_16(op1, extra);
                let src = if op2 == Operand::IMMEDIATE_S {
                    self.get_imm8() as i8 as i16 as u16
                } else if op2 == Operand::IMMEDIATE && op1 == Operand::MEMORY {
                    self.get_imm16()
                } else {
                    self.resolve_src_16(op2, extra)
                };

                let result = dest.wrapping_sub(src);

                self.update_flags_sub_16(dest, src, result, 0);
            }
            Mode::M32 => unreachable!(),
        }
        // println!("CMP address after: {:05X}", self.apply_segment(self.PC.wrapping_add(self.pc_displacement), self.PS));
    }

    pub fn dec(&mut self, op: Operand, mode: Mode, extra: u8) {
        let carry = self.PSW.contains(CpuStatus::CARRY);
        match op {
            Operand::REGISTER => {
                let bits = self.current_op[0] & 0b111;
                let RegisterType::RW(r) = self.resolve_register_operand(bits, Mode::M16) else {unreachable!()};
                let a = *r;
                let result = a.wrapping_sub(1);
                *r = result;
                self.update_flags_sub_16(a, 1, result, 0);
            }
            Operand::MEMORY => {
                match mode {
                    Mode::M8 => {
                        let a = self.resolve_src_8(Operand::MEMORY, extra);
                        let result = a.wrapping_sub(1);
                        self.update_flags_sub_8(a, 1, result, 0);
                        self.write_src_to_dest_8(Operand::MEMORY, result, extra);
                    }
                    Mode::M16 => {
                        let a = self.resolve_src_16(Operand::MEMORY, extra);
                        let result = a.wrapping_sub(1);
                        self.update_flags_sub_16(a, 1, result, 0);
                        self.write_src_to_dest_16(Operand::MEMORY, result, extra);
                    }
                    _ => unreachable!(),
                }
            }
            _ => unreachable!()
        }
        self.PSW.set(CpuStatus::CARRY, carry);
    }

    pub fn div(&mut self, mode: Mode, extra: u8) {
        match mode {
            Mode::M8 => {
                let divisor = self.resolve_mem_src_8(self.current_op[1], extra) as i8 as i16;
                if divisor == 0 && self.AW != 0x8000 {
                    self.PSW.remove(CpuStatus::AUX_CARRY);
                    self.PSW.remove(CpuStatus::SIGN);
                    self.PSW.remove(CpuStatus::PARITY);
                    return self.raise_exception(0)
                }

                let dividend = self.AW as i16;

                let (quotient, remainder) = if self.AW == 0x8000 && divisor == 0 {
                    (0x81, 0x00)
                } else {
                    (dividend / divisor, dividend.wrapping_rem(divisor) as i8)
                };

                if quotient > 0x7F || quotient < -0x7F {
                    self.PSW.remove(CpuStatus::AUX_CARRY);
                    self.PSW.remove(CpuStatus::SIGN);
                    self.PSW.remove(CpuStatus::PARITY);
                    return self.raise_exception(0)
                }

                self.PSW.remove(CpuStatus::AUX_CARRY);
                self.PSW.remove(CpuStatus::CARRY);
                self.PSW.remove(CpuStatus::OVERFLOW);
                self.PSW.set(CpuStatus::ZERO, quotient == 0);
                self.PSW.set(CpuStatus::PARITY, parity(quotient as i8 as u8));
                self.PSW.set(CpuStatus::SIGN, quotient < 0);

                self.AW = swap_h(self.AW, remainder as i8 as u8);
                self.AW = swap_l(self.AW, quotient as i8 as u8);
            }
            Mode::M16 => {
                let divisor = self.resolve_mem_src_16(self.current_op[1], extra) as i16 as i32;
                if divisor == 0 {
                    self.PSW.remove(CpuStatus::CARRY);
                    self.PSW.remove(CpuStatus::OVERFLOW);
                    self.PSW.remove(CpuStatus::AUX_CARRY);
                    self.PSW.remove(CpuStatus::SIGN);
                    self.PSW.remove(CpuStatus::PARITY);
                    return self.raise_exception(0)
                }

                let dividend = ((self.DW as u32) << 16 | self.AW as u32) as i32;

                let (quotient, remainder) = if self.DW == 0x8000 && divisor == 0 {
                    (0x8081, 0x00)
                } else {
                    (dividend / divisor, dividend.wrapping_rem(divisor))
                };

                if quotient > 0x7FFF || quotient < -0x7FFF {
                    self.PSW.remove(CpuStatus::CARRY);
                    self.PSW.remove(CpuStatus::OVERFLOW);
                    self.PSW.remove(CpuStatus::AUX_CARRY);
                    self.PSW.remove(CpuStatus::SIGN);
                    self.PSW.remove(CpuStatus::PARITY);
                    return self.raise_exception(0)
                }

                self.PSW.remove(CpuStatus::AUX_CARRY);
                self.PSW.remove(CpuStatus::CARRY);
                self.PSW.remove(CpuStatus::OVERFLOW);
                self.PSW.set(CpuStatus::ZERO, quotient == 0);
                self.PSW.set(CpuStatus::PARITY, parity(quotient as i8 as u8));
                self.PSW.set(CpuStatus::SIGN, quotient < 0);

                self.DW = remainder as i16 as u16;
                self.AW = quotient as i16 as u16;
            }
            _ => unreachable!()
        }
    }

    pub fn divu(&mut self, mode: Mode, extra: u8) {
        match mode {
            Mode::M8 => {
                let divisor = self.resolve_mem_src_8(self.current_op[1], extra) as u16;
                if divisor == 0 {
                    self.PSW.remove(CpuStatus::AUX_CARRY);
                    self.PSW.remove(CpuStatus::SIGN);
                    self.PSW.remove(CpuStatus::PARITY);
                    return self.raise_exception(0)
                }

                let dividend = self.AW;

                let quotient = dividend / divisor;
                if quotient > 0xFF {
                    self.PSW.remove(CpuStatus::AUX_CARRY);
                    self.PSW.remove(CpuStatus::SIGN);
                    self.PSW.remove(CpuStatus::PARITY);
                    return self.raise_exception(0)
                }

                let remainder = dividend.wrapping_rem(divisor) as u8;

                self.PSW.remove(CpuStatus::AUX_CARRY);
                self.PSW.remove(CpuStatus::SIGN);
                self.PSW.remove(CpuStatus::PARITY);
                self.PSW.set(CpuStatus::ZERO, remainder == 0 && (quotient & 1 != 0));

                self.AW = swap_h(self.AW, remainder);
                self.AW = swap_l(self.AW, quotient as u8);
            }
            Mode::M16 => {
                let divisor = self.resolve_mem_src_16(self.current_op[1], extra) as u32;
                if divisor == 0 {
                    self.PSW.remove(CpuStatus::CARRY);
                    self.PSW.remove(CpuStatus::OVERFLOW);
                    self.PSW.remove(CpuStatus::AUX_CARRY);
                    self.PSW.remove(CpuStatus::SIGN);
                    self.PSW.remove(CpuStatus::PARITY);
                    return self.raise_exception(0)
                }

                let quotient = self.AW as u32 / divisor;
                if quotient > 0xFFFF {
                    self.PSW.remove(CpuStatus::CARRY);
                    self.PSW.remove(CpuStatus::OVERFLOW);
                    self.PSW.remove(CpuStatus::AUX_CARRY);
                    self.PSW.remove(CpuStatus::SIGN);
                    self.PSW.remove(CpuStatus::PARITY);
                    return self.raise_exception(0)
                }

                let remainder = (self.AW as u32).wrapping_rem(divisor);

                self.PSW.remove(CpuStatus::CARRY);
                self.PSW.remove(CpuStatus::OVERFLOW);
                self.PSW.remove(CpuStatus::AUX_CARRY);
                self.PSW.remove(CpuStatus::SIGN);
                self.PSW.remove(CpuStatus::PARITY);
                self.PSW.set(CpuStatus::ZERO, remainder == 0 && (quotient & 1 != 0));

                self.DW = remainder as u16;
                self.AW = quotient as u16;
            }
            _ => unreachable!()
        }
    }

    pub fn inc(&mut self, op: Operand, mode: Mode, extra: u8) {
        let carry = self.PSW.contains(CpuStatus::CARRY);
        match op {
            Operand::REGISTER => {
                let bits = self.current_op[0] & 0b111;
                let RegisterType::RW(r) = self.resolve_register_operand(bits, Mode::M16) else {unreachable!()};
                let a = *r as u32;
                let result = a + 1;
                *r = result as u16;
                self.update_flags_add_16(a, 1, result, 0);
            }
            Operand::MEMORY => {
                match mode {
                    Mode::M8 => {
                        let a = self.resolve_src_8(Operand::MEMORY, extra) as u16;
                        let result = a + 1;
                        self.update_flags_add_8(a, 1, result, 0);
                        self.write_src_to_dest_8(Operand::MEMORY, result as u8, extra);
                    }
                    Mode::M16 => {
                        let a = self.resolve_src_16(Operand::MEMORY, extra) as u32;
                        let result = a + 1;
                        self.update_flags_add_16(a, 1, result, 0);
                        self.write_src_to_dest_16(Operand::MEMORY, result as u16, extra);
                    }
                    _ => unreachable!(),
                }
            }
            _ => unreachable!()
        }
        self.PSW.set(CpuStatus::CARRY, carry);
    }

    pub fn mul(&mut self, op3: Option<Operand>, mode: Mode, extra: u8) {
        match op3 {
            None => {
                match mode {
                    Mode::M8 => {
                        let factor = self.resolve_mem_src_8(self.current_op[1], extra) as i8 as i16;

                        self.AW = ((self.AW as u8 as i8 as i16) * factor) as u16;
                        let sign_ext = (self.AW & 0x80 == 0 && self.AW >> 8 != 0x00) || (self.AW & 0x80 != 0 && self.AW >> 8 != 0xFF);
                        self.PSW.set(CpuStatus::OVERFLOW, sign_ext);
                        self.PSW.set(CpuStatus::CARRY, sign_ext);
                    }
                    Mode::M16 => {
                        let factor1 = self.resolve_mem_src_16(self.current_op[1], extra) as i16 as i32;
                        let factor2 = self.AW as i16 as i32;
                        let result = factor1 * factor2;
                        self.AW = result as i16 as u16;
                        self.DW = (result >> 16) as i16 as u16;
                        
                        let sign_ext = (self.AW & 0x8000 == 0 && self.DW != 0x0000) || (self.AW & 0x8000 != 0 && self.DW != 0xFFFF);
                        self.PSW.set(CpuStatus::OVERFLOW, sign_ext);
                        self.PSW.set(CpuStatus::CARRY, sign_ext); 
                    }
                    Mode::M32 => unreachable!(),
                }
            }
            Some(op3) => {
                let factor1 = self.resolve_mem_src_16(self.current_op[1], extra) as i16;
                let factor2 = match op3 {
                    Operand::IMMEDIATE_S => self.get_imm8() as i8 as i16,
                    Operand::IMMEDIATE => self.get_imm16() as i16,
                    _ => unreachable!(),
                };

                let result = factor1 as i32 * factor2 as i32;
                let high_byte = (result >> 16) as i16 as u16;

                self.AW = result as i16 as u16;
                        
                let sign_ext = (self.AW & 0x8000 == 0 && high_byte != 0x0000) || (self.AW & 0x8000 != 0 && high_byte != 0xFFFF);
                self.PSW.set(CpuStatus::OVERFLOW, sign_ext);
                self.PSW.set(CpuStatus::CARRY, sign_ext); 
            }
        }
        self.PSW.insert(CpuStatus::ZERO);
        self.PSW.remove(CpuStatus::SIGN);
        self.PSW.remove(CpuStatus::PARITY);
        self.PSW.remove(CpuStatus::AUX_CARRY);
    }

    pub fn mulu(&mut self, mode: Mode, extra: u8) {
        match mode {
            Mode::M8 => {
                let factor = self.resolve_mem_src_8(self.current_op[1], extra) as u16;
                let result = self.AW as u8 as u16 * factor;
                self.AW = result;
                
                self.PSW.set(CpuStatus::OVERFLOW, result >> 8 != 0);
                self.PSW.set(CpuStatus::CARRY, result >> 8 != 0);
            }
            Mode::M16 => {
                let src = self.resolve_mem_src_16(self.current_op[1], extra) as u32;
                let result = self.AW as u32 * src;
                self.AW = result as u16;
                self.DW = (result >> 16) as u16;
                
                self.PSW.set(CpuStatus::OVERFLOW, self.DW != 0);
                self.PSW.set(CpuStatus::CARRY, self.DW != 0);
            }
            _ => unreachable!()
        }
        self.PSW.insert(CpuStatus::ZERO);
        self.PSW.remove(CpuStatus::SIGN);
        self.PSW.remove(CpuStatus::PARITY);
        self.PSW.remove(CpuStatus::AUX_CARRY);
    }

    pub fn neg(&mut self, mode: Mode, extra: u8) {
        match mode {
            Mode::M8 => {
                let src = self.resolve_mem_src_8(self.current_op[1], extra);
                let res = 0u8.wrapping_sub(src);
                self.write_mem_operand_8(res, extra);

                self.PSW.set(CpuStatus::ZERO, res == 0);
                self.PSW.set(CpuStatus::SIGN, res & 0x80 != 0);
                self.PSW.set(CpuStatus::OVERFLOW, src == (i8::MIN) as u8);
                self.PSW.set(CpuStatus::CARRY, src != 0);
                self.PSW.set(CpuStatus::PARITY, parity(res as u8));
                self.PSW.set(CpuStatus::AUX_CARRY, src & 0xF != 0);
            }
            Mode::M16 => {
                let src = self.resolve_mem_src_16(self.current_op[1], extra);
                let res = 0u16.wrapping_sub(src);
                self.write_mem_operand_16(res, extra);

                self.PSW.set(CpuStatus::ZERO, res == 0);
                self.PSW.set(CpuStatus::SIGN, res & 0x8000 != 0);
                self.PSW.set(CpuStatus::OVERFLOW, src == (i16::MIN) as u16);
                self.PSW.set(CpuStatus::CARRY, src != 0);
                self.PSW.set(CpuStatus::PARITY, parity(res as u8));
                self.PSW.set(CpuStatus::AUX_CARRY, src & 0xF != 0);
            }
            _ => unreachable!()
        }
    }

    pub fn cvtbd(&mut self) {
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
    }

    pub fn cvtdb(&mut self) {
        let src = self.current_op[1] as u16;

        let (AH, AL) = ((self.AW >> 8) as u8, self.AW as u8);

        let mul = AH as u16 * src;
        let result = mul + AL as u16;

        self.AW = swap_l(self.AW, result as u8);
        self.AW &= 0xFF;
        self.update_flags_add_8(AL as u16, mul, result, 0);
    }

    pub fn sub(&mut self, op1: Operand, op2: Operand, mode: Mode, extra: u8) {
        // Subtracts the two operands. The result is stored in the left operand.
        match mode {
            Mode::M8 => {
                let old_dest = self.resolve_src_8(op1, extra);
                let src = if op2 == Operand::IMMEDIATE && op1 == Operand::MEMORY {
                    self.get_imm8()
                } else {
                    self.resolve_src_8(op2, extra)
                };

                let result = old_dest.wrapping_sub(src);

                self.update_flags_sub_8(old_dest, src, result, 0);
                self.write_src_to_dest_8(op1, result, extra)
            }
            Mode::M16 => {
                let old_dest = self.resolve_src_16(op1, extra);
                let src = if op2 == Operand::IMMEDIATE_S {
                    self.get_imm8() as i8 as i16 as u16
                } else if op2 == Operand::IMMEDIATE && op1 == Operand::MEMORY {
                    self.get_imm16()
                } else {
                    self.resolve_src_16(op2, extra)
                };

                let result = old_dest.wrapping_sub(src);

                self.update_flags_sub_16(old_dest, src, result, 0);
                self.write_src_to_dest_16(op1, result, extra)
            }
            Mode::M32 => unreachable!(),
        }
    }

    pub fn subc(&mut self, op1: Operand, op2: Operand, mode: Mode, extra: u8) {
        // Subtracts the two operands. The result is stored in the left operand.
        match mode {
            Mode::M8 => {
                let old_dest = self.resolve_src_8(op1, extra);
                let src = if op2 == Operand::IMMEDIATE && op1 == Operand::MEMORY {
                    self.get_imm8()
                } else {
                    self.resolve_src_8(op2, extra)
                };

                let carry = self.PSW.contains(CpuStatus::CARRY) as u8;

                let result = old_dest.wrapping_sub(src).wrapping_sub(carry);

                self.update_flags_sub_8(old_dest, src, result, carry);
                self.write_src_to_dest_8(op1, result, extra)
            }
            Mode::M16 => {
                let old_dest = self.resolve_src_16(op1, extra);
                let src = if op2 == Operand::IMMEDIATE_S {
                    self.get_imm8() as i8 as i16 as u16
                } else if op2 == Operand::IMMEDIATE && op1 == Operand::MEMORY {
                    self.get_imm16()
                } else {
                    self.resolve_src_16(op2, extra)
                };

                let carry = self.PSW.contains(CpuStatus::CARRY) as u16;

                let result = old_dest.wrapping_sub(src).wrapping_sub(carry);

                self.update_flags_sub_16(old_dest, src, result, carry);
                self.write_src_to_dest_16(op1, result, extra)
            }
            Mode::M32 => unreachable!(),
        }
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod test {
    use crate::soc::SoC;
    use crate::assert_eq_hex;

    use super::*;

    #[test]
    fn test_0x00_add_register_to_memory_8() {
        let mut soc = SoC::test_build();
        soc.set_wram(vec![
            0x00, 0x06, 0xFE, 0x00, // [0x00FE] <- [0x00FE] + AL
        ]);
        soc.get_cpu().AW = 0x1234;
        soc.get_wram().borrow_mut()[0x00FE] = 0x01;

        soc.tick_cpu_no_cycles();
        assert_eq_hex!(soc.get_cpu().PC, 0x0004);
        assert_eq_hex!(soc.get_wram().borrow()[0x00FE], 0x35);
        assert!(soc.get_cpu().PSW.contains(CpuStatus::PARITY));
    }

    #[test]
    fn test_0x01_add_register_to_memory_16() {
        let mut soc = SoC::test_build();
        soc.set_wram(vec![
            0x01, 0x06, 0xFE, 0x00, // [0x00FE] <- [0x00FE] + AW
        ]);
        soc.get_cpu().AW = 0x1234;
        soc.get_wram().borrow_mut()[0x00FE] = 0xFF;
        soc.get_wram().borrow_mut()[0x00FF] = 0x01;

        soc.tick_cpu_no_cycles();
        assert_eq_hex!(soc.get_cpu().PC, 0x0004);
        assert_eq_hex!(soc.get_wram().borrow()[0x00FE], 0x33);
        assert_eq_hex!(soc.get_wram().borrow()[0x00FF], 0x14);
        assert!(soc.get_cpu().PSW.contains(CpuStatus::AUX_CARRY));
        assert!(soc.get_cpu().PSW.contains(CpuStatus::PARITY));
    }

    #[test]
    fn test_0x02_add_memory_to_register_8() {
        let mut soc = SoC::test_build();
        soc.set_wram(vec![
            0x02, 0x06, 0xFE, 0x00,
        ]);
        soc.get_cpu().AW = 0x1234;
        soc.get_wram().borrow_mut()[0x00FE] = 0x01;

        soc.tick_cpu_no_cycles();
        assert_eq_hex!(soc.get_cpu().PC, 0x0004);
        assert_eq_hex!(soc.get_cpu().AW, 0x1235);
        assert!(!soc.get_cpu().PSW.contains(CpuStatus::AUX_CARRY));
        assert!(soc.get_cpu().PSW.contains(CpuStatus::PARITY));
    }

    #[test]
    fn test_0x03_add_memory_to_register_16() {
        let mut soc = SoC::test_build();
        soc.set_wram(vec![
            0x03, 0x06, 0xFE, 0x00,
        ]);
        soc.get_cpu().AW = 0x1234;
        soc.get_wram().borrow_mut()[0x00FE] = 0xFF;
        soc.get_wram().borrow_mut()[0x00FF] = 0x01;

        soc.tick_cpu_no_cycles();
        assert_eq_hex!(soc.get_cpu().PC, 0x0004);
        assert_eq_hex!(soc.get_cpu().AW, 0x1433);
        assert!(soc.get_cpu().PSW.contains(CpuStatus::AUX_CARRY));
        assert!(soc.get_cpu().PSW.contains(CpuStatus::PARITY));
    }

    #[test]
    fn test_0x04_add_immediate_to_accumulator_8() {
        let mut soc = SoC::test_build();
        soc.set_wram(vec![
            0x04, 0xFF,
        ]);
        soc.get_cpu().AW = 0x12FF;

        soc.tick_cpu_no_cycles();
        assert_eq_hex!(soc.get_cpu().PC, 0x0002);
        assert_eq_hex!(soc.get_cpu().AW, 0x12FE);
        assert!(soc.get_cpu().PSW.contains(CpuStatus::AUX_CARRY));
        assert!(soc.get_cpu().PSW.contains(CpuStatus::SIGN));
        assert!(soc.get_cpu().PSW.contains(CpuStatus::CARRY));
        assert!(!soc.get_cpu().PSW.contains(CpuStatus::OVERFLOW));
    }

    #[test]
    fn test_0x05_add_immediate_to_accumulator_16() {
        let mut soc = SoC::test_build();
        soc.set_wram(vec![
            0x05, 0xFF, 0xFF
        ]);
        soc.get_cpu().AW = 0x12FF;

        soc.tick_cpu_no_cycles();
        assert_eq_hex!(soc.get_cpu().PC, 0x0003);
        assert_eq_hex!(soc.get_cpu().AW, 0x12FE);
        assert!(soc.get_cpu().PSW.contains(CpuStatus::AUX_CARRY));
        assert!(!soc.get_cpu().PSW.contains(CpuStatus::SIGN));
        assert!(soc.get_cpu().PSW.contains(CpuStatus::CARRY));
        assert!(!soc.get_cpu().PSW.contains(CpuStatus::OVERFLOW));
    }

    #[test]
    fn test_0x10_addc_register_to_memory_8() {
        let mut soc = SoC::test_build();
        soc.set_wram(vec![
            0x10, 0x06, 0xFE, 0x00, // [0x00FE] <- [0x00FE] + AL + carry
        ]);
        soc.get_cpu().AW = 0x1234;
        soc.get_cpu().PSW.insert(CpuStatus::CARRY);
        soc.get_wram().borrow_mut()[0x00FE] = 0x01;

        soc.tick_cpu_no_cycles();
        assert_eq_hex!(soc.get_cpu().PC, 0x0004);
        assert_eq_hex!(soc.get_wram().borrow()[0x00FE], 0x36);
        assert!(soc.get_cpu().PSW.contains(CpuStatus::PARITY));
    }

    #[test]
    fn test_0x11_addc_register_to_memory_16() {
        let mut soc = SoC::test_build();
        soc.set_wram(vec![
            0x11, 0x06, 0xFE, 0x00, // [0x00FE] <- [0x00FE] + AW + carry
        ]);
        soc.get_cpu().AW = 0x1234;
        soc.get_cpu().PSW.insert(CpuStatus::CARRY);
        soc.get_wram().borrow_mut()[0x00FE] = 0xFF;
        soc.get_wram().borrow_mut()[0x00FF] = 0x01;

        soc.tick_cpu_no_cycles();
        assert_eq_hex!(soc.get_cpu().PC, 0x0004);
        assert_eq_hex!(soc.get_wram().borrow()[0x00FE], 0x34);
        assert_eq_hex!(soc.get_wram().borrow()[0x00FF], 0x14);
        assert!(soc.get_cpu().PSW.contains(CpuStatus::AUX_CARRY));
        assert!(!soc.get_cpu().PSW.contains(CpuStatus::PARITY));
    }

    #[test]
    fn test_0x12_addc_memory_to_register_8() {
        let mut soc = SoC::test_build();
        soc.set_wram(vec![
            0x12, 0x06, 0xFE, 0x00,
        ]);
        soc.get_cpu().AW = 0x1234;
        soc.get_cpu().PSW.insert(CpuStatus::CARRY);
        soc.get_wram().borrow_mut()[0x00FE] = 0x01;

        soc.tick_cpu_no_cycles();
        assert_eq_hex!(soc.get_cpu().PC, 0x0004);
        assert_eq_hex!(soc.get_cpu().AW, 0x1236);
        assert!(!soc.get_cpu().PSW.contains(CpuStatus::AUX_CARRY));
        assert!(soc.get_cpu().PSW.contains(CpuStatus::PARITY));
    }

    #[test]
    fn test_0x13_addc_memory_to_register_16() {
        let mut soc = SoC::test_build();
        soc.set_wram(vec![
            0x13, 0x06, 0xFE, 0x00,
        ]);
        soc.get_cpu().AW = 0x1234;
        soc.get_cpu().PSW.insert(CpuStatus::CARRY);
        soc.get_wram().borrow_mut()[0x00FE] = 0xFF;
        soc.get_wram().borrow_mut()[0x00FF] = 0x01;

        soc.tick_cpu_no_cycles();
        assert_eq_hex!(soc.get_cpu().PC, 0x0004);
        assert_eq_hex!(soc.get_cpu().AW, 0x1434);
        assert!(soc.get_cpu().PSW.contains(CpuStatus::AUX_CARRY));
        assert!(!soc.get_cpu().PSW.contains(CpuStatus::PARITY));
    }

    #[test]
    fn test_0x14_addc_immediate_to_accumulator_8() {
        let mut soc = SoC::test_build();
        soc.set_wram(vec![
            0x14, 0xFF,
        ]);
        soc.get_cpu().AW = 0x12FF;
        soc.get_cpu().PSW.insert(CpuStatus::CARRY);

        soc.tick_cpu_no_cycles();
        assert_eq_hex!(soc.get_cpu().PC, 0x0002);
        assert_eq_hex!(soc.get_cpu().AW, 0x12FF); // 0xFF + 0xFF + 1 = 0x1FF
        assert!(soc.get_cpu().PSW.contains(CpuStatus::AUX_CARRY));
        assert!(soc.get_cpu().PSW.contains(CpuStatus::CARRY));
        assert!(soc.get_cpu().PSW.contains(CpuStatus::SIGN));
    }

    #[test]
    fn test_0x15_addc_immediate_to_accumulator_16() {
        let mut soc = SoC::test_build();
        soc.set_wram(vec![
            0x15, 0xFF, 0xFF
        ]);
        soc.get_cpu().AW = 0x12FF;
        soc.get_cpu().PSW.insert(CpuStatus::CARRY);

        soc.tick_cpu_no_cycles();
        assert_eq_hex!(soc.get_cpu().PC, 0x0003);
        assert_eq_hex!(soc.get_cpu().AW, 0x12FF); // 0x12FF + 0xFFFF + 1 = 0x12FF
        assert!(soc.get_cpu().PSW.contains(CpuStatus::CARRY));
        assert!(soc.get_cpu().PSW.contains(CpuStatus::AUX_CARRY));
        assert!(!soc.get_cpu().PSW.contains(CpuStatus::SIGN));
    }
}