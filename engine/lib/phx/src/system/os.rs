pub struct Os;

#[luajit_ffi_gen::luajit_ffi(name = "OS")]
impl Os {
    pub fn get_cpu_count() -> u32 {
        num_cpus::get() as u32
    }
}
