use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

pub struct Guid;

#[luajit_ffi_gen::luajit_ffi(name = "GUID")]
impl Guid {
    pub fn create() -> u64 {
        NEXT_ID.fetch_add(1, Ordering::Relaxed)
    }

    pub fn exists(id: u64) -> bool {
        id < NEXT_ID.load(Ordering::Relaxed) && id != 0
    }

    pub fn reset() {
        NEXT_ID.store(1, Ordering::Relaxed);
    }
}
