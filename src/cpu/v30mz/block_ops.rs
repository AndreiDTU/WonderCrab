use crate::{bus::{io_bus::IOBusConnection, mem_bus::MemBusConnection}, cpu::{swap_l, Mode}};

use super::{CpuStatus, V30MZ};

impl V30MZ {
    /// CMPBK instruction
    /// 
    /// Updates the flags according to `[IX] - [IY]` and discards the result.
    /// 
    /// Intel name: CMPS
    pub fn cmpbk(&mut self, mode: Mode, cycles: u8, rep_cycles: u8) {
        let addr_x = self.get_physical_address(self.IX, self.DS0);
        let addr_y = self.apply_segment(self.IY, self.DS1);
        match mode {
            Mode::M8 => {
                let x = self.read_mem(addr_x);
                let y = self.read_mem(addr_y);

                self.update_flags_sub_8(x, y, x.wrapping_sub(y), 0);
            }
            Mode::M16 => {
                let x = self.read_mem_16(addr_x);
                let y = self.read_mem_16(addr_y);

                self.update_flags_sub_16(x, y, x.wrapping_sub(y), 0);
            }
            _ => unreachable!()
        }
        self.IX = self.update_block_index(mode, self.IX);
        self.IY = self.update_block_index(mode, self.IY);

        if self.rep {
            self.CW -= 1;
            self.rep = self.PSW.contains(CpuStatus::ZERO) == self.rep_z;
        }

        self.base = if self.rep {
            if self.rep_z {
                rep_cycles + 1
            } else {
                rep_cycles
            }
        } else {
            cycles
        };
        self.cycles = self.base;
    }

    /// CMPM instruction
    /// 
    /// Updates the flags according to `AW - [IY]` and discards the result.
    /// 
    /// Intel name: SCAS
    pub fn cmpm(&mut self, mode: Mode, cycles: u8, rep_cycles: u8) {
        let addr = self.apply_segment(self.IY, self.DS1);
        match mode {
            Mode::M8 => {
                let a = self.AW as u8;
                let b = self.read_mem(addr);

                self.update_flags_sub_8(a, b, a.wrapping_sub(b), 0);
            }
            Mode::M16 => {
                let b = self.read_mem_16(addr);

                self.update_flags_sub_16(self.AW, b, self.AW.wrapping_sub(b), 0);
            }
            _ => unreachable!()
        }
        self.IY = self.update_block_index(mode, self.IY);

        if self.rep {
            self.CW -= 1;
            self.rep = self.PSW.contains(CpuStatus::ZERO) == self.rep_z;
        }

        self.base = if self.rep {rep_cycles} else {cycles};
        self.cycles = self.base;
    }

    /// INM instruction
    /// 
    /// Reads the I/O port indicated by `DW` into `[IY]`
    /// 
    /// Intel name: INS
    pub fn inm(&mut self, mode: Mode, cycles: u8, rep_cycles: u8) {
        let addr = self.apply_segment(self.IY, self.DS1);
        match mode {
            Mode::M8 => {
                let byte = self.read_io(self.DW as u8 as u16);
                self.write_mem(addr, byte);
            }
            Mode::M16 => {
                let (lo, hi) = self.read_io_16(self.DW);
                let word = u16::from_le_bytes([lo, hi]);
                self.write_mem_16(addr, word);
            }
            _ => unreachable!()
        }
        self.IY = self.update_block_index(mode, self.IY);

        self.base = if self.rep {
            self.CW -= 1;
            rep_cycles
        } else {cycles};
        self.cycles = self.base;
    }

    /// LDM instruction
    /// 
    /// `acc <- [IX]`
    /// 
    /// Intel name: LODS
    pub fn ldm(&mut self, mode: Mode, cycles: u8, rep_cycles: u8) {
        let addr = self.get_physical_address(self.IX, self.DS0);
        match mode {
            Mode::M8 => {
                let src = self.read_mem(addr);
                self.AW = swap_l(self.AW, src);
            }
            Mode::M16 => self.AW = self.read_mem_16(addr),
            _ => unreachable!()
        }
        self.IX = self.update_block_index(mode, self.IX);

        self.base = if self.rep {
            self.CW -= 1;
            rep_cycles
        } else {cycles};
        self.cycles = self.base;
    }

    /// MOVBK instruction
    /// 
    /// `[IY] <- [IX]`
    /// 
    /// Intel name: MOVS
    pub fn movbk(&mut self, mode: Mode, cycles: u8, rep_cycles: u8) {
        let addr_x = self.get_physical_address(self.IX, self.DS0);
        let addr_y = self.apply_segment(self.IY, self.DS1);
        match mode {
            Mode::M8 => {
                let byte = self.read_mem(addr_x);
                self.write_mem(addr_y, byte);
            }
            Mode::M16 => {
                let word = self.read_mem_16(addr_x);
                self.write_mem_16(addr_y, word);
            }
            _ => unreachable!()
        }
        self.IX = self.update_block_index(mode, self.IX);
        self.IY = self.update_block_index(mode, self.IY);

        self.base = if self.rep {
            self.CW -= 1;
            rep_cycles
        } else {cycles};
        self.cycles = self.base;
    }

    /// OUTM instruction
    /// 
    /// Writes the value at `[IX]` into the I\O port indicated by `DW`
    /// 
    /// Intel name: OUTS
    pub fn outm(&mut self, mode: Mode, cycles: u8, rep_cycles: u8) {
        let addr = self.get_physical_address(self.IX, self.DS0);
        match mode {
            Mode::M8 => {
                let byte = self.read_mem(addr);
                self.write_io(self.DW as u8 as u16, byte);
            }
            Mode::M16 => {
                let word = self.read_mem_16(addr);
                self.write_io_16(self.DW, word);
            }
            _ => unreachable!()
        }
        self.IX = self.update_block_index(mode, self.IX);

        self.base = if self.rep {
            self.CW -= 1;
            rep_cycles
        } else {cycles};
        self.cycles = self.base;
    }

    /// STM instruction
    /// 
    /// `[IY] <- acc`
    /// 
    /// Intel name: STOS
    pub fn stm(&mut self, mode: Mode, cycles: u8, rep_cycles: u8) {
        let addr = self.apply_segment(self.IY, self.DS1);
        match mode {
            Mode::M8 => self.write_mem(addr, self.AW as u8),
            Mode::M16 => self.write_mem_16(addr, self.AW),
            _ => unreachable!()
        }
        self.IY = self.update_block_index(mode, self.IY);

        self.base = if self.rep {
            self.CW -= 1;
            rep_cycles
        } else {cycles};
        self.cycles = self.base;
    }

    /// Updates the provided in the index parameter based on the mode parameter and the direction flag
    fn update_block_index(&mut self, mode: Mode, index: u16) -> u16 {
        match (self.PSW.contains(CpuStatus::DIRECTION), mode) {
            (false, Mode::M8) => index.wrapping_add(1),
            (true, Mode::M8) => index.wrapping_sub(1),
            (false, Mode::M16) => index.wrapping_add(2),
            (true, Mode::M16) => index.wrapping_sub(2),
            _ => unreachable!()
        }
    }
}