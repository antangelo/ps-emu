pub struct JitState<'ctx> {
    context: &'ctx inkwell::context::Context,
    module: inkwell::module::Module<'ctx>,
    builder: inkwell::builder::Builder<'ctx>,
    execution_engine: inkwell::execution_engine::ExecutionEngine<'ctx>,
}

type TbDynFunc = unsafe extern "C" fn(state: *mut CpuState, bus: *mut super::bus::Bus);

// TODO: Find way to free old TBs
impl<'ctx> JitState<'ctx> {
    pub fn new<'a>(ctx: &'ctx inkwell::context::Context) -> Self {
        let module = ctx.create_module("tb_module");
        let execution_engine = module.create_jit_execution_engine(inkwell::OptimizationLevel::None).unwrap();

        // FIXME: Split callback/type registration into its own function
        let i32_type = ctx.i32_type();
        let bus_type = ctx.opaque_struct_type("mips_bus").ptr_type(inkwell::AddressSpace::Generic);
        let bool_type = ctx.bool_type();

        let read_fn = module.add_function("tb_mem_read", i32_type.fn_type(&[bus_type.into(), i32_type.into(), i32_type.into()], false), None);
        let write_fn = module.add_function("tb_mem_write", bool_type.fn_type(&[bus_type.into(), i32_type.into(), i32_type.into(), i32_type.into()], false), None);

        execution_engine.add_global_mapping(&read_fn, tb_mem_read as usize);
        execution_engine.add_global_mapping(&write_fn, tb_mem_write as usize);

        let state_type = ctx.opaque_struct_type("mips_state");
        state_type.set_body(&[i32_type.into(); 33], false);

        JitState{context: ctx, module, builder: ctx.create_builder(), execution_engine}
    }

    fn translate(&self) -> Option<inkwell::execution_engine::JitFunction<TbDynFunc>> {
        let void_type = self.context.void_type();
        let state_type = self.module.get_struct_type("mips_state").unwrap().ptr_type(inkwell::AddressSpace::Generic);
        let bus_type = self.module.get_struct_type("mips_bus").unwrap().ptr_type(inkwell::AddressSpace::Generic);
        let fn_type = void_type.fn_type(&[state_type.into(), bus_type.into()], false);

        // FIXME: Generate unique tb name
        let tb_name = format!("tb_name");
        let func = self.module.add_function(&tb_name, fn_type, None);
        let fn_block = self.context.append_basic_block(func, "entry");
        self.builder.position_at_end(fn_block);

        let state_arg = func.get_nth_param(0)?.into_pointer_value();
        let bus_arg = func.get_nth_param(1)?.into_pointer_value();

        let i32_type = self.context.i32_type();
        //let addr_var = self.builder.build_alloca(i32_type, "addr");
        let addr_const = i32_type.const_int(0xffff, false);
        //self.builder.build_store(addr_var, addr_const);

        //let size_var = self.builder.build_alloca(i32_type, "size");
        let size_const = i32_type.const_int(32, false);
        //self.builder.build_store(size_var, size_const);

        let read_fn = self.module.get_function("tb_mem_read").unwrap();
        let mem_read = self.builder.build_call(read_fn, &[bus_arg.into(), addr_const.into(), size_const.into()], "read_call");

        let reg_1 = self.builder.build_struct_gep(state_arg, 0, "state_r1").unwrap();

        self.builder.build_store(reg_1, mem_read.try_as_basic_value().left().unwrap());

        self.builder.build_return(None);

        self.module.print_to_stderr();

        unsafe {
            self.execution_engine.get_function(&tb_name).ok()
        }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct CpuState {
    gpr: [u32; 31],
    hi: u32,
    lo: u32,
}

impl Default for CpuState {
    fn default() -> Self {
        CpuState{gpr: [0;31], hi: 0, lo: 0}
    }
}

#[no_mangle]
pub unsafe extern "C" fn tb_mem_read(bus: *mut super::bus::Bus, addr: u32, size: u32) -> u32 {
    assert_ne!(std::ptr::null_mut(), bus);
    (*bus).read(addr, size).unwrap() // FIXME
}

#[no_mangle]
pub unsafe extern "C" fn tb_mem_write(bus: *mut super::bus::Bus, addr: u32, size: u32, value: u32) -> bool {
    assert_ne!(std::ptr::null_mut(), bus);
    match (*bus).write(addr, size, value) {
        Ok(_) => true,
        Err(_) => false,
    }
}

pub fn execute(_instr: super::decode::MipsInstr, bus: &mut super::bus::Bus, state: &mut CpuState) -> Result<(), ()> {
    let context = inkwell::context::Context::create();
    let jit_state = JitState::new(&context);
    let func = jit_state.translate().unwrap();

    bus.write(0xffff, 32, 24).unwrap();

    unsafe {
        println!("About to execute");
        func.call(core::ptr::addr_of_mut!(*state), core::ptr::addr_of_mut!(*bus));
    }

    println!("CpuState: {:?}", state);

    Ok(())
}

#[cfg(test)]
mod test {
    #[test]
    #[should_panic]
    fn test_tb_write_null() {
        unsafe {
            super::tb_mem_write(core::ptr::null_mut(), 0, 0, 0);
        }
    }

    #[test]
    #[should_panic]
    fn test_tb_read_null() {
        unsafe {
            super::tb_mem_read(core::ptr::null_mut(), 0, 0);
        }
    }
}

