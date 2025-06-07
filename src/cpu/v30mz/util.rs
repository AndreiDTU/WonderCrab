use crate::cpu::parity;

use super::*;

impl V30MZ {
    pub fn allocate_instruction(&mut self) -> &OpCode {
        fn allocate_mod_rm(mod_rm: u8) -> u8 {
            let mode = mod_rm >> 6;
            let rm = mod_rm & 0b111;
            match (mode, rm) {
                (0b00, 0b110) | (0b10, _) => 2,
                (0b01, _) => 1,
                _ => 0,
            }
        }

        #[inline(always)]
        fn match_operand(op: &OpCode, operand: Operand) -> bool {
            op.op1 == operand || op.op2 == operand || op.op3 == Some(operand)
        }

        let addr = self.get_pc_address();
        let code = self.read_mem(addr);
        self.current_op.push(code);
        self.pc_displacement += 1;

        let op = &CPU_OP_CODES[code as usize];

        if op.code == 0x9A || op.code == 0xEA {
            self.pc_displacement = 5;
            let (word1, word2) = self.read_mem_32(addr.wrapping_add(1));
            let ([byte1, byte2], [byte3, byte4]) = (word1.to_le_bytes(), word2.to_le_bytes());
            self.current_op.push(byte1);
            self.current_op.push(byte2);
            self.current_op.push(byte3);
            self.current_op.push(byte4);
            return op;
        }

        if op.code == 0xC8 {
            self.pc_displacement = 4;
            let imm16 = self.read_mem_16(addr.wrapping_add(1)).to_le_bytes();
            let imm8 = self.read_mem(addr.wrapping_add(3));
            self.current_op.push(imm16[0]);
            self.current_op.push(imm16[1]);
            self.current_op.push(imm8);
            return op;
        }

        if match_operand(op, Operand::MEMORY) {
            let mod_rm = self.read_mem(addr.wrapping_add(1));
            self.current_op.push(mod_rm);
            self.pc_displacement += 1;

            let mem_bytes = allocate_mod_rm(mod_rm);
            self.pc_displacement += mem_bytes as u16;
            for i in 0..mem_bytes {
                let byte = self.read_mem(addr.wrapping_add(2 + (i as u32)));
                self.current_op.push(byte);
            }

            if code == 0xF6 || code == 0xF7 {
                let sub_op = (mod_rm & 0b0011_1000) >> 3;
                if sub_op != 0b000 {return op}
            }
        }

        let imm = match_operand(op, Operand::IMMEDIATE);

        if (((imm && op.mode == Mode::M8) || (match_operand(op, Operand::IMMEDIATE_S))) && op.code != 0xE8 && op.code != 0xE9) || 
            op.code == 0xC1 || op.code == 0xE5 || op.code == 0xE7
        {
            let imm8 = self.read_mem(addr.wrapping_add(self.pc_displacement as u32));
            self.pc_displacement += 1;
            self.current_op.push(imm8);
        } else if imm || match_operand(op, Operand::DIRECT) || op.code == 0xE8 || op.code == 0xE9 {
            let imm16 = self.read_mem_16(addr.wrapping_add(self.pc_displacement as u32)).to_le_bytes();
            self.pc_displacement += 2;
            self.current_op.push(imm16[0]);
            self.current_op.push(imm16[1]);
        }

        op
    }

    pub fn get_imm8(&mut self) -> u8 {
        *self.current_op.last().unwrap()
    }

    pub fn get_imm16(&mut self) -> u16 {
        u16::from_le_bytes(*self.current_op.last_chunk().unwrap())
    }

    pub(super) fn branch(&mut self, cond: bool) {
        // println!();
        // println!("Branch address before: {:05X}", self.get_pc_address());
        let displacement = self.current_op[1] as i8 as i16 as u16;
        if cond {
            self.PC = self.PC.wrapping_add(self.pc_displacement);
            self.pc_displacement = 0;
            self.PC = self.PC.wrapping_add(displacement);
        }
        // println!("Branch address after: {:05X}", self.get_pc_address());
        // println!()
    }

    pub fn update_flags_sub_8(&mut self, a: u8, b: u8, res: u8, carry: u8) {
        let old_sign = a & 0x80;
        let new_sign = res & 0x80;

        self.PSW.set(CpuStatus::ZERO, res == 0);
        self.PSW.set(CpuStatus::SIGN, new_sign != 0);
        self.PSW.set(CpuStatus::OVERFLOW, old_sign != b & 0x80 && old_sign != new_sign);
        self.PSW.set(CpuStatus::CARRY, a < b || (a as u16) < b as u16 + carry as u16);
        self.PSW.set(CpuStatus::PARITY, parity(res));
        self.PSW.set(CpuStatus::AUX_CARRY, a & 0x0F < b & 0x0F || (a & 0x0F) - (b & 0x0F) < carry);
    }

    pub fn update_flags_sub_16(&mut self, a: u16, b: u16, res: u16, carry: u16) {
        let old_sign = a & 0x8000;
        let new_sign = res & 0x8000;

        self.PSW.set(CpuStatus::ZERO, res == 0);
        self.PSW.set(CpuStatus::SIGN, new_sign != 0);
        self.PSW.set(CpuStatus::OVERFLOW, old_sign != b & 0x8000 && old_sign != new_sign);
        self.PSW.set(CpuStatus::CARRY, a < b || (a as u32) < b as u32 + carry as u32);
        self.PSW.set(CpuStatus::PARITY, parity(res as u8));
        self.PSW.set(CpuStatus::AUX_CARRY, a & 0x0F < b & 0x0F || (a & 0x0F) - (b & 0x0F) < carry);
    }

    pub fn update_flags_add_8(&mut self, a: u16, b: u16, res: u16, carry: u16) {
        let sign = res & 0x80;

        self.PSW.set(CpuStatus::ZERO, res as u8 == 0);
        self.PSW.set(CpuStatus::SIGN, sign != 0);
        self.PSW.set(CpuStatus::OVERFLOW, sign != a & 0x80 && sign != b & 0x80);
        self.PSW.set(CpuStatus::CARRY, res > 0xFF);
        self.PSW.set(CpuStatus::PARITY, parity(res as u8));
        self.PSW.set(CpuStatus::AUX_CARRY, (a & 0xF) + (b & 0xF) + carry > 0xF);
    }

    pub fn update_flags_add_16(&mut self, a: u32, b: u32, res: u32, carry: u32) {
        let sign = res & 0x8000;

        self.PSW.set(CpuStatus::ZERO, res as u16 == 0);
        self.PSW.set(CpuStatus::SIGN, sign != 0);
        self.PSW.set(CpuStatus::OVERFLOW, sign != a & 0x8000 && sign != b & 0x8000);
        self.PSW.set(CpuStatus::CARRY, res > 0xFFFF);
        self.PSW.set(CpuStatus::PARITY, parity(res as u8));
        self.PSW.set(CpuStatus::AUX_CARRY, (a & 0xF) + (b & 0xF) + carry > 0xF);
    }

    pub fn update_flags_bitwise_8(&mut self, res: u8) {
        self.PSW.set(CpuStatus::ZERO, res == 0);
        self.PSW.set(CpuStatus::SIGN, res & 0x80 != 0);
        self.PSW.remove(CpuStatus::OVERFLOW);
        self.PSW.remove(CpuStatus::CARRY);
        self.PSW.remove(CpuStatus::AUX_CARRY);
        self.PSW.set(CpuStatus::PARITY, parity(res));

    }

    pub fn update_flags_bitwise_16(&mut self, res: u16) {
        self.PSW.set(CpuStatus::ZERO, res == 0);
        self.PSW.set(CpuStatus::SIGN, res & 0x8000 != 0);
        self.PSW.remove(CpuStatus::OVERFLOW);
        self.PSW.remove(CpuStatus::CARRY);
        self.PSW.remove(CpuStatus::AUX_CARRY);
        self.PSW.set(CpuStatus::PARITY, parity(res as u8));

    }

    pub fn get_rot_src(&mut self, code: u8) -> u8 {
        // The src is always one byte
        return match code & 0xFE {
            0xC0 => self.get_imm8() & 0x1F,
            0xD0 => 1,
            0xD2 => self.CW as u8 & 0x1F,
            _ => unreachable!(),
        } & 0x1F
    }

    pub fn push(&mut self, src: u16) {
        self.SP = self.SP.wrapping_sub(2);
        let addr = self.get_stack_address();
        self.write_mem_16(addr, src);
        // if src == old_SP {println!("new SP = {:04X}", self.SP)}
    }

    pub fn pop(&mut self) -> u16 {
        let addr = self.get_stack_address();
        self.SP = self.SP.wrapping_add(2);
        self.read_mem_16(addr)
    }

    pub fn load_register_immediate(&mut self, mode: Mode) {
        match mode {
            Mode::M8 => {
                let src = self.current_op[1];

                let r_bits = self.current_op[0] & 0b111;
                let dest = self.resolve_register_operand(r_bits, mode);
                match dest {
                    RegisterType::RH(rh) => {
                        // println!("dest: {:04X} src: {:02X}", *rh, src);
                        *rh = swap_h(*rh, src)
                    },
                    RegisterType::RL(rl) => *rl = swap_l(*rl, src),
                    RegisterType::RW(_) => unreachable!(),
                }
            }
            Mode::M16 => {
                let src = u16::from_le_bytes([self.current_op[1], self.current_op[2]]);

                let r_bits = self.current_op[0] & 0b111;
                let dest = self.resolve_register_operand(r_bits, mode);
                match dest {
                    RegisterType::RW(r) => *r = src,
                    _ => unreachable!(),
                }
            }
            Mode::M32 => panic!("Mode not supported for immediate values!"),
        }
    }

    pub fn resolve_src_16(&mut self, op: Operand, extra: u8) -> u16 {
        match op {
            Operand::MEMORY => {
                let byte = self.current_op[1];

                self.resolve_mem_src_16(byte, extra)
            },
            Operand::REGISTER => {
                let r_bits = (self.current_op[1] & 0b0011_1000) >> 3;
                self.resolve_register_operand(r_bits, Mode::M16).try_into().unwrap()
            }
            Operand::ACCUMULATOR => self.AW,
            Operand::IMMEDIATE => {
                u16::from_le_bytes([self.current_op[1], self.current_op[2]])
            },
            Operand::SEGMENT => {
                let s_bits = (self.current_op[1] & 0b0001_1000) >> 3;
                *self.resolve_segment(s_bits)
            },
            Operand::DIRECT => {
                let addr = self.get_direct_mem_address();
                self.read_mem_16(addr)
            }
            Operand::IMMEDIATE_S => {
                self.current_op[1] as i8 as i16 as u16
            }
            Operand::NONE => panic!("None src not supported"),
        }
    }

    pub fn resolve_src_8(&mut self, op: Operand, extra: u8) -> u8 {
        match op {
            Operand::MEMORY => self.resolve_mem_src_8(self.current_op[1], extra),
            Operand::REGISTER => {
                let r_bits = (self.current_op[1] & 0b0011_1000) >> 3;
                self.resolve_register_operand(r_bits, Mode::M8).try_into().unwrap()
            }
            Operand::ACCUMULATOR => self.AW as u8,
            Operand::IMMEDIATE => self.current_op[1],
            Operand::DIRECT => {
                let addr = self.get_direct_mem_address();
                self.read_mem(addr)
            }
            _ => panic!("Unsuported 8-bit source type"),
        }
    }

    pub fn resolve_mem_src_32(&mut self, byte: u8, extra: u8) -> (u16, u16) {
        let (mem_operand, default_segment) = self.resolve_mem_operand(byte, Mode::M16, extra);

        match mem_operand {
            MemOperand::Offset(offset) => {
                let addr = self.get_physical_address(offset, default_segment);
                self.read_mem_32(addr)
            }
            MemOperand::Register(_) => unimplemented!()
        }
    }

    pub fn resolve_mem_src_16(&mut self, byte: u8, extra: u8) -> u16 {
        let (mem_operand, default_segment) = self.resolve_mem_operand(byte, Mode::M16, extra);

        match mem_operand {
            MemOperand::Offset(offset) => {
                let addr = self.get_physical_address(offset, default_segment);
                self.read_mem_16(addr)
            }
            MemOperand::Register(register_type) => register_type.try_into().unwrap()
        }
    }

    pub fn resolve_mem_src_8(&mut self, byte: u8, extra: u8) -> u8 {
        let (mem_operand, default_segment) = self.resolve_mem_operand(byte, Mode::M8, extra);

        match mem_operand {
            MemOperand::Offset(offset) => {
                let addr = self.get_physical_address(offset, default_segment);
                self.read_mem(addr)
            }
            MemOperand::Register(register_type) => register_type.try_into().unwrap()
        }
    }

    pub fn write_src_to_dest_16(&mut self, dest: Operand, src: u16, extra: u8) {
        match dest {
            Operand::MEMORY => self.write_mem_operand_16(src, extra),
            Operand::REGISTER => {
                let bits = (self.current_op[1] & 0b0011_1000) >> 3;
                self.write_reg_operand_16(src, bits);
            }
            Operand::ACCUMULATOR => self.AW = src,
            Operand::SEGMENT => self.write_to_seg_operand(src),
            Operand::DIRECT => {
                let addr = self.get_direct_mem_address();
                self.write_mem_16(addr, src)
            }
            _ => panic!("Unsupported 16-bit destination")
        }
    }

    pub fn write_src_to_dest_8(&mut self, dest: Operand, src: u8, extra: u8) {
        match dest {
            Operand::MEMORY => self.write_mem_operand_8(src, extra),
            Operand::REGISTER => {
                let bits = (self.current_op[1] & 0b0011_1000) >> 3;
                self.write_reg_operand_8(src, bits);
            }
            Operand::ACCUMULATOR => self.AW = swap_l(self.AW, src),
            Operand::DIRECT => {
                let addr = self.get_direct_mem_address();
                self.write_mem(addr, src)
            }
            _ => panic!("Unsupported 8-bit destination type"),
        };
    }

    pub fn get_direct_mem_address(&mut self) -> u32 {
        u16::from_le_bytes([self.current_op[1], self.current_op[2]]) as u32
    }

    pub fn write_mem_operand_16(&mut self, src: u16, extra: u8) {
        let byte = self.current_op[1];
        let (mem_operand, default_segment) = self.resolve_mem_operand(byte, Mode::M16, extra);

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
    }

    pub fn write_mem_operand_8(&mut self, src: u8, extra: u8) {
        let byte = self.current_op[1];
        let (mem_operand, default_segment) = self.resolve_mem_operand(byte, Mode::M8, extra);

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
    }

    pub fn write_reg_operand_16(&mut self, src: u16, bits: u8) {
        match self.resolve_register_operand(bits, Mode::M16) {
            RegisterType::RW(r) => *r = src,
            _ => unreachable!(),
        }
    }

    pub fn write_reg_operand_8(&mut self, src: u8, bits: u8) {
        match self.resolve_register_operand(bits, Mode::M8) {
            RegisterType::RW(_) => unreachable!(),
            RegisterType::RH(rh) => *rh = swap_h(*rh, src),
            RegisterType::RL(rl) => *rl = swap_l(*rl, src),
        }
    }

    pub fn write_to_seg_operand(&mut self, src: u16) {
        let s_bits = (self.current_op[1] & 0b0001_1000) >> 3;

        *self.resolve_segment(s_bits) = src;
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
    pub fn resolve_mem_operand(&mut self, byte: u8, mode: Mode, extra: u8) -> (MemOperand, u16) {
        let segment = self.DS0;
        let a = byte >> 6;
        let m = byte & 0b111;

        // When a is 3, m specifies the index of the register containing the operand's value.
        if a == 3 {return (MemOperand::Register(self.resolve_register_operand(m, mode)), segment)}
        else if self.cycles == self.base {
            self.cycles += extra;
        }

        // When a is 0 and m is 6, the operand's memory offset is not given by an expression.
        // Instead, the literal 16-bit offset is present as two additional bytes of program code (low byte first).
        if a == 0 && m == 6 {
            let offset = u16::from_le_bytes([self.current_op[2], self.current_op[3]]);
            return (MemOperand::Offset(offset), segment);
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
            1 => ((self.current_op[2] as i8) as i16) as u16,
            2 => u16::from_le_bytes([self.current_op[2], self.current_op[3]]),
            _ => unreachable!(),
        };

        (MemOperand::Offset(base.wrapping_add(displacement)), result_segment)
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

    pub fn get_physical_address(&self, offset: u16, default_segment: u16) -> u32 {
        let segment = match self.segment_override {
            None => default_segment,
            Some(s) => s,
        };

        self.apply_segment(offset, segment)
    }

    pub fn get_io_address(&mut self, src: Operand) -> u16 {
        // Use either the next byte padded with 0s or DW as the io_address
        match src {
            Operand::IMMEDIATE => self.current_op[1] as u16,
            Operand::NONE => self.DW,
            _ => panic!("Unsupported src operand for I/O Port"),
        }
    }

    pub fn get_stack_address(&self) -> u32 {
        self.apply_segment(self.SP, self.SS)
    }

    pub fn apply_segment(&self, offset: u16, segment: u16) -> u32 {
        let segment = (segment as u32) << 4;
        let offset = offset as u32;
        (offset + segment) & 0xFFFFF
    }
    
}