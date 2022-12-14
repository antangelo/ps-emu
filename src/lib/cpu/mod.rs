pub mod bus;
pub mod bus_vec;
pub mod decode;
pub mod interpret;
pub mod jit;
pub mod opcode;
pub mod threaded;
pub mod trie;

#[cfg(test)]
pub mod test;

pub mod cop0 {
    pub enum Register {
        BadVaddr = 8,
        Sr = 12,
        Cause = 13,
        Epc = 14,
        Prid = 15,
    }

    #[repr(u8)]
    pub enum ExceptionCause {
        Interrupt = 0x0,
        AddressErrOnLoad = 0x4,
        AddressErrOnStore = 0x5,
        Syscall = 0x8,
        Break = 0x9,
        ReservedInstruction = 0x10,
        CopUnusable(u8) = 0x11,
        Overflow = 0x12,
    }

    impl ExceptionCause {
        pub fn to_int(&self) -> u8 {
            unsafe { *(self as *const Self as *const u8) }
        }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct CpuState {
    pub(super) gpr: [u32; 31],
    pub(super) hi: u32,
    pub(super) lo: u32,
    pub(super) pc: u32,

    // u32 here makes the offset math easier
    // and it's unlikely to save space thanks to padding anyway
    pub(super) load_delay_register: u32,
    pub(super) load_delay_register_value: u32,

    pub(super) cop0_reg: [u32; 16],
}

impl CpuState {
    pub fn set_pc(&mut self, pc: u32) {
        self.pc = pc;
    }

    pub fn get_reg_val(&self, reg: u8) -> u32 {
        if reg == 0 {
            0
        } else {
            self.gpr[(reg - 1) as usize]
        }
    }

    pub fn set_reg_val(&mut self, reg: u8, val: u32) {
        if reg == 0 {
            return;
        }

        self.gpr[(reg - 1) as usize] = val;
    }
}

impl Default for CpuState {
    fn default() -> Self {
        CpuState {
            gpr: [0; 31],
            hi: 0,
            lo: 0,
            pc: 0,
            load_delay_register: 0,
            load_delay_register_value: 0,
            cop0_reg: [0; 16],
        }
    }
}
