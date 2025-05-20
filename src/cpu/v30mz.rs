use bitflags::bitflags;

use super::{opcode::CPU_OP_CODES, Mode, Operand};

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

    // MEMORY BUS COMMUNICATION
    pub current_op: Vec<u8>,
    pub mem_request: bool,

    // I/O BUS COMMUNICATION
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

            current_op: Vec::new(), mem_request: false,

            io_response: Vec::new(), io_address: 0, io_request: false,
        }
    }

    pub fn tick(&mut self) {
        self.current_op = Vec::new();
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
        self.mem_request = self.current_op.len() == 0;
        if self.mem_request {return;}

        let opcode = &CPU_OP_CODES[self.current_op[0] as usize];

        // This will return OK only if there are no pending bus requests
        let result = match opcode.code {
            0x8D => self.ldea(),
            0x98 => self.cvtbw(),
            0x99 => self.cvtwl(),
            0xE4 | 0xE5 | 0xEC | 0xED =>
                self.in_op(opcode.mode, opcode.op2),
                
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

    // Instructions

    fn ldea(&mut self) -> Result<(), ()> {
        // Calculates the offset of a memory operand and stores
        // the result into a 16-bit register.

        // LDEA requires at least one byte of operand code
        self.mem_request = self.current_op.len() < 2;
        if self.mem_request {return Err(())}

        todo!()
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
                    self.mem_request = self.current_op.len() < 2;
                    if self.mem_request {return Err(())}

                    self.current_op[1] as u16
                }
                Operand::NONE => {
                    self.DW
                }
                _ => panic!("Unsupported src operand for IN"),
            };
        }

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

    // Test functions

    #[cfg(test)]
    pub fn set_aw(&mut self, data: u16) {
        self.AW = data;
    }

    #[cfg(test)]
    pub fn get_aw(&self) -> u16 {
        self.AW
    }

    #[cfg(test)]
    pub fn get_dw(&self) -> u16 {
        self.DW
    }
}

#[cfg(test)]
mod test {
    use super::V30MZ;

    #[test]
    fn test_cvtbw() {
        let mut cpu = V30MZ::new();
        
        cpu.set_aw(0x00FF);
        cpu.current_op = vec![0x98];
        cpu.execute();
        assert_eq!(cpu.get_aw(), 0xFFFF);

        cpu.set_aw(0xFF00);
        cpu.current_op = vec![0x98];
        cpu.execute();
        assert_eq!(cpu.get_aw(), 0x0000);
    }

    #[test]
    fn test_cvtwl() {
        let mut cpu = V30MZ::new();

        cpu.set_aw(0x8000);
        cpu.current_op = vec![0x99];
        cpu.execute();
        assert_eq!(cpu.get_dw(), 0xFFFF);

        cpu.set_aw(0x7FFF);
        cpu.current_op = vec![0x99];
        cpu.execute();
        assert_eq!(cpu.get_dw(), 0x0000);
    }
}