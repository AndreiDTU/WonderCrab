use crate::{bus::mem_bus::MemBusConnection, cpu::{Mode, Operand}};

use super::{CpuStatus, V30MZ};

impl V30MZ {
    /// BR instruction (unconditional jump)
    /// 
    /// Intel name: JMP
    pub fn branch_op(&mut self, op: Operand, mode: Mode, extra: u8) {
        // println!("JUMP from address: {:05X}", self.get_pc_address());
        match op {
            Operand::IMMEDIATE => {
                self.PC = u16::from_le_bytes(self.current_op[1..=2].try_into().unwrap());
                self.PS = u16::from_le_bytes(self.current_op[3..=4].try_into().unwrap());
            },
            Operand::IMMEDIATE_S => {
                match mode {
                    Mode::M8 => self.branch(true),
                    Mode::M16 => {
                        let displacement = self.get_imm16();
                        self.PC = self.PC.wrapping_add(self.pc_displacement);
                        self.PC = self.PC.wrapping_add(displacement);
                    }
                    _ => unreachable!()
                }
            }
            Operand::MEMORY => {
                match mode {
                    Mode::M16 => self.PC = self.resolve_mem_src_16(self.current_op[1], extra),
                    Mode::M32 => (self.PC, self.PS) = self.resolve_mem_src_32(self.current_op[1], extra),
                    _ => unreachable!(),
                }
            }
            _ => unreachable!()
        }
        self.pc_displacement = 0;
        // println!("to address: {:05X}", self.get_pc_address());
    }

    /// BRK instruction
    /// 
    /// Raises an exception with the given vector
    /// 
    /// Intel name: INT, INT3
    pub fn brk(&mut self, op: Operand) {
        let vector = match op {
            Operand::IMMEDIATE => self.get_imm8(),
            Operand::NONE => 3,
            _ => unreachable!(),
        };

        self.raise_exception(vector);
    }

    /// BRKV instruction
    /// 
    /// Raises an instruction with vector 4 if the overflow flag is set
    /// 
    /// Intel name: INTO
    pub fn brkv(&mut self) {
        if self.PSW.contains(CpuStatus::OVERFLOW) {self.raise_exception(4)}
    }

    /// CALL instruction
    /// 
    /// Unconditional jump but also pushes the PC and if 32-bit PS to the stack
    pub fn call(&mut self, op: Operand, mode: Mode, extra: u8) {
        // println!("CALL old PC = {:04X}", self.PC);
        let old_PS = self.PS;
        let old_PC = self.PC;

        self.branch_op(op, mode, extra);

        if mode == Mode::M32 {
            self.push(old_PS);
        }
        self.push(old_PC.wrapping_add(self.current_op.len() as u16));
        // println!("CALL pushed: PC = {:04X}", old_PC.wrapping_add(self.current_op.len() as u16));
        // println!("New PC = {:04X}", self.PC);
    }

    /// CHKIND instruction
    /// 
    /// Reads two consecutive little-endian words from memory and raises an exception if the interval they define does not contain the register
    /// 
    /// Intel name: BOUND
    pub fn chkind(&mut self, extra: u8) {
        let reg = self.resolve_src_16(Operand::REGISTER, extra);
        let (lo, hi) = self.resolve_mem_src_32(self.current_op[1], extra);
        if !(reg >= lo && reg < hi) {self.raise_exception(5)}
    }

    /// DISPOSE instruction
    /// 
    /// Performs the following:
    /// - SP <- BP
    /// - pop BP
    /// 
    /// Intel name: LEAVE
    pub fn dispose(&mut self) {
        self.SP = self.BP;
        self.BP = self.pop();
    }

    /// PREPARE instruction
    /// 
    /// The instruction has the following structure:
    /// 
    /// - op (1 byte)
    /// - imm16 (2 bytes)
    /// - imm5 (1 byte)
    /// 
    /// Performs the following:
    /// - push BP
    /// - temp = SP
    /// - for 1..imm5
    ///     * BP -= 2
    ///     * push \[BP\] (word)
    /// - if imm5 > 0
    ///     * push temp
    /// - BP <- temp
    /// - SP -= imm16
    /// 
    /// Intel name: ENTER
    pub fn prepare(&mut self) {
        let imm16 = u16::from_le_bytes(self.current_op[1..=2].try_into().unwrap());
        let imm5 = self.current_op[3] & 0x1F;
        self.push(self.BP);
        let temp = self.SP;
        if imm5 > 0 {
            for _ in 0..(imm5 - 1) {
                self.BP = self.BP.wrapping_sub(2);
                let addr = self.get_physical_address(self.BP, self.SS);
                let word = self.read_mem_16(addr);
                self.push(word);
            }
            self.push(temp);
        }
        self.BP = temp;
        self.SP = self.SP.wrapping_sub(imm16);
    }

    /// RETN instruction
    /// 
    /// Pops the `PC` from the stack and adds the operand to `SP`
    pub fn retn(&mut self, op: Operand) {
        // println!("RETN before PC: {:04X} PS: {:04X} SP: {:04X}", self.PC, self.PS, self.SP);
        let temp_pc = self.pop();
        let dest = match op {
            Operand::IMMEDIATE => self.get_imm16(),
            Operand::NONE => 0,
            _ => unreachable!(),
        };
        self.SP = self.SP.wrapping_add(dest);
        self.PC = temp_pc;
        self.pc_displacement = 0;
        // println!("RETN after PC: {:04X} PS: {:04X} SP: {:04X}", self.PC, self.PS, self.SP);
    }

    /// RETF instruction
    /// 
    /// Pops the `PC` and `PS` from the stack and adds the operand to `SP`
    pub fn retf(&mut self, op: Operand) {
        // println!("RETF before PC: {:04X} PS: {:04X} SP: {:04X}", self.PC, self.PS, self.SP);
        let temp_pc = self.pop();
        let temp_ps = self.pop();
        let dest = match op {
            Operand::IMMEDIATE => self.get_imm16(),
            Operand::NONE => 0,
            _ => unreachable!(),
        };
        self.SP = self.SP.wrapping_add(dest);
        self.PC = temp_pc;
        self.PS = temp_ps;
        self.pc_displacement = 0;
        // println!("RETF after PC: {:04X} PS: {:04X} SP: {:04X}", self.PC, self.PS, self.SP);
    }

    /// RETI instruction
    /// 
    /// Pops the `PC`, `PS` and `PSW` registers from the stack
    /// 
    /// Intel name: IRET
    pub fn reti(&mut self) {
        // println!("RETI before PC: {:04X} PS: {:04X}", self.PC, self.PS);
        self.PC = self.pop();
        self.PS = self.pop();
        self.PSW = CpuStatus::from_bits_truncate(self.pop());
        self.pc_displacement = 0;
        // println!("RETI after PC: {:04X} PS: {:04X}", self.PC, self.PS);
    }
}