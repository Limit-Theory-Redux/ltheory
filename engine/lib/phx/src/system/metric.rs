use std::sync::{LazyLock, Mutex};

#[luajit_ffi_gen::luajit_ffi(with_impl = true)]
#[derive(Debug, Clone, Copy)]
pub enum Metric {
    None,
    DrawCalls,
    Immediate,
    PolysDrawn,
    TrisDrawn,
    VertsDrawn,
    Flush,
    FBOSwap,
    SIZE,
}

pub static VALUE_CURR: LazyLock<Mutex<[u64; Metric::SIZE as usize]>> =
    LazyLock::new(Default::default);

#[luajit_ffi_gen::luajit_ffi]
impl Metric {
    pub fn get(this: Self) -> u64 {
        let value_curr = VALUE_CURR.lock().expect("Cannot lock metric values");
        value_curr[this as usize]
    }

    pub fn get_name(this: Self) -> Option<&'static str> {
        match this {
            Self::DrawCalls => Some("Draw Calls"),
            Self::Immediate => Some("Draw Calls (Immediate)"),
            Self::PolysDrawn => Some("Polys"),
            Self::TrisDrawn => Some("Tris"),
            Self::VertsDrawn => Some("Vertices"),
            Self::Flush => Some("Pipeline Flushes"),
            Self::FBOSwap => Some("Framebuffer Swaps"),
            _ => None,
        }
    }
}

impl Metric {
    pub fn add_draw(polys: u64, tris: u64, verts: u64) {
        let mut value_curr = VALUE_CURR.lock().expect("Cannot lock metric values");
        value_curr[Self::DrawCalls as usize] += 1;
        value_curr[Self::PolysDrawn as usize] += polys as u64;
        value_curr[Self::TrisDrawn as usize] += tris as u64;
        value_curr[Self::VertsDrawn as usize] += verts as u64;
    }

    pub fn add_draw_imm(polys: u64, tris: u64, verts: u64) {
        let mut value_curr = VALUE_CURR.lock().expect("Cannot lock metric values");
        value_curr[Self::Immediate as usize] += 1;
        value_curr[Self::PolysDrawn as usize] += polys as u64;
        value_curr[Self::TrisDrawn as usize] += tris as u64;
        value_curr[Self::VertsDrawn as usize] += verts as u64;
    }

    pub fn inc(self) {
        let mut value_curr = VALUE_CURR.lock().expect("Cannot lock metric values");
        value_curr[self as usize] += 1;
    }

    pub fn inc_delta(self, delta: u64) {
        let mut value_curr = VALUE_CURR.lock().expect("Cannot lock metric values");
        value_curr[self as usize] += delta;
    }

    pub fn reset() {
        let mut value_curr = VALUE_CURR.lock().expect("Cannot lock metric values");
        value_curr.iter_mut().for_each(|v| *v = 0);
    }
}
