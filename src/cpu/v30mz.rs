use bitflags::bitflags;

use super::{opcode::CPU_OP_CODES, MemOperand, Mode, Operand, RegisterType};

bitflags! {
    // http://perfectkiosk.net/stsws.html
    pub struct CpuStatus: u16 {
        const FIXED_ON_1  = 0x8000;
        const FIXED_ON_2  = 0x4000;
        const FIXED_ON_3  = 0x2000;
        const FIXED_ON_4  = 0x1000;

        const OVERFLOW    = 0x0800; // Set when the result of an operation is too large
        const DIRECTION   = 0x0400; // Specifies the direction of a memory or block operation
        const INTERRUPT   = 0x0200; // When set, interrupts will be processed
        const BREAK       = 0x0100; // When set, after each instruction executed, an exception is raised with vector 1

        const SIGN        = 0x0080; // Set when the result of an operation is negative
        const ZERO        = 0x0040; // Set when the result of an operation is zero
        const FIXED_OFF_1 = 0x0020;
        const AUX_CARRY   = 0x0010; // Similar to CY, but applies with respect to the lowest 4 bits of the operation.

        const FIXED_OFF_2 = 0x0008;
        const PARITY      = 0x0004; // Set when the number of set bits in the lower 8 bits of an operation is even, or cleared if odd.
        const FIXED_ON_5  = 0x0002;
        const CARRY       = 0x0001; // Set when an operation produces a carry or borrows.
    }
}

pub struct V30MZ {
    // REGISTERS

    // GENERAL-PURPOSE
    AW: u16, // Names ending W refer to the whole register
    BW: u16, // Names ending in L or H refer to the low
    CW: u16, // and high byte respectively
    DW: u16,

    // SEGMENT
    DS0: u16, // DATA SEGMENT 0
    DS1: u16, // DATA SEGMENT 1
    PS: u16,  // PROGRAM SEGMENT
    SS: u16,  // STACK SEGMENT

    // INDEX
    IX: u16,
    IY: u16,

    // POINTERS
    SP: u16, // STACK POINTER
    BP: u16, // BASE POINTER

    PC: u16, // PROGRAM COUNTER

    PSW: CpuStatus, // PROGRAM STATUS WORD

    // COMMUNICATION

    // MEMORY BUS
    pub current_op: Vec<u8>,
    pub op_request: bool,
    segment_override: Option<u16>,

    // I/O BUS
    pub io_response: Vec<u8>,
    io_address: u16,
    pub io_request: bool,
}

impl V30MZ {
    pub fn new() -> Self {
        Self {
            AW: 0, BW: 0, CW: 0, DW: 0,

            DS0: 0, DS1: 0, PS: 0, SS: 0,

            IX: 0, IY: 0,
            
            SP: 0, BP: 0,
            
            PC: 0,
            
            PSW: CpuStatus::from_bits_truncate(0),

            current_op: Vec::new(), op_request: false, segment_override: None,

            io_response: Vec::new(), io_address: 0, io_request: false,
        }
    }

    pub fn tick(&mut self) {
        self.execute();
    }

    pub fn get_pc_address(&mut self) -> u32 {
        let segment = (self.PS as u32) << 4;
        let counter = self.PC as u32;
        self.PC += 1;
        (counter + segment) & 0xFFFFF
    }

    pub fn get_io_address(&mut self) -> u16 {
        let addr = self.io_address;
        self.io_address += 1;
        addr
    }

    pub fn execute(&mut self) {
        // CPU requires at least one byte of instruction code to execute
        self.op_request = self.current_op.len() == 0;
        if self.op_request {return;}

        let op = &CPU_OP_CODES[self.current_op[0] as usize];

        // This will return OK only if there are no pending bus requests
        let result = match op.code {
            0x8D => self.ldea(op.mode),
            0x98 => self.cvtbw(),
            0x99 => self.cvtwl(),
            0xE4 | 0xE5 | 0xEC | 0xED =>
                self.in_op(op.mode, op.op2),
                
            _ => todo!(),
        };

        if result.is_ok() {self.finish_op();}
    }

    // Utility functions

    fn finish_op(&mut self) {
        self.current_op = Vec::new();
        self.io_response = Vec::new();
    }

    fn set_AL(&mut self, AL: u8) {
        self.AW = (self.AW & 0xFF00) | AL as u16
    }

    fn set_AH(&mut self, AH: u8) {
        self.AW = (self.AW & 0x00FF) | ((AH as u16) << 8)
    }

    fn resolve_register_operand(&mut self, bits: u8, mode: Mode) -> RegisterType<'_> {
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
                _ => unreachable!(),
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
                _ => unreachable!(),
            }
            _ => unreachable!()
        }
    }

    fn resolve_mem_operand(&mut self, byte: u8, mode: Mode) -> Result<(MemOperand, u16), ()> {
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

    // Instructions

    fn ldea(&mut self, mode: Mode) -> Result<(), ()> {
        // Calculates the offset of a memory operand and stores
        // the result into a 16-bit register.

        // LDEA requires at least one byte of operand code
        self.op_request = self.current_op.len() < 2;
        if self.op_request {return Err(())}

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
                self.set_AH(AH);
            }
            MemOperand::Register(RegisterType::RL(rl)) => {
                let AL = *rl as u8;
                self.set_AL(AL);
            }
        }

        Ok(())
    }

    fn cvtbw(&mut self) -> Result<(), ()> {
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

    fn cvtwl(&mut self) -> Result<(), ()> {
        // Sign-extends AW into DW,AW. If the highest bit of AW is clear,
        // stores 0x0000 into DW. Otherwise, stores 0xFFFF into DW.

        let sign = self.AW & 0x8000 != 0;
        self.DW = if sign {0xFFFF} else {0x0000};

        Ok(())
    }

    fn in_op(&mut self, mode: Mode, src: Operand) -> Result<(), ()> {
        // Inputs the value from the I/O port pointed to by src and stores it into AL.
        // If 16-bit, inputs the value from the I/O port pointed to by src + 1 and stores it into AH.

        // Use either the next byte padded with 0s or DW as the io_address
        if self.io_response.is_empty() {
            self.io_address = match src {
                Operand::IMMEDIATE => {
                    // Need at least one operand byte to access immediate value
                    self.op_request = self.current_op.len() < 2;
                    if self.op_request {return Err(())}

                    self.current_op[1] as u16
                }
                Operand::NONE => {
                    self.DW
                }
                _ => panic!("Unsupported src operand for IN"),
            };
        }

        // Request either one byte to be loaded into AL
        // or two bytes to be loaded into AL and AH respectively
        match mode {
            Mode::M8 => {
                self.io_request = self.io_response.len() == 0;
                if self.io_request {return Err(())}

                let AL = self.io_response[0];

                self.set_AL(AL);
            }
            Mode::M16 => {
                self.io_request = self.io_response.len() < 2;
                if self.io_request {return Err(())}

                let (AL, AH) = (self.io_response[0], self.io_response[1]);

                self.set_AL(AL);
                self.set_AH(AH);
            }
            Mode::M32 => panic!("Unsuported mode"),
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::{cpu, soc::SoC};

    use super::V30MZ;

    #[test]
    fn test_cvtbw() {
        let mut cpu = V30MZ::new();
        
        cpu.AW = 0x00FF;
        cpu.current_op = vec![0x98];
        cpu.execute();
        assert_eq!(cpu.AW, 0xFFFF);

        cpu.AW = 0xFF00;
        cpu.current_op = vec![0x98];
        cpu.execute();
        assert_eq!(cpu.AW, 0x0000);
    }

    #[test]
    fn test_cvtwl() {
        let mut cpu = V30MZ::new();

        cpu.AW = 0x8000;
        cpu.current_op = vec![0x99];
        cpu.execute();
        assert_eq!(cpu.DW, 0xFFFF);

        cpu.AW = 0x7FFF;
        cpu.current_op = vec![0x99];
        cpu.execute();
        assert_eq!(cpu.DW, 0x0000);
    }

    #[test]
    fn test_in_0xe4() {
        let mut soc = SoC::new();
        soc.set_wram(vec![0xE4, 0x00]);
        soc.set_io(vec![0xCD, 0xAB]);
        soc.tick();
        assert_eq!(soc.get_cpu().AW, 0x00CD);
    }

    #[test]
    fn test_in_0xe5() {
        let mut soc = SoC::new();
        soc.set_wram(vec![0xE5, 0x00]);
        soc.set_io(vec![0xCD, 0xAB]);
        soc.tick();
        assert_eq!(soc.get_cpu().AW, 0xABCD);
    }

    #[test]
    fn test_in_0xec() {
        let mut soc = SoC::new();
        soc.set_wram(vec![0xEC, 0xFF]);
        soc.set_io(vec![0xCD, 0xAB]);
        soc.get_cpu().DW = 0x00;
        soc.tick();
        assert_eq!(soc.get_cpu().AW, 0x00CD);
    }

    #[test]
    fn test_in_0xed() {
        let mut soc = SoC::new();
        soc.set_wram(vec![0xED, 0xFF]);
        soc.set_io(vec![0xCD, 0xAB]);
        soc.get_cpu().DW = 0x00;
        soc.tick();
        assert_eq!(soc.get_cpu().AW, 0xABCD);
    }

    #[test]
    fn test_ldea_0x8d() {
        let mut soc = SoC::new();
        soc.set_wram(vec![
            0x8D, 0x06, 0xCD, 0xAB, // Immediate offset
            0x8D, 0xC1,             // Pointer to CW
            0x8D, 0x40, 0x11,       // BW + IX + 0x1111
            0x8D, 0x40, 0xFF,       // BW + IX - 1
            0x8D, 0x80, 0x11, 0x11, // BW + IX + 0x1111
            0x8D, 0x80, 0xFF, 0xFF, // BW + IX - 1
        ]);

        soc.get_cpu().CW = 0x1234;
        soc.get_cpu().BW = 0x5678;
        soc.get_cpu().IX = 0x1111;

        soc.tick();
        assert_eq!(soc.get_cpu().AW, 0xABCD);

        soc.tick();
        assert_eq!(soc.get_cpu().AW, 0x1234);

        soc.tick();
        assert_eq!(soc.get_cpu().AW, 0x679A);

        soc.tick();
        assert_eq!(soc.get_cpu().AW, 0x6788);

        soc.tick();
        assert_eq!(soc.get_cpu().AW, 0x789A);

        soc.tick();
        assert_eq!(soc.get_cpu().AW, 0x6788);
    }
}