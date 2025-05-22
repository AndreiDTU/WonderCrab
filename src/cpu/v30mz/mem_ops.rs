use super::*;

impl V30MZ {
    pub fn push_op(&mut self, src: Operand) -> Result<(), ()> {
        // Stores a 16-bit value on the stack.
        let src = match src {
            Operand::MEMORY => {
                self.expect_op_bytes(2)?;
                self.resolve_mem_src_16(self.current_op[1])?
            },
            Operand::REGISTER => {
                let bits = self.current_op[0] & 0b111;
                self.resolve_register_operand(bits, Mode::M16).try_into().unwrap()
            },
            Operand::ACCUMULATOR => self.AW,
            Operand::IMMEDIATE => {
                self.expect_op_bytes(3)?;
                u16::from_le_bytes([self.current_op[1], self.current_op[2]])
            },
            Operand::SEGMENT => {
                let bits = self.current_op[0] >> 3;
                *self.resolve_segment(bits)
            },
            Operand::DIRECT => unreachable!(),

            // Using this to represent PUSH PSW
            // PUSH R implemented separately
            Operand::NONE => self.PSW.bits()
        };
        self.push(src);

        Ok(())
    }

    pub fn push_s(&mut self) -> Result<(), ()> {
        self.expect_op_bytes(2)?;
        let src = self.current_op[1] as u16;
        self.push(src);

        Ok(())
    }

    pub fn push_r(&mut self) {
        let temp = self.SP;
        self.push(self.AW);
        self.push(self.CW);
        self.push(self.DW);
        self.push(self.BW);
        self.push(temp);
        self.push(self.BP);
        self.push(self.IX);
        self.push(self.IY);
    }

    pub fn pop_op(&mut self, dest: Operand) -> Result<(), ()> {
        // Retrieves a 16-bit value from the stack and stores it in the operand.
        let src = self.pop()?;
        match dest {
            Operand::MEMORY => self.write_mem_operand_16(src)?,
            Operand::REGISTER => {
                let bits = self.current_op[0] & 0b111;
                let RegisterType::RW(r) = self.resolve_register_operand(bits, Mode::M16) else {unreachable!()};
                *r = src;
            },
            Operand::ACCUMULATOR => self.AW = src,
            Operand::SEGMENT => {
                let bits = (self.current_op[0] & 0b0001_1000) >> 3;
                *self.resolve_segment(bits) = src;
            }

            // Using this to represent POP PSW
            // POP R implemented separately
            Operand::NONE => self.PSW = CpuStatus::from_bits_truncate(src),

            _ => unreachable!(),
        };

        Ok(())
    }

    pub fn pop_r(&mut self) -> Result<(), ()> {
        self.IY = self.pop()?;
        self.IX = self.pop()?;
        self.BP = self.pop()?;
        self.SP = self.SP.wrapping_add(2);
        self.BW = self.pop()?;
        self.DW = self.pop()?;
        self.CW = self.pop()?;
        self.AW = self.pop()?;

        Ok(())
    }

    pub fn mov(&mut self, operation: &OpCode) -> Result<(), ()> {
        // Copies the value of op2 to op1
        // or reads two u16s from op3 and copies their values to op1 and op2
        let (mode, op1, op2, op3) = (operation.mode, operation.op1, operation.op2, operation.op3);

        if (op1, op2) == (Operand::REGISTER, Operand::IMMEDIATE) {
            return self.load_register_immediate(mode);
        }

        if (op1, op2) == (Operand::MEMORY, Operand::IMMEDIATE) {
            self.expect_op_bytes(2)?;
            let byte = self.current_op[1];
            self.resolve_mem_operand(byte, mode)?;
            let imm_addr = self.get_pc_address();
            
            if mode == Mode::M8 {
                let src = self.read_mem(imm_addr)?;
                self.write_mem_operand_8(src)?;
            } else {
                let src = self.read_mem_16(imm_addr)?;
                self.PC = self.PC.wrapping_add(1);
                self.write_mem_operand_16(src)?;
            }
            return Ok(());
        }

        match op3 {
            None => {
                match mode {
                    Mode::M8 => {
                        let src = self.resolve_src_8(op2)?;
                        self.write_src_to_dest_8(op1, src)?;
                    }
                    Mode::M16 => {
                        let src = self.resolve_src_16(op2)?;
                        self.write_src_to_dest_16(op1, src)?;
                    }
                    Mode::M32 => panic!("32-bit move only valid when op3 exists"),
                }
            }
            Some(_) => {
                self.expect_op_bytes(2)?;
                let byte = self.current_op[1];
                let src = self.resolve_mem_src_32(byte)?;

                let bits = (self.current_op[1] & 0b0011_1000) >> 3;

                self.write_reg_operand_16(src.0, bits)?;
                match operation.code {
                    0xC4 => self.DS1 = src.1,
                    0xC5 => self.DS0 = src.1,
                    code => panic!("Not a valid 3-term move opcode: {:02X}", code),
                }
            }
        }

        Ok(())
    }

    pub fn ldea(&mut self, mode: Mode) -> Result<(), ()> {
        // Calculates the offset of a memory operand and stores
        // the result into a 16-bit register.

        // LDEA requires at least one byte of operand code
        self.expect_op_bytes(2)?;

        let byte = self.current_op[1];
        let address = match self.resolve_mem_operand(byte, mode) {
            Err(_) => return Err(()),
            Ok((op, _)) => {
                op
            }
        };

        match address {
            MemOperand::Offset(offset) => self.AW = offset,
            MemOperand::Register(RegisterType::RW(r)) => self.AW = *r,
            MemOperand::Register(RegisterType::RH(rh)) => {
                let AH = *rh as u8;
                self.AW = swap_h(self.AW, AH);
            }
            MemOperand::Register(RegisterType::RL(rl)) => {
                let AL = *rl as u8;
                self.AW = swap_l(self.AW, AL);
            }
        }

        Ok(())
    }

    pub fn cvtbw(&mut self) -> Result<(), ()> {
        // Sign-extends AL into AW. If the highest bit of AL is clear,
        // stores 0x00 into AH. Otherwise, stores 0xFF into AH.

        let sign = self.AW & 0x0080 != 0;
        if sign {
            self.AW |= 0xFF00;
        } else {
            self.AW &= 0x00FF;
        }

        Ok(())
    }

    pub fn cvtwl(&mut self) -> Result<(), ()> {
        // Sign-extends AW into DW,AW. If the highest bit of AW is clear,
        // stores 0x0000 into DW. Otherwise, stores 0xFFFF into DW.

        let sign = self.AW & 0x8000 != 0;
        self.DW = if sign {0xFFFF} else {0x0000};

        Ok(())
    }

    pub fn salc(&mut self) {
        // Sets AL according to the status of CY. If CY is clear,
        // stores 0x00 into AL. Otherwise, stores 0xFF into AL. 
        if self.PSW.contains(CpuStatus::CARRY) {
            self.AW = swap_l(self.AW, 0xFF);
        } else {
            self.AW = swap_l(self.AW, 0x00);
        }
    }

    pub fn trans(&mut self) -> Result<(), ()> {
        // Calculates a memory offset as the unsigned sum of BW and AL,
        // and loads the byte at that offset into AL.
        let offset = self.BW.wrapping_add(self.AW & 0b0000_1111);
        let addr = self.get_physical_address(offset, self.DS0);
        self.AW = swap_l(self.AW, self.read_mem(addr)?);
        
        Ok(())
    }

    pub fn in_op(&mut self, mode: Mode, src: Operand) -> Result<(), ()> {
        // Inputs the value from the I/O port pointed to by src and stores it into AL.
        // If 16-bit, inputs the value from the I/O port pointed to by src + 1 and stores it into AH.

        let addr = self.get_io_address(src)?;

        // Request either one byte to be loaded into AL
        // or two bytes to be loaded into AL and AH respectively
        match mode {
            Mode::M8 => {
                let AL = self.read_io(addr)?;

                self.AW = swap_l(self.AW, AL);
            }
            Mode::M16 => {
                let (AL, AH) = self.read_io_16(addr)?;

                self.AW = swap_l(self.AW, AL);
                self.AW = swap_h(self.AW, AH);
            }
            Mode::M32 => panic!("Unsuported mode"),
        }

        Ok(())
    }

    pub fn out_op(&mut self, mode: Mode, dest: Operand) -> Result<(), ()> {
        // Outputs the value of AL to the I/O port pointed to by dest.
        // If 16-bit, outputs the value of AH to the I/O port pointed to by dest + 1.

        let dest = self.get_io_address(dest)?;
        match mode {
            Mode::M8 => self.write_io(dest, self.AW as u8),
            Mode::M16 => self.write_io(dest.wrapping_add(1), (self.AW >> 8) as u8),
            Mode::M32 => unreachable!()
        }

        Ok(())
    }
    
    pub fn xch(&mut self, mode: Mode, op1: Operand, op2: Operand) -> Result<(), ()> {
        // Exchanges the values stored in the operands. 

        match mode {
            Mode::M8 => {
                let src1 = self.resolve_src_8(op1)?;
                let src2 = self.resolve_src_8(op2)?;
                self.write_src_to_dest_8(op1, src2)?;
                self.write_src_to_dest_8(op2, src1)?;
            }
            Mode::M16 => {
                match op2 {
                    Operand::MEMORY => {
                        let src1 = self.resolve_src_16(op1)?;
                        let src2 = self.resolve_src_16(op2)?;
                        self.write_src_to_dest_16(op1, src2)?;
                        self.write_src_to_dest_16(op2, src1)?;
                    }
                    _ => {
                        let src1 = self.AW;
                        let bits = (self.current_op[0] & 0b0011_1000) >> 3;
                        let RegisterType::RW(r) = self.resolve_register_operand(bits, Mode::M16) else {unreachable!()};
                        let src2 = *r;
                        *r = src1;
                        self.AW = src2;
                    }
                }
            }
            _ => unreachable!(),
        }

        Ok(())
    }
}