use super::*;

impl V30MZ {
    pub fn push_op(&mut self, src: Operand, extra: u8) {
        // Stores a 16-bit value on the stack.
        let src = match src {
            Operand::SEGMENT => {
                let bits = (self.current_op[0] & 0b0001_1000) >> 3;
                *self.resolve_segment(bits)
            }
            Operand::REGISTER => {
                let bits = self.current_op[0] & 0b111;
                self.resolve_register_operand(bits, Mode::M16).try_into().unwrap()
            }

            // Using this to represent PUSH PSW
            // PUSH R implemented separately
            Operand::NONE => self.PSW.bits(),
            _ => self.resolve_src_16(src, extra)
        };
        // if src == self.SP {println!("Pushing src = {:04X}", src)};
        self.push(src);
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

    pub fn pop_op(&mut self, dest: Operand, extra: u8) {
        // Retrieves a 16-bit value from the stack and stores it in the operand.
        let src = self.pop();
        match dest {
            Operand::MEMORY => self.write_mem_operand_16(src, extra),
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
    }

    pub fn pop_r(&mut self) {
        // println!("POP R");
        // println!("SP before: {:04X}", self.SP);
        self.IY = self.pop();
        self.IX = self.pop();
        self.BP = self.pop();
        self.SP = self.SP.wrapping_add(2);
        self.BW = self.pop();
        self.DW = self.pop();
        self.CW = self.pop();
        self.AW = self.pop();
        // println!("IY {:04X} IX {:04X} BP {:04X} SP {:04X}", self.IY, self.IX, self.BP, self.SP);
        // println!("BW {:04X} DW {:04X} CW {:04X} AW {:04X}", self.BW, self.DW, self.CW, self.AW);
    }

    pub fn mov(&mut self, operation: &OpCode, extra: u8) {
        // Copies the value of op2 to op1
        // or reads two u16s from op3 and copies their values to op1 and op2
        let (mode, op1, op2, op3) = (operation.mode, operation.op1, operation.op2, operation.op3);

        if (op1, op2) == (Operand::REGISTER, Operand::IMMEDIATE) {
            return self.load_register_immediate(mode);
        }

        if (op1, op2) == (Operand::MEMORY, Operand::IMMEDIATE) {
            self.expect_op_bytes(2);
            let byte = self.current_op[1];
            self.resolve_mem_operand(byte, mode, extra);
            
            if mode == Mode::M8 {
                let src = self.expect_extra_byte();
                self.write_mem_operand_8(src, extra);
            } else {
                let src = self.expect_extra_word();
                self.write_mem_operand_16(src, extra);
            }
            return;
        }

        match op3 {
            None => {
                match mode {
                    Mode::M8 => {
                        let src = self.resolve_src_8(op2, extra);
                        self.write_src_to_dest_8(op1, src, extra);
                    }
                    Mode::M16 => {
                        let src = self.resolve_src_16(op2, extra);
                        self.write_src_to_dest_16(op1, src, extra);
                    }
                    Mode::M32 => panic!("32-bit move only valid when op3 exists"),
                }
            }
            Some(_) => {
                self.expect_op_bytes(2);
                let byte = self.current_op[1];
                let src = self.resolve_mem_src_32(byte, extra);

                let bits = (self.current_op[1] & 0b0011_1000) >> 3;

                self.write_reg_operand_16(src.0, bits);
                match operation.code {
                    0xC4 => self.DS1 = src.1,
                    0xC5 => self.DS0 = src.1,
                    code => panic!("Not a valid 3-term move opcode: {:02X}", code),
                }
            }
        }
    }

    pub fn ldea(&mut self, extra: u8) {
        // Calculates the offset of a memory operand and stores
        // the result into a 16-bit register.

        // LDEA requires at least one byte of operand code
        self.expect_op_bytes(2);

        let byte = self.current_op[1];
        let (address, _) = self.resolve_mem_operand(byte, Mode::M16, extra);

        let src = match address {
            MemOperand::Offset(offset) => offset,
            MemOperand::Register(RegisterType::RW(r)) => *r,
            _ => unreachable!(),
        };

        let bits = (self.current_op[1] >> 3) & 7;

        self.write_reg_operand_16(src, bits);
    }

    pub fn cvtbw(&mut self) {
        // Sign-extends AL into AW. If the highest bit of AL is clear,
        // stores 0x00 into AH. Otherwise, stores 0xFF into AH.

        let sign = self.AW & 0x0080 != 0;
        if sign {
            self.AW |= 0xFF00;
        } else {
            self.AW &= 0x00FF;
        }
    }

    pub fn cvtwl(&mut self) {
        // Sign-extends AW into DW,AW. If the highest bit of AW is clear,
        // stores 0x0000 into DW. Otherwise, stores 0xFFFF into DW.

        let sign = self.AW & 0x8000 != 0;
        self.DW = if sign {0xFFFF} else {0x0000};
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

    pub fn trans(&mut self) {
        // Calculates a memory offset as the unsigned sum of BW and AL,
        // and loads the byte at that offset into AL.
        let offset = self.BW.wrapping_add(self.AW & 0xFF);
        let addr = self.get_physical_address(offset, self.DS0);
        self.AW = swap_l(self.AW, self.read_mem(addr));
    }

    pub fn in_op(&mut self, mode: Mode, src: Operand) {
        // Inputs the value from the I/O port pointed to by src and stores it into AL.
        // If 16-bit, inputs the value from the I/O port pointed to by src + 1 and stores it into AH.

        let addr = self.get_io_address(src);

        // Request either one byte to be loaded into AL
        // or two bytes to be loaded into AL and AH respectively
        match mode {
            Mode::M8 => {
                let AL = self.read_io(addr as u8 as u16);

                self.AW = swap_l(self.AW, AL);
            }
            Mode::M16 => {
                let (AL, AH) = self.read_io_16(addr);

                self.AW = swap_l(self.AW, AL);
                self.AW = swap_h(self.AW, AH);
            }
            Mode::M32 => panic!("Unsuported mode"),
        }
    }

    pub fn out_op(&mut self, mode: Mode, dest: Operand) {
        // Outputs the value of AL to the I/O port pointed to by dest.
        // If 16-bit, outputs the value of AH to the I/O port pointed to by dest + 1.

        let dest = self.get_io_address(dest);
        match mode {
            Mode::M8 => self.write_io(dest as u8 as u16, self.AW as u8),
            Mode::M16 => self.write_io_16(dest, self.AW),
            Mode::M32 => unreachable!()
        }
    }
    
    pub fn xch(&mut self, mode: Mode, op1: Operand, op2: Operand, extra: u8) {
        // Exchanges the values stored in the operands. 

        match mode {
            Mode::M8 => {
                let src1 = self.resolve_src_8(op1, extra);
                let src2 = self.resolve_src_8(op2, extra);
                self.write_src_to_dest_8(op1, src2, extra);
                self.write_src_to_dest_8(op2, src1, extra);
            }
            Mode::M16 => {
                if op1 == Operand::MEMORY || op2 == Operand::MEMORY {
                    let src1 = self.resolve_src_16(op1, extra);
                    let src2 = self.resolve_src_16(op2, extra);
                    self.write_src_to_dest_16(op1, src2, extra);
                    self.write_src_to_dest_16(op2, src1, extra);
                } else {
                    let src1 = self.AW;
                    let bits = self.current_op[0] & 0b111;
                    let RegisterType::RW(r) = self.resolve_register_operand(bits, Mode::M16) else {unreachable!()};
                    let src2 = *r;
                    *r = src1;
                    self.AW = src2;

                }
            }
            _ => unreachable!(),
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
    fn test_0x06_push_seg() {
        let mut soc = SoC::test_build();
        soc.set_wram(vec![
            0xBC, 0x00, 0x10, // SP <- 0x1000
            0x06              // PUSH DS1
        ]);

        soc.get_cpu().DS1 = 0x1234;

        soc.tick_cpu_no_cycles();
        assert_eq_hex!(soc.get_cpu().SP, 0x1000);
        assert_eq_hex!(soc.get_cpu().PC, 0x0003);
        soc.tick_cpu_no_cycles();
        let addr = soc.get_cpu().get_stack_address();
        assert_eq_hex!(soc.get_cpu().PC, 0x0004);
        assert_eq_hex!(soc.read_mem_16(addr), 0x1234);
    }

    #[test]
    fn test_0x06_push_seg_cycles() {
        let mut soc = SoC::test_build();
        soc.set_wram(vec![
            0xBC, 0x00, 0x10, // SP <- 0x1000
            0x06              // PUSH DS1
        ]);

        soc.get_cpu().DS1 = 0x1234;

        soc.tick();
        assert_eq_hex!(soc.get_cpu().SP, 0x1000);
        assert_eq_hex!(soc.get_cpu().PC, 0x0003);
        soc.tick();
        soc.tick();
        let addr = soc.get_cpu().get_stack_address();
        assert_eq_hex!(soc.get_cpu().PC, 0x0004);
        assert_eq_hex!(soc.read_mem_16(addr), 0x1234);
    }

    #[test]
    fn test_push_pop_reg() {
        let mut soc = SoC::test_build();
        soc.set_wram(vec![
            0xB8, 0x11, 0x22, // AW <- 0x2211
            0xB9, 0x33, 0x44, // CW <- 0x4433
            0xBA, 0x55, 0x66, // DW <- 0x6655
            0xBB, 0x77, 0x88, // BW <- 0x8877
            0xBC, 0x00, 0x10, // SP <- 0xAA99
            0xBD, 0xBB, 0xCC, // BP <- 0xCCBB
            0xBE, 0xDD, 0xEE, // IX <- 0xEEDD
            0xBF, 0xFF, 0x00, // IY <- 0x00FF

            0x54, 0x50, 0x51, // PUSH
            0x52, 0x53, 0x55, // ALL
            0x56, 0x57,       // REGISTERS

            0x58, 0x59, 0x5A, // POP AW, CW, DW
        ]);

        for _ in 0..8 {
            soc.tick_cpu_no_cycles();
        }

        soc.tick_cpu_no_cycles();
        let addr = soc.get_cpu().get_stack_address();
        assert_eq_hex!(soc.read_mem_16(addr), soc.get_cpu().SP);

        soc.tick_cpu_no_cycles();
        let addr = soc.get_cpu().get_stack_address();
        assert_eq_hex!(soc.read_mem_16(addr), soc.get_cpu().AW);

        soc.tick_cpu_no_cycles();
        let addr = soc.get_cpu().get_stack_address();
        assert_eq_hex!(soc.read_mem_16(addr), soc.get_cpu().CW);

        soc.tick_cpu_no_cycles();
        let addr = soc.get_cpu().get_stack_address();
        assert_eq_hex!(soc.read_mem_16(addr), soc.get_cpu().DW);

        soc.tick_cpu_no_cycles();
        let addr = soc.get_cpu().get_stack_address();
        assert_eq_hex!(soc.read_mem_16(addr), soc.get_cpu().BW);

        soc.tick_cpu_no_cycles();
        let addr = soc.get_cpu().get_stack_address();
        assert_eq_hex!(soc.read_mem_16(addr), soc.get_cpu().BP);

        soc.tick_cpu_no_cycles();
        let addr = soc.get_cpu().get_stack_address();
        assert_eq_hex!(soc.read_mem_16(addr), soc.get_cpu().IX);

        soc.tick_cpu_no_cycles();
        let addr = soc.get_cpu().get_stack_address();
        assert_eq_hex!(soc.read_mem_16(addr), soc.get_cpu().IY);

        soc.tick_cpu_no_cycles();
        assert_eq_hex!(soc.get_cpu().AW, soc.get_cpu().IY);

        soc.tick_cpu_no_cycles();
        assert_eq_hex!(soc.get_cpu().CW, soc.get_cpu().IX);

        soc.tick_cpu_no_cycles();
        assert_eq_hex!(soc.get_cpu().DW, soc.get_cpu().BP);
    }

    #[test]
    fn test_0x88_mov_reg_to_mem_8() {
        let mut soc = SoC::test_build();
        soc.set_wram(vec![
            0x88, 0xC1,             // CL <- AL
            0x88, 0x04,             // WRAM[IX] <- AL
            0x88, 0x26, 0xFE, 0x00, // WRAM[0x00FE] <- AH
        ]);
        soc.get_cpu().AW = 0x1234;
        soc.get_cpu().IX = 0xFF;

        soc.tick_cpu_no_cycles();
        assert_eq_hex!(soc.get_cpu().PC, 0x0002);
        assert_eq_hex!(soc.get_cpu().CW, 0x0034);

        soc.tick_cpu_no_cycles();
        assert_eq_hex!(soc.get_cpu().PC, 0x0004);
        assert_eq_hex!(soc.get_wram().borrow()[0x00FF], 0x34);

        soc.tick_cpu_no_cycles();
        assert_eq_hex!(soc.get_cpu().PC, 0x0008);
        assert_eq_hex!(soc.get_wram().borrow()[0x00FE], 0x12);
    }

    #[test]
    fn test_0x89_mov_reg_to_mem_16() {
        let mut soc = SoC::test_build();
        soc.set_wram(vec![
            0x89, 0xC1,             // CW <- AW
            0x89, 0x04,             // WRAM[IX].16 <- AW
            0x89, 0x06, 0xFE, 0x00, // WRAM[0x00FE].16 <- AW
        ]);
        soc.get_cpu().AW = 0x1234;
        soc.get_cpu().IX = 0xFF;

        soc.tick_cpu_no_cycles();
        assert_eq_hex!(soc.get_cpu().PC, 0x0002);
        assert_eq_hex!(soc.get_cpu().CW, 0x1234);

        soc.tick_cpu_no_cycles();
        assert_eq_hex!(soc.get_cpu().PC, 0x0004);
        assert_eq_hex!(soc.get_wram().borrow()[0x00FF], 0x34);
        assert_eq_hex!(soc.get_wram().borrow()[0x0100], 0x12);

        soc.tick_cpu_no_cycles();
        assert_eq_hex!(soc.get_cpu().PC, 0x0008);
        assert_eq_hex!(soc.get_wram().borrow()[0x00FE], 0x34);
        assert_eq_hex!(soc.get_wram().borrow()[0x00FF], 0x12);
    }

    #[test]
    fn test_0x8a_mov_mem_to_reg_8() {
        let mut soc = SoC::test_build();
        soc.set_wram(vec![
            0x8A, 0xC1,             // AL <- CL
            0x8A, 0x04,             // AL <- WRAM[IX] = 0xFF
            0x8A, 0x06, 0xFE, 0x00, // AL <- WRAM[0x00FE] = 0x12
        ]);
        soc.get_cpu().CW = 0x1234;
        soc.get_cpu().IX = 0xFF;
        soc.get_wram().borrow_mut()[0x00FF] = 0xFF;
        soc.get_wram().borrow_mut()[0x00FE] = 0x12;

        soc.tick_cpu_no_cycles();
        assert_eq_hex!(soc.get_cpu().PC, 0x0002);
        assert_eq_hex!(soc.get_cpu().AW, 0x0034);

        soc.tick_cpu_no_cycles();
        assert_eq_hex!(soc.get_cpu().PC, 0x0004);
        assert_eq_hex!(soc.get_cpu().AW, 0x00FF);

        soc.tick_cpu_no_cycles();
        assert_eq_hex!(soc.get_cpu().PC, 0x0008);
        assert_eq_hex!(soc.get_cpu().AW, 0x0012);
    }

    #[test]
    fn test_0x8b_mov_mem_to_reg_16() {
        let mut soc = SoC::test_build();
        soc.set_wram(vec![
            0x8B, 0xC1,             // AW <- CW
            0x8B, 0x04,             // AW <- WRAM[IX] = 0xFFFF
            0x8B, 0x06, 0xFE, 0x00, // AW <- WRAM[0x00FE] = 0xFF12
            0x8B, 0xD0              // DW <- AW
        ]);
        soc.get_cpu().CW = 0x1234;
        soc.get_cpu().IX = 0xFF;
        soc.get_wram().borrow_mut()[0x00FF] = 0xFF;
        soc.get_wram().borrow_mut()[0x0100] = 0xFF;
        soc.get_wram().borrow_mut()[0x00FE] = 0x12;

        soc.tick_cpu_no_cycles();
        assert_eq_hex!(soc.get_cpu().PC, 0x0002);
        assert_eq_hex!(soc.get_cpu().AW, 0x1234);

        soc.tick_cpu_no_cycles();
        assert_eq_hex!(soc.get_cpu().PC, 0x0004);
        assert_eq_hex!(soc.get_cpu().AW, 0xFFFF);

        soc.tick_cpu_no_cycles();
        assert_eq_hex!(soc.get_cpu().PC, 0x0008);
        assert_eq_hex!(soc.get_cpu().AW, 0xFF12);

        soc.get_cpu().AW = 0x5101;

        soc.tick_cpu_no_cycles();
        assert_eq_hex!(soc.get_cpu().PC, 0x000A);
        assert_eq_hex!(soc.get_cpu().DW, 0x5101);
    }

    #[test]
    fn test_0x8c_mov_seg_to_mem() {
        let mut soc = SoC::test_build();
        soc.set_wram(vec![
            0x8C, 0xC1,             // CW <- DS1
            0x8C, 0x04,             // WRAM[IX] <- DS1
            0x8C, 0x06, 0xFE, 0x00, // WRAM[0x00FE] <- DS1
        ]);
        soc.get_cpu().DS1 = 0x1234;
        soc.get_cpu().IX = 0xFF;

        soc.tick_cpu_no_cycles();
        assert_eq_hex!(soc.get_cpu().PC, 0x0002);
        assert_eq_hex!(soc.get_cpu().CW, 0x1234);

        soc.tick_cpu_no_cycles();
        assert_eq_hex!(soc.get_cpu().PC, 0x0004);
        assert_eq_hex!(soc.get_wram().borrow()[0x00FF], 0x34);
        assert_eq_hex!(soc.get_wram().borrow()[0x0100], 0x12);

        soc.tick_cpu_no_cycles();
        assert_eq_hex!(soc.get_cpu().PC, 0x0008);
        assert_eq_hex!(soc.get_wram().borrow()[0x00FE], 0x34);
        assert_eq_hex!(soc.get_wram().borrow()[0x00FF], 0x12);
    }

    #[test]
    fn test_0x8d_ldea() {
        let mut soc = SoC::test_build();
        soc.set_wram(vec![
            0x8D, 0x06, 0xCD, 0xAB, // Immediate offset
            0x8D, 0xC1,             // Pointer to CW
            0x8D, 0x40, 0x11,       // BW + IX + 0x11
            0x8D, 0x40, 0xFF,       // BW + IX - 1
            0x8D, 0x80, 0x11, 0x11, // BW + IX + 0x1111
            0x8D, 0x80, 0xFF, 0xFF, // BW + IX - 1
        ]);

        soc.get_cpu().CW = 0x1234;
        soc.get_cpu().BW = 0x5678;
        soc.get_cpu().IX = 0x1111;

        soc.tick_cpu_no_cycles();
        assert_eq_hex!(soc.get_cpu().PC, 0x0004);
        assert_eq_hex!(soc.get_cpu().AW, 0xABCD);

        soc.tick_cpu_no_cycles();
        assert_eq_hex!(soc.get_cpu().PC, 0x0006);
        assert_eq_hex!(soc.get_cpu().AW, 0x1234);

        soc.tick_cpu_no_cycles();
        assert_eq_hex!(soc.get_cpu().PC, 0x0009);
        assert_eq_hex!(soc.get_cpu().AW, 0x679A);

        soc.tick_cpu_no_cycles();
        assert_eq_hex!(soc.get_cpu().AW, 0x6788);

        soc.tick_cpu_no_cycles();
        assert_eq_hex!(soc.get_cpu().AW, 0x789A);

        soc.tick_cpu_no_cycles();
        assert_eq_hex!(soc.get_cpu().AW, 0x6788);
    }

    #[test]
    fn test_0x8e_mov_mem_to_seg() {
        let mut soc = SoC::test_build();
        soc.set_wram(vec![
            0x8E, 0xC1,             // DS1 <- CW
            0x8E, 0x04,             // DS1 <- WRAM[IX] = 0xFFFF
            0x8E, 0x06, 0xFE, 0x00, // DS1 <- WRAM[0x00FE] = 0xFF12
        ]);
        soc.get_cpu().CW = 0x1234;
        soc.get_cpu().IX = 0xFF;
        soc.get_wram().borrow_mut()[0x00FF] = 0xFF;
        soc.get_wram().borrow_mut()[0x0100] = 0xFF;
        soc.get_wram().borrow_mut()[0x00FE] = 0x12;

        soc.tick_cpu_no_cycles();
        assert_eq_hex!(soc.get_cpu().PC, 0x0002);
        assert_eq_hex!(soc.get_cpu().DS1, 0x1234);

        soc.tick_cpu_no_cycles();
        assert_eq_hex!(soc.get_cpu().PC, 0x0004);
        assert_eq_hex!(soc.get_cpu().DS1, 0xFFFF);

        soc.tick_cpu_no_cycles();
        assert_eq_hex!(soc.get_cpu().PC, 0x0008);
        assert_eq_hex!(soc.get_cpu().DS1, 0xFF12);
    }

    #[test]
    fn test_0x98_cvtbw() {
        let mut soc = SoC::test_build();
        
        soc.get_cpu().AW = 0x00FF;
        soc.get_cpu().current_op = vec![0x98];
        soc.tick_cpu_no_cycles();
        assert_eq_hex!(soc.get_cpu().AW, 0xFFFF);

        soc.get_cpu().AW = 0xFF00;
        soc.get_cpu().current_op = vec![0x98];
        soc.tick_cpu_no_cycles();
        assert_eq_hex!(soc.get_cpu().AW, 0x0000);
    }

    #[test]
    fn test_0x99_cvtwl() {
        let mut soc = SoC::test_build();

        soc.get_cpu().AW = 0x8000;
        soc.get_cpu().current_op = vec![0x99];
        soc.tick_cpu_no_cycles();
        assert_eq_hex!(soc.get_cpu().DW, 0xFFFF);

        soc.get_cpu().AW = 0x7FFF;
        soc.get_cpu().current_op = vec![0x99];
        soc.tick_cpu_no_cycles();
        assert_eq_hex!(soc.get_cpu().DW, 0x0000);
    }

    #[test]
    fn test_0x9e_mov_ah_to_psw() {
        let mut soc = SoC::test_build();
        soc.set_wram(vec![
            0x9E, // PSW <- AH
        ]);

        soc.get_cpu().AW = 0x5500;
        soc.tick_cpu_no_cycles();
        assert_eq_hex!(soc.get_cpu().PC, 0x0001);
        assert_eq_hex!(soc.get_cpu().PSW.bits() & 0xFF, 0x57);
    }

    #[test]
    fn test_0x9f_mov_psw_to_ah() {
        let mut soc = SoC::test_build();
        soc.set_wram(vec![
            0x9F, // AH <- PSW
        ]);

        soc.get_cpu().PSW = CpuStatus::from_bits_truncate(0b1010_1010);
        soc.tick_cpu_no_cycles();
        assert_eq_hex!(soc.get_cpu().PC, 0x0001);
        assert_eq_hex!(soc.get_cpu().AW >> 8, 0xAA);
    }

    #[test]
    fn test_0xa0_mov_dir_mem_to_acc_16() {
        let mut soc = SoC::test_build();
        soc.set_wram(vec![
            0xA0, 0x34, 0x12, // AL <- [0x1234]
        ]);

        soc.get_cpu().DS0 = 0x0000;
        soc.get_cpu().AW = 0x0000;
        soc.get_wram().borrow_mut()[0x1234] = 0xAB;

        soc.tick_cpu_no_cycles();
        assert_eq_hex!(soc.get_cpu().PC, 0x0003);
        assert_eq_hex!(soc.get_cpu().AW, 0x00AB);
    }

    #[test]
    fn test_0xa1_mov_dir_mem_to_acc_16() {
        let mut soc = SoC::test_build();
        soc.set_wram(vec![
            0xA1, 0x34, 0x12, // AW <- [0x1234]
        ]);

        soc.get_cpu().DS0 = 0x0000;
        soc.get_cpu().AW = 0x0000;
        soc.get_wram().borrow_mut()[0x1234] = 0xCD;
        soc.get_wram().borrow_mut()[0x1235] = 0xAB;

        soc.tick_cpu_no_cycles();
        assert_eq_hex!(soc.get_cpu().PC, 0x0003);
        assert_eq_hex!(soc.get_cpu().AW, 0xABCD);
    }

    #[test]
    fn test_0xa2_mov_al_to_dir_mem_8() {
        let mut soc = SoC::test_build();
        soc.set_wram(vec![
            0xA2, 0x34, 0x12, // [0x1234] <- AL
        ]);

        soc.get_cpu().DS0 = 0x0000;
        soc.get_cpu().AW = 0x00FE;

        soc.tick_cpu_no_cycles();
        assert_eq_hex!(soc.get_cpu().PC, 0x0003);
        assert_eq_hex!(soc.get_wram().borrow()[0x1234], 0xFE);
    }

    #[test]
    fn test_0xa3_mov_aw_to_dir_mem() {
        let mut soc = SoC::test_build();
        soc.set_wram(vec![
            0xA3, 0x34, 0x12, // [0x1234] <- AW
        ]);

        soc.get_cpu().DS0 = 0x0000;
        soc.get_cpu().AW = 0xBEEF;

        soc.tick_cpu_no_cycles();
        assert_eq_hex!(soc.get_cpu().PC, 0x0003);
        assert_eq_hex!(soc.get_wram().borrow()[0x1234], 0xEF);
        assert_eq_hex!(soc.get_wram().borrow()[0x1235], 0xBE);
    }

    #[test]
    fn test_mov_imm_to_reg_8() {
        let mut soc = SoC::test_build();
        soc.set_wram(vec![
            0xB0, 0x12, // AL <- 0x12
            0xB4, 0x34, // AH <- 0x34
        ]);

        soc.tick_cpu_no_cycles();
        assert_eq_hex!(soc.get_cpu().PC, 0x0002);
        assert_eq_hex!(soc.get_cpu().AW, 0x0012);
        soc.tick_cpu_no_cycles();
        assert_eq_hex!(soc.get_cpu().PC, 0x0004);
        assert_eq_hex!(soc.get_cpu().AW, 0x3412);
    }

    #[test]
    fn test_0xb8_mov_imm_to_reg_16() {
        let mut soc = SoC::test_build();
        soc.set_wram(vec![
            0xB8, 0x34, 0x12, // AW <- 0x1234
        ]);

        soc.tick_cpu_no_cycles();
        assert_eq_hex!(soc.get_cpu().PC, 0x0003);
        assert_eq_hex!(soc.get_cpu().AW, 0x1234);
    }

    #[test]
    fn test_0xc4_mov_mem_to_ds1_reg() {
        let mut soc = SoC::test_build();
        soc.set_wram(vec![
            0xC4, 0x06, 0x00, 0x01, // CW <- 0x1234, DS1 <- 0x5678
        ]);
        soc.get_wram().borrow_mut()[0x0100] = 0x34;
        soc.get_wram().borrow_mut()[0x0101] = 0x12;
        soc.get_wram().borrow_mut()[0x0102] = 0x78;
        soc.get_wram().borrow_mut()[0x0103] = 0x56;

        soc.tick_cpu_no_cycles();
        assert_eq_hex!(soc.get_cpu().PC, 0x0004);
        assert_eq_hex!(soc.get_cpu().AW, 0x1234);
        assert_eq_hex!(soc.get_cpu().DS1, 0x5678);
    }

    #[test]
    fn test_0xc5_mov_mem_to_ds0_reg() {
        let mut soc = SoC::test_build();
        soc.set_wram(vec![
            0xC5, 0x06, 0x00, 0x01, // CW <- 0x1234, DS0 <- 0x5678
        ]);
        soc.get_wram().borrow_mut()[0x0100] = 0x34;
        soc.get_wram().borrow_mut()[0x0101] = 0x12;
        soc.get_wram().borrow_mut()[0x0102] = 0x78;
        soc.get_wram().borrow_mut()[0x0103] = 0x56;

        soc.tick_cpu_no_cycles();
        assert_eq_hex!(soc.get_cpu().PC, 0x0004);
        assert_eq_hex!(soc.get_cpu().AW, 0x1234);
        assert_eq_hex!(soc.get_cpu().DS0, 0x5678);
    }

    #[test]
    fn test_0xc6_mov_imm_to_mem_8() {
        let mut soc = SoC::test_build();
        soc.set_wram(vec![
            0xC6, 0x06, 0x00, 0x01, 0xAB, // WRAM[0x0100] <- 0xAB
        ]);

        soc.tick_cpu_no_cycles();
        assert_eq_hex!(soc.get_cpu().PC, 0x0005);
        assert_eq_hex!(soc.get_wram().borrow()[0x0100], 0xAB);
    }

    #[test]
    fn test_0xc7_mov_imm_to_mem_16() {
        let mut soc = SoC::test_build();
        soc.set_wram(vec![
            0xC7, 0x06, 0x00, 0x01, 0x34, 0x12, // WRAM[0x0100] <- 0x1234
        ]);

        soc.tick_cpu_no_cycles();
        assert_eq_hex!(soc.get_cpu().PC, 0x0006);
        assert_eq_hex!(soc.get_wram().borrow()[0x0100], 0x34);
        assert_eq_hex!(soc.get_wram().borrow()[0x0101], 0x12);
    }

    #[test]
    fn test_0xd6_salc() {
        let mut soc = SoC::test_build();
        soc.set_wram(vec![0xD6, 0xD6]);

        soc.get_cpu().PSW.insert(CpuStatus::CARRY);
        soc.tick_cpu_no_cycles();
        assert_eq_hex!(soc.get_cpu().AW, 0x00FF);
        
        soc.get_cpu().PSW.remove(CpuStatus::CARRY);
        soc.tick_cpu_no_cycles();
        assert_eq_hex!(soc.get_cpu().AW, 0x0000);
    }

    #[test]
    fn test_0xe4_in() {
        let mut soc = SoC::test_build();
        soc.set_wram(vec![0xE4, 0x00]);
        soc.set_io(vec![0xCD, 0xAB]);
        soc.tick_cpu_no_cycles();
        assert_eq_hex!(soc.get_cpu().AW, 0x00CD);
    }

    #[test]
    fn test_0xe5_in() {
        let mut soc = SoC::test_build();
        soc.set_wram(vec![0xE5, 0x00]);
        soc.set_io(vec![0xCD, 0xAB]);
        soc.tick_cpu_no_cycles();
        assert_eq_hex!(soc.get_cpu().AW, 0xABCD);
    }

    #[test]
    fn test_0xec_in() {
        let mut soc = SoC::test_build();
        soc.set_wram(vec![0xEC, 0xFF]);
        soc.set_io(vec![0xCD, 0xAB]);
        soc.get_cpu().DW = 0x00;
        soc.tick_cpu_no_cycles();
        assert_eq_hex!(soc.get_cpu().AW, 0x00CD);
    }

    #[test]
    fn test_0xed_in() {
        let mut soc = SoC::test_build();
        soc.set_wram(vec![0xED, 0xFF]);
        soc.set_io(vec![0xCD, 0xAB]);
        soc.get_cpu().DW = 0x00;
        soc.tick_cpu_no_cycles();
        assert_eq_hex!(soc.get_cpu().AW, 0xABCD);
    }
}