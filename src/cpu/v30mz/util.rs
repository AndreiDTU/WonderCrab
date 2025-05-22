use crate::cpu::parity;

use super::*;

impl V30MZ {
    pub fn update_flags_8(&mut self, a: u16, b: u16, res: u16) {
        let sign = res & 0x80;

        self.PSW.set(CpuStatus::ZERO, res as u8 == 0);
        self.PSW.set(CpuStatus::SIGN, sign != 0);
        self.PSW.set(CpuStatus::OVERFLOW, sign != a & 0x80 && sign != b & 0x80);
        self.PSW.set(CpuStatus::CARRY, res > 0xFF);
        self.PSW.set(CpuStatus::PARITY, parity(res as u8));
        self.PSW.set(CpuStatus::AUX_CARRY, (a & 0xF) + (b & 0xF) > 0xF);
    }

    pub fn update_flags_16(&mut self, a: u32, b: u32, res: u32) {
        let sign = res & 0x8000;

        self.PSW.set(CpuStatus::ZERO, res as u16 == 0);
        self.PSW.set(CpuStatus::SIGN, sign != 0);
        self.PSW.set(CpuStatus::OVERFLOW, sign != a & 0x8000 && sign != b & 0x8000);
        self.PSW.set(CpuStatus::CARRY, res > 0xFFFF);
        self.PSW.set(CpuStatus::PARITY, parity(res as u8));
        self.PSW.set(CpuStatus::AUX_CARRY, (a & 0xF) + (b & 0xF) > 0xF);
    }

    pub fn push(&mut self, src: u16) {
        self.SP = self.SP.wrapping_sub(2);
        let addr = self.get_stack_address();
        self.write_mem_16(addr, src);
    }

    pub fn pop(&mut self) -> Result<u16, ()> {
        let addr = self.get_stack_address();
        self.PS = self.PS.wrapping_add(2);
        self.read_mem_16(addr)
    }

    pub fn load_register_immediate(&mut self, mode: Mode) -> Result<(), ()> {
        match mode {
            Mode::M8 => {
                self.expect_op_bytes(2)?;
                let src = self.current_op[1];

                let r_bits = (self.current_op[0] & 0b111) >> 3;
                let dest = self.resolve_register_operand(r_bits, mode);
                match dest {
                    RegisterType::RH(rh) => *rh = swap_h(*rh, src),
                    RegisterType::RL(rl) => *rl = swap_l(*rl, src),
                    RegisterType::RW(_) => unreachable!(),
                }
            }
            Mode::M16 => {
                self.expect_op_bytes(3)?;
                let src = u16::from_le_bytes([self.current_op[1], self.current_op[2]]);

                let r_bits = (self.current_op[0] & 0b111) >> 3;
                let dest = self.resolve_register_operand(r_bits, mode);
                match dest {
                    RegisterType::RW(r) => *r = src,
                    _ => unreachable!(),
                }
            }
            Mode::M32 => panic!("Mode not supported for immediate values!"),
        }

        Ok(())
    }

    pub fn resolve_src_16(&mut self, op: Operand) -> Result<u16, ()> {
        match op {
            Operand::MEMORY => {
                self.expect_op_bytes(2)?;
                let byte = self.current_op[1];

                self.resolve_mem_src_16(byte)
            },
            Operand::REGISTER => {
                self.expect_op_bytes(2)?;

                let r_bits = (self.current_op[1] & 0b0011_1000) >> 3;
                self.resolve_register_operand(r_bits, Mode::M16).try_into()
            }
            Operand::ACCUMULATOR => Ok(self.AW),
            Operand::IMMEDIATE => {
                self.expect_op_bytes(3)?;
                Ok(u16::from_le_bytes([self.current_op[1], self.current_op[2]]))
            },
            Operand::SEGMENT => {
                self.expect_op_bytes(2)?;
                let s_bits = (self.current_op[1] & 0b0001_1000) >> 3;
                Ok(*self.resolve_segment(s_bits))
            },
            Operand::DIRECT => {
                let addr = self.get_direct_mem_address()?;
                self.read_mem_16(addr)
            }
            Operand::NONE => panic!("None src not supported"),
        }
    }

    pub fn resolve_src_8(&mut self, op: Operand) -> Result<u8, ()> {
        match op {
            Operand::MEMORY => {
                self.expect_op_bytes(2)?;

                let src = self.resolve_mem_src_8(self.current_op[1])?;
                Ok(src)
            }
            Operand::REGISTER => {
                self.expect_op_bytes(2)?;

                let r_bits = (self.current_op[1] & 0b0011_1000) >> 3;
                self.resolve_register_operand(r_bits, Mode::M8).try_into()
            }
            Operand::ACCUMULATOR => Ok(self.AW as u8),
            Operand::IMMEDIATE => {
                self.expect_op_bytes(2)?;

                Ok(self.current_op[1])
            }
            Operand::DIRECT => {
                let addr = self.get_direct_mem_address()?;
                self.read_mem(addr)
            }
            _ => panic!("Unsuported 8-bit source type"),
        }
    }

    pub fn resolve_mem_src_32(&mut self, byte: u8) -> Result<(u16, u16), ()> {
        let (mem_operand, default_segment) = self.resolve_mem_operand(byte, Mode::M16)?;

        match mem_operand {
            MemOperand::Offset(offset) => {
                let addr = self.get_physical_address(offset, default_segment);
                Ok(self.read_mem_32(addr)?)
            }
            MemOperand::Register(_) => unimplemented!()
        }
    }

    pub fn resolve_mem_src_16(&mut self, byte: u8) -> Result<u16, ()> {
        let (mem_operand, default_segment) = self.resolve_mem_operand(byte, Mode::M16)?;

        match mem_operand {
            MemOperand::Offset(offset) => {
                let addr = self.get_physical_address(offset, default_segment);
                self.read_mem_16(addr)
            }
            MemOperand::Register(register_type) => register_type.try_into()
        }
    }

    pub fn resolve_mem_src_8(&mut self, byte: u8) -> Result<u8, ()> {
        let (mem_operand, default_segment) = self.resolve_mem_operand(byte, Mode::M8)?;

        match mem_operand {
            MemOperand::Offset(offset) => {
                let addr = self.get_physical_address(offset, default_segment);
                self.read_mem(addr)
            }
            MemOperand::Register(register_type) => register_type.try_into()
        }
    }

    pub fn write_src_to_dest_16(&mut self, dest: Operand, src: u16) -> Result<(), ()> {
        match dest {
            Operand::MEMORY => self.write_mem_operand_16(src)?,
            Operand::REGISTER => {
                self.expect_op_bytes(2)?;

                let bits = (self.current_op[1] & 0b0011_1000) >> 3;
                self.write_reg_operand_16(src, bits)?;
            }
            Operand::ACCUMULATOR => self.AW = src,
            Operand::SEGMENT => self.write_to_seg_operand(src)?,
            Operand::DIRECT => {
                let addr = self.get_direct_mem_address()?;
                self.write_mem_16(addr, src)
            }
            _ => panic!("Unsupported 16-bit destination")
        }

        Ok(())
    }

    pub fn write_src_to_dest_8(&mut self, dest: Operand, src: u8) -> Result<(), ()> {
        match dest {
            Operand::MEMORY => self.write_mem_operand_8(src)?,
            Operand::REGISTER => {
                self.expect_op_bytes(2)?;

                let bits = (self.current_op[1] & 0b0011_1000) >> 3;
                self.write_reg_operand_8(src, bits)?;
            }
            Operand::ACCUMULATOR => self.AW = swap_l(self.AW, src),
            Operand::DIRECT => {
                let addr = self.get_direct_mem_address()?;
                self.write_mem(addr, src)
            }
            _ => panic!("Unsupported 8-bit destination type"),
        };

        Ok(())
    }

    pub fn get_direct_mem_address(&mut self) -> Result<u32, ()> {
        self.expect_op_bytes(3)?;

        let offset = u16::from_le_bytes([self.current_op[1], self.current_op[2]]);
        Ok(self.apply_segment(offset, self.DS0))
    }

    pub fn write_mem_operand_16(&mut self, src: u16) -> Result<(), ()> {
        self.expect_op_bytes(2)?;
        let byte = self.current_op[1];
        let (mem_operand, default_segment) = self.resolve_mem_operand(byte, Mode::M16)?;

        match mem_operand {
            MemOperand::Offset(offset) => {
                let addr = self.get_physical_address(offset, default_segment);
                self.write_mem_16(addr, src);
            }
            MemOperand::Register(register_type) => match register_type {
                RegisterType::RW(r) => *r = src,
                _ => unreachable!()
            }
        }

        Ok(())
    }

    pub fn write_mem_operand_8(&mut self, src: u8) -> Result<(), ()> {
        self.expect_op_bytes(2)?;
        let byte = self.current_op[1];
        let (mem_operand, default_segment) = self.resolve_mem_operand(byte, Mode::M8)?;

        match mem_operand {
            MemOperand::Offset(offset) => {
                let addr = self.get_physical_address(offset, default_segment);
                self.write_mem(addr, src);
            }
            MemOperand::Register(register_type) => match register_type {
                RegisterType::RW(_) => unreachable!(),
                RegisterType::RH(rh) => *rh = swap_h(*rh, src),
                RegisterType::RL(rl) => *rl = swap_l(*rl, src),
            }
        }

        Ok(())
    }

    pub fn write_reg_operand_16(&mut self, src: u16, bits: u8) -> Result<(), ()> {
        match self.resolve_register_operand(bits, Mode::M16) {
            RegisterType::RW(r) => *r = src,
            _ => unreachable!(),
        }

        Ok(())
    }

    pub fn write_reg_operand_8(&mut self, src: u8, bits: u8) -> Result<(), ()> {
        match self.resolve_register_operand(bits, Mode::M8) {
            RegisterType::RW(_) => unreachable!(),
            RegisterType::RH(rh) => *rh = swap_h(*rh, src),
            RegisterType::RL(rl) => *rl = swap_l(*rl, src),
        }

        Ok(())
    }

    pub fn write_to_seg_operand(&mut self, src: u16) -> Result<(), ()> {
        self.expect_op_bytes(2)?;
        let s_bits = (self.current_op[1] & 0b0001_1000) >> 3;

        *self.resolve_segment(s_bits) = src;

        Ok(())
    }

    pub fn resolve_register_operand(&mut self, bits: u8, mode: Mode) -> RegisterType<'_> {
        match mode {
            Mode::M8 => match bits {
                0 => RegisterType::RL(&mut self.AW),
                1 => RegisterType::RL(&mut self.CW),
                2 => RegisterType::RL(&mut self.DW),
                3 => RegisterType::RL(&mut self.BW),
                4 => RegisterType::RH(&mut self.AW),
                5 => RegisterType::RH(&mut self.CW),
                6 => RegisterType::RH(&mut self.DW),
                7 => RegisterType::RH(&mut self.BW),
                e => panic!("Invalid register index: {}", e),
            }
            Mode::M16 => match bits {
                0 => RegisterType::RW(&mut self.AW),
                1 => RegisterType::RW(&mut self.CW),
                2 => RegisterType::RW(&mut self.DW),
                3 => RegisterType::RW(&mut self.BW),
                4 => RegisterType::RW(&mut self.SP),
                5 => RegisterType::RW(&mut self.BP),
                6 => RegisterType::RW(&mut self.IX),
                7 => RegisterType::RW(&mut self.IY),
                e => panic!("Invalid register index: {}", e),
            }
            _ => panic!("Invalid register addressing mode"),
        }
    }

    // Returns the operand and its default segment's value
    pub fn resolve_mem_operand(&mut self, byte: u8, mode: Mode) -> Result<(MemOperand, u16), ()> {
        let segment = self.DS0;
        let a = byte >> 6;
        let m = byte & 0b111;

        // When a is 3, m specifies the index of the register containing the operand's value.
        if a == 3 {return Ok((MemOperand::Register(self.resolve_register_operand(m, mode)), segment))};

        // When a is 0 and m is 6, the operand's memory offset is not given by an expression.
        // Instead, the literal 16-bit offset is present as two additional bytes of program code (low byte first).
        if a == 0 && m == 6 {
            self.op_request = self.current_op.len() < 4;
            if self.op_request {return Err(())};

            let offset = u16::from_le_bytes([self.current_op[2], self.current_op[3]]);
            return Ok((MemOperand::Offset(offset), segment));
        }

        // When a is not 3, m specifies the base of the expression to use to calculate a memory offset.
        // If BP is present, the default segment register is SS. If BP is not present, the defaut segment register is DS0.
        let (base, result_segment) = match m {
            0 => (self.BW.wrapping_add(self.IX), segment),
            1 => (self.BW.wrapping_add(self.IY), segment),
            2 => (self.BW.wrapping_add(self.IX), self.SS),
            3 => (self.BW.wrapping_add(self.IY), self.SS),
            4 => (self.IX, segment),
            5 => (self.IY, segment),
            6 => (self.BP, self.SS),
            7 => (self.BW, segment),
            _ => unreachable!()
        };

        // The offset portion of the operand's physical address is calculated by evaluating the expression base
        // and optionally adding a signed displacement offset to it.
        let displacement = match a {
            0 => 0,
            1 => {
                self.op_request = self.current_op.len() < 3;
                if self.op_request {return Err(())}

                ((self.current_op[2] as i8) as i16) as u16
            }
            2 => {
                self.op_request = self.current_op.len() < 4;
                if self.op_request {return Err(())};

                u16::from_le_bytes([self.current_op[2], self.current_op[3]])
            }
            _ => unreachable!(),
        };

        Ok((MemOperand::Offset(base.wrapping_add(displacement)), result_segment))
    }

    pub fn resolve_segment(&mut self, bits: u8) -> &mut u16 {
        match bits {
            0 => &mut self.DS1,
            1 => &mut self.PS,
            2 => &mut self.SS,
            3 => &mut self.DS0,
            e => panic!("Invalid segment index: {}", e),
        }
    }

    // Returns Err if the current_op is shorter than the amount of bytes
    pub fn expect_op_bytes(&mut self, bytes: usize) -> Result<(), ()> {
        self.op_request = self.current_op.len() < bytes;
        if self.op_request {return Err(())} else {Ok(())}
    }

    pub fn get_physical_address(&self, offset: u16, default_segment: u16) -> u32 {
        let segment = match self.segment_override {
            None => default_segment,
            Some(s) => s,
        };

        self.apply_segment(offset, segment)
    }

    pub fn get_io_address(&mut self, src: Operand) -> Result<u16, ()> {
        // Use either the next byte padded with 0s or DW as the io_address
        match src {
            Operand::IMMEDIATE => {
                // Need at least one operand byte to access immediate value
                self.expect_op_bytes(2)?;

                Ok(self.current_op[1] as u16)
            }
            Operand::NONE => {
                Ok(self.DW)
            }
            _ => panic!("Unsupported src operand for I/O Port"),
        }
    }

    pub fn get_stack_address(&self) -> u32 {
        self.apply_segment(self.PS, self.SS)
    }

    pub fn apply_segment(&self, offset: u16, segment: u16) -> u32 {
        let segment = (segment as u32) << 4;
        let offset = offset as u32;
        (offset + segment) & 0xFFFFF
    }
    
}