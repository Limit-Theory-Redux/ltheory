use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

#[no_mangle]
pub extern "C" fn GUID_Create() -> u64 {
    NEXT_ID.fetch_add(1, Ordering::Relaxed)
}

#[no_mangle]
pub extern "C" fn GUID_Exists(id: u64) -> bool {
    id < NEXT_ID.load(Ordering::Relaxed) && id != 0
}

#[no_mangle]
pub extern "C" fn GUID_Reset() {
    NEXT_ID.store(1, Ordering::Relaxed);
}
