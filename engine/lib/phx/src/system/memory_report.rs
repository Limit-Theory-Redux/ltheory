//! Lua-side memory analysis report (feature `stats-server`).
//!
//! The Lua state (a `MemoryReporter` module) walks its object census once
//! per second: entities by class, rigid bodies, meshes, textures, asteroid
//! data arrays, spawned asteroid entities, etc. It pushes each category as
//! a (name, count, bytes) row into a global store. The stats dashboard
//! serves the rows via `/memory.json` so a growing category shows up as a
//! rising byte count over time - i.e. what a memory-analysis tool should
//! show, not just a single process RSS number.
//!
//! The store is plain data written from Lua and read by the dashboard
//! thread; it never touches GL, the render thread, or the physics world.

use std::sync::{LazyLock, Mutex};

/// One memory category: Lua object census row.
#[derive(Debug, Clone, Default)]
pub struct MemoryEntry {
    pub category: String,
    pub count: u64,
    pub bytes: u64,
}

#[derive(Debug, Default)]
struct MemoryReportStore {
    entries: Vec<MemoryEntry>,
    /// Monotonic report sequence (bumped by BeginFrame)
    frame: u64,
}

static MEMORY_REPORT: LazyLock<Mutex<MemoryReportStore>> = LazyLock::new(Default::default);

/// The Lua-visible reporter. Static methods are bound to Lua as
/// `MemoryReport_<Method>` by luajit-ffi-gen (attribute on the impl).
pub struct MemoryReport;

#[luajit_ffi_gen::luajit_ffi]
impl MemoryReport {
    /// Clear the report store for a new sampling round. Called once per
    /// sampling interval from Lua (NOT per render frame).
    pub fn begin_frame() {
        if let Ok(mut store) = MEMORY_REPORT.lock() {
            store.entries.clear();
            store.frame = store.frame.wrapping_add(1);
        }
    }

    /// Add one memory category row. Bytes are the Lua-side estimate for
    /// the category (e.g. `#asteroids * sizeof(struct)` for SoA buffers).
    pub fn add(category: &str, count: u64, bytes: u64) {
        if let Ok(mut store) = MEMORY_REPORT.lock() {
            store.entries.push(MemoryEntry {
                category: category.to_string(),
                count,
                bytes,
            });
        }
    }
}

/// Snapshot of the current report (called by the dashboard thread).
pub fn snapshot() -> (u64, Vec<MemoryEntry>) {
    match MEMORY_REPORT.lock() {
        Ok(store) => (store.frame, store.entries.clone()),
        Err(_) => (0, Vec::new()),
    }
}
