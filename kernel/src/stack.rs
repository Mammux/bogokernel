const KTRAP_STACK_SIZE: usize = 4096;

#[repr(align(16))]
struct Trapstack([u8; KTRAP_STACK_SIZE]);
static mut KTRAP_STACK: Trapstack = Trapstack([0; KTRAP_STACK_SIZE]);

pub fn init_trap_stack() {
    unsafe {
        let top = (&raw const KTRAP_STACK.0 as *const u8 as usize) + KTRAP_STACK_SIZE;
        riscv::register::sscratch::write(top);
    }
}
