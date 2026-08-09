use std::cmp::Ordering;
use std::sync::atomic::{AtomicBool, Ordering as AtomicOrdering};
use std::sync::{LazyLock, Mutex, MutexGuard};

use cli_table::{Cell, Style, Table};
use indexmap::IndexMap;
use tracing::{error, info, warn};

use super::{Signal, TimeStamp, signal_add_handler_all, signal_remove_handler_all};

const MAX_SCOPE_STACK_SIZE: usize = 128;

#[derive(Debug, Clone, Default)]
pub struct Scope {
    pub name: String,
    /// Name of the scope that was active when this scope began ("" for a
    /// top-level scope). Enables flame-graph rendering from the snapshot.
    pub parent: String,
    pub last: TimeStamp,
    pub frame: f64,
    pub total: f64,
    pub count: f64,
    pub mean: f64,
    pub var: f64,
    pub min: f64,
    pub max: f64,
}

impl Scope {
    fn new(name: &str, parent: &str) -> Self {
        Self {
            name: name.into(),
            parent: parent.into(),
            min: 1e30,
            max: -1e30,
            ..Default::default()
        }
    }
}

/// The Profiler tracks timing information for different code scopes
///
/// The Profiler provides a signal handler that can be used to track
/// backtraces of active profiling scopes when certain signals occur
///
/// # Usage:
/// - Use `Profiler::begin()` and `Profiler::end()` to create and close scopes
/// - Call `loop_marker()` periodically within loops to record timing data
/// - Profiler automatically tracks metrics like total time, frame variance,
///   min/max frame times for each scope
#[derive(Debug, Clone, Default)]
pub struct Profiler {
    /// Whether profiling is currently enabled
    pub is_enabled: bool,

    /// Map of named scopes tracking their performance metrics.
    /// Each scope records total time, frame times, count,
    /// mean, variance, min and max frame durations
    pub scopes: IndexMap<String, Scope>,

    /// Stack of active scopes in call hierarchy
    pub stack: Vec<String>,

    /// Timestamp when profiler was started
    pub start: TimeStamp,
}

pub static PROFILER: LazyLock<Mutex<Profiler>> = LazyLock::new(Default::default);

/// Cheap gate mirroring `Profiler::is_enabled`, read without the global mutex.
/// `begin`/`end`/`loop_marker` are called from hot render paths (per glyph,
/// per draw, per shader start) and previously locked the mutex even when the
/// profiler was disabled; this atomic makes the disabled path lock-free.
static PROFILER_ENABLED: AtomicBool = AtomicBool::new(false);

/// Set by the stats dashboard (HTTP thread) when the user clicks the
/// profile toggle. The main thread picks it up at the next safe point
/// (Application:onPreRender, same spot as the F10 binding) and calls
/// `enable()`/`disable()` there - never from the HTTP thread, which could
/// hit `disable()` mid-scope and panic.
static PROFILER_TOGGLE_REQUEST: AtomicBool = AtomicBool::new(false);

/// A single profiler scope's live metrics, ready for serialization.
/// No mutating normalization is applied (unlike `disable()`'s print path).
#[derive(Debug, Clone)]
pub struct ScopeSnapshot {
    pub name: String,
    /// Parent scope name ("" for top-level). Drives flame-graph rendering.
    pub parent: String,
    pub scope_pct: f64,
    pub cumul_pct: f64,
    pub total_ms: f64,
    pub min_ms: f64,
    pub max_ms: f64,
    pub mean_ms: f64,
}

#[luajit_ffi_gen::luajit_ffi]
impl Profiler {
    /// True if profiling is currently enabled (lock-free read).
    pub fn is_enabled() -> bool {
        PROFILER_ENABLED.load(AtomicOrdering::Acquire)
    }

    /// Request a profiling toggle. The actual enable/disable happens on the
    /// main thread at the next safe point (`Profiler::pending_toggle`).
    pub fn request_toggle() {
        PROFILER_TOGGLE_REQUEST.store(true, AtomicOrdering::Release);
    }

    /// Check-and-clear the toggle request. Called once per frame from the
    /// main thread's safe point (Application:onPreRender) - returns true
    /// exactly once per dashboard click, mirroring the F10 path.
    pub fn pending_toggle() -> bool {
        PROFILER_TOGGLE_REQUEST.swap(false, AtomicOrdering::AcqRel)
    }

    /// Enables profiling and initializes the profiler state
    pub fn enable() {
        {
            let mut profiler = PROFILER.lock().expect("Cannot lock profiler");

            profiler.is_enabled = true;
            profiler.scopes.clear();
            profiler.stack.clear();
            profiler.start = TimeStamp::now();
        }
        PROFILER_ENABLED.store(true, AtomicOrdering::Release);

        Self::begin("[Root]");

        signal_add_handler_all(profiler_signal_handler);
    }

    /// Disables profiling and processes results
    pub fn disable() {
        PROFILER_ENABLED.store(false, AtomicOrdering::Release);
        let mut profiler = PROFILER.lock().expect("Cannot lock profiler");
        if profiler.stack.len() > 1 {
            panic!(
                "Profiler::disable: Cannot stop profiler from within a profiled section. Active scope(s): {:?}",
                profiler.stack
            );
        }

        Self::end_intern(&mut profiler, true);

        let total = profiler.start.get_elapsed();
        let total_ms = profiler.start.get_elapsed_ms();

        for (_, scope) in &mut profiler.scopes {
            scope.var /= scope.count - 1.0f64;
            scope.var = f64::sqrt(scope.var);
        }

        profiler.scopes.sort_by(|_, s1, _, s2| {
            if s2.total < s1.total {
                Ordering::Less
            } else {
                Ordering::Greater
            }
        });

        info!("-- PHX PROFILER -------------------------------------");
        info!("-- Measured timespan: {total_ms}ms");
        info!("-- Measured scopes: {}", profiler.scopes.len());

        if !profiler.scopes.is_empty() {
            let mut table = vec![];
            let mut cumulative = 0.0;

            for (_, scope) in &profiler.scopes {
                let scope_total = scope.total;

                cumulative += scope_total;

                if scope_total / total > 0.01 || scope.max > 0.01 {
                    table.push(vec![
                        scope.name.clone().cell(),
                        format!("{:5.1}", 100.0 * (scope_total / total)).cell(),
                        format!("{:5.0}", 100.0 * (cumulative / total)).cell(),
                        format!("{:6.0}", 1000.0 * scope_total).cell(),
                        format!("{:6.2}", 1000.0 * scope.min).cell(),
                        format!("{:8.2}", 1000.0 * scope.max).cell(),
                        format!("{:6.2}", 1000.0 * scope.mean).cell(),
                        format!("{:5.2}", 1000.0 * scope.var).cell(),
                        format!("{:7.0}", 100.0 * (scope.var / scope.mean)).cell(),
                    ]);
                }
            }

            let table = table.table().title(vec![
                "Name".cell().bold(true),
                "Scope %".cell().bold(true),
                "Cumul %".cell().bold(true),
                "Scope (ms)".cell().bold(true),
                "Min".cell().bold(true),
                "Max".cell().bold(true),
                "Mean".cell().bold(true),
                "Var".cell().bold(true),
                "Var/Mean %".cell().bold(true),
            ]);

            info!("\n{}", table.display().expect("No profiler table"));

            profiler.scopes.clear();
        }

        profiler.is_enabled = false;

        signal_remove_handler_all(profiler_signal_handler);
    }

    /// Starts a new profiling scope
    pub fn begin(name: &str) {
        if !PROFILER_ENABLED.load(AtomicOrdering::Acquire) {
            return;
        }
        let mut profiler = PROFILER.lock().expect("Cannot lock profiler");
        if !profiler.is_enabled {
            return;
        }

        if profiler.stack.len() >= MAX_SCOPE_STACK_SIZE {
            Self::backtrace_intern(&profiler);
            warn!("Profiler::begin: Maximum stack depth exceeded");
            return;
        }

        let now = TimeStamp::now();
        let scope_name = profiler.stack.last().cloned().unwrap_or_default();
        if !scope_name.is_empty() {
            let prev = profiler.scopes.get_mut(&scope_name).expect("Unknown scope");

            prev.frame += prev.last.get_difference(&now);
            prev.last = now;
        }

        let parent = profiler.stack.last().cloned().unwrap_or_default();
        let curr = profiler
            .scopes
            .entry(name.into())
            .or_insert_with(|| Scope::new(name, &parent));
        curr.last = now;

        profiler.stack.push(name.into());
    }

    /// Ends the current profiling scope
    pub fn end() {
        if !PROFILER_ENABLED.load(AtomicOrdering::Acquire) {
            return;
        }
        let mut profiler = PROFILER.lock().expect("Cannot lock profiler");
        Self::end_intern(&mut profiler, false);
    }

    pub fn set_value(_name: &str, _value: i32) {
        // empty
    }

    /// Records frame timing for each active scope
    pub fn loop_marker() {
        if !PROFILER_ENABLED.load(AtomicOrdering::Acquire) {
            return;
        }
        let mut profiler = PROFILER.lock().expect("Cannot lock profiler");
        if !profiler.is_enabled {
            return;
        }

        for (_, scope) in &mut profiler.scopes {
            if scope.frame > 0.0 {
                scope.total += scope.frame;
                let frame = scope.frame;
                scope.min = f64::min(scope.min, frame);
                scope.max = f64::max(scope.max, frame);
                scope.count += 1.0;
                let d1 = frame - scope.mean;
                scope.mean += d1 / scope.count;
                let d2 = frame - scope.mean;
                scope.var += d1 * d2;
                scope.frame = 0.0;
            }
        }
    }

    /// Prints backtrace of active scopes
    pub fn backtrace() {
        let profiler = PROFILER.lock().expect("Cannot lock profiler");
        Self::backtrace_intern(&profiler);
    }
}

impl Profiler {
    /// Live scope metrics for the dashboard. Same filtering and ordering as
    /// the F10 print path (drop scopes under 1% with no >10ms spike), but
    /// read-only: nothing is cleared, normalized, or printed.
    pub fn snapshot() -> Vec<ScopeSnapshot> {
        let profiler = PROFILER.lock().expect("Cannot lock profiler");
        if !profiler.is_enabled {
            return Vec::new();
        }

        let total = profiler.start.get_elapsed();

        // Sort by total descending, mirroring disable()'s print path.
        let mut scopes: Vec<(&String, &Scope)> = profiler.scopes.iter().collect();
        scopes.sort_by(|(_, a), (_, b)| {
            if b.total < a.total {
                Ordering::Less
            } else {
                Ordering::Greater
            }
        });

        let mut rows = Vec::new();
        let mut cumulative = 0.0;

        for (name, scope) in scopes {
            let scope_total = scope.total;
            cumulative += scope_total;

            if scope_total / total > 0.01 || scope.max > 0.01 {
                rows.push(ScopeSnapshot {
                    name: name.clone(),
                    parent: scope.parent.clone(),
                    scope_pct: 100.0 * (scope_total / total),
                    cumul_pct: 100.0 * (cumulative / total),
                    total_ms: 1000.0 * scope_total,
                    min_ms: 1000.0 * scope.min,
                    max_ms: 1000.0 * scope.max,
                    mean_ms: 1000.0 * scope.mean,
                });
            }
        }

        rows
    }

    fn end_intern(profiler: &mut MutexGuard<Profiler>, is_root: bool) {
        if !profiler.is_enabled {
            return;
        }

        if !is_root && profiler.stack.len() == 1 {
            error!("Attempting to remove profiler root scope");
            return;
        }

        if let Some(scope_name) = profiler.stack.pop() {
            let now = TimeStamp::now();
            let scope = profiler.scopes.get_mut(&scope_name).expect("Unknown scope");
            scope.frame += scope.last.get_difference(&now);

            let scope_name = profiler.stack.last().cloned().unwrap_or_default();

            if !scope_name.is_empty() {
                let scope = profiler.scopes.get_mut(&scope_name).expect("Unknown scope");
                scope.last = now;
            }
        } else {
            Self::backtrace_intern(profiler);
            error!("Profiler::end: Attempting to pop an empty stack. Profiler::begin is missing.");
        }
    }

    fn backtrace_intern(profiler: &MutexGuard<Profiler>) {
        if !profiler.is_enabled {
            return;
        }

        info!("PHX Profiler Backtrace:");

        profiler
            .stack
            .iter()
            .enumerate()
            .rev()
            .for_each(|(index, scope_name)| {
                let scope = profiler.scopes.get(scope_name).expect("Unknown scope");
                info!("  [{index}] {:?}", scope.name)
            });
    }
}

extern "C" fn profiler_signal_handler(_: Signal) {
    Profiler::backtrace();
}
