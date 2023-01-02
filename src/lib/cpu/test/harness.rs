use crate::cpu::bus::BusDevice;
use crate::cpu::bus_vec::VecBus;
use crate::cpu::{decode, CpuState};
use crate::mem::memory::RAM;

pub(crate) struct TestHarness {
    addr: u32,
    icount: u32,
    bus: VecBus,
}

impl Default for TestHarness {
    fn default() -> Self {
        let mut bus = VecBus::default();
        let mem = Box::new(RAM::new(0x1000));

        let addr = 0x1000;
        bus.map(addr, 0x1000, mem);

        Self {
            addr,
            icount: 0,
            bus,
        }
    }
}

impl TestHarness {
    pub(crate) fn push_instr(&mut self, op: &str, d: u8, s: u8, t: u8, imm: u16, tgt: u32) {
        let instr_bin = decode::mips_encode_str(op, d, s, t, imm, tgt).unwrap();
        self.bus
            .write(self.addr + 4 * self.icount, 32, instr_bin)
            .unwrap();
        self.icount += 1;
    }

    pub(crate) fn finish(&mut self) {
        // Simulate a return and nop in delay slot
        // Don't need the return address to be valid since we only execute one block
        self.push_instr("jr", 31, 31, 31, 0, 0);
        self.push_instr("sll", 0, 0, 0, 0, 0);
    }

    pub(crate) fn load32(&mut self, reg: u8, imm: u32) {
        self.push_instr("lui", 0, 0, reg, (imm >> 16) as u16, 0);
        self.push_instr("ori", 0, reg, reg, (imm & 0xffff) as u16, 0);
    }

    pub(crate) fn execute(&mut self, state: &mut CpuState) -> Result<(), String> {
        let ctx = inkwell::context::Context::create();
        let mut tb_mgr = crate::cpu::jit::TbManager::new();

        state.set_pc(self.addr);
        let tb = tb_mgr.get_tb(&ctx, self.addr, &mut self.bus)?;
        tb.execute(state, &mut self.bus, &mut tb_mgr)?;

        self.addr = state.pc;

        Ok(())
    }

    pub(crate) fn execute_generic(
        &mut self,
        state: &mut CpuState,
        executor: Box<dyn Fn(&mut CpuState, &mut VecBus) -> Result<(), String>>,
    ) -> Result<(), String> {
        state.set_pc(self.addr);
        executor(state, &mut self.bus)?;

        self.addr = state.pc;

        Ok(())
    }
}
