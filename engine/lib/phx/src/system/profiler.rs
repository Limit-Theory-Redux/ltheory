use std::sync::{LazyLock, Mutex, MutexGuard};

use indexmap::IndexMap;
use tracing::{info, warn};

use super::{Signal, Signal_AddHandlerAll, TimeStamp};
use crate::system::Signal_RemoveHandlerAll;

#[derive(Debug, Clone, Default)]
pub struct Scope {
    pub name: String,
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
    fn new(name: &str) -> Self {
        Self {
            name: name.into(),
            min: 1e30,
            max: -1e30,
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Profiler {
    pub is_enabled: bool,
    pub scopes: IndexMap<String, Scope>,
    pub stack: Vec<String>,
    pub start: TimeStamp,
}

pub static THIS: LazyLock<Mutex<Profiler>> = LazyLock::new(Default::default);

#[luajit_ffi_gen::luajit_ffi]
impl Profiler {
    pub fn enable() {
        {
            let mut profiler = THIS.lock().expect("Cannot lock profiler");

            profiler.is_enabled = true;
            profiler.scopes.clear();
            profiler.stack.clear();
            profiler.start = TimeStamp::now();
        }

        Self::begin("[Root]");

        unsafe { Signal_AddHandlerAll(Profiler_SignalHandler) };
    }

    pub fn disable() {
        let mut profiler = THIS.lock().expect("Cannot lock profiler");
        if !profiler.stack.is_empty() {
            panic!("Profiler_Disable: Cannot stop profiler from within a profiled section");
        }

        Self::end_intern(&mut profiler);

        let total = profiler.start.get_elapsed();
        let total_ms = profiler.start.get_elapsed_ms();

        for (_, scope) in &mut profiler.scopes {
            scope.var /= scope.count - 1.0f64;
            scope.var = f64::sqrt(scope.var);
        }

        // profiler.scopes.sort_by(|pa: &*mut Scope, pb: &*mut Scope| {
        //     let (a, b) = (&**pa, &**pb);

        //     if b.total < a.total {
        //         Ordering::Less
        //     } else {
        //         Ordering::Greater
        //     }
        // });

        info!("-- PHX PROFILER -------------------------------------");
        info!("-- Measured timespan: {total_ms}ms");

        if !profiler.scopes.is_empty() {
            info!("");
            info!(
                " Scope |  Cumul |    Scope |    Min |      Max |   Mean |   Var | Var/Mean | Name"
            );
            info!("-------|--------|----------|--------|----------|--------|-------|----------|---------------------------");

            let mut cumulative = 0.0;

            for (_, scope) in &profiler.scopes {
                let scopeTotal = scope.total as f64;

                cumulative += scopeTotal;

                // if scopeTotal / total > 0.01 || scope.max > 0.01 {
                info!(
                    "{:5.1}% | {:5.0}% | {:6.0}ms | {:6.2} | {:8.2} | {:6.2} | {:5.2} | {:7.0}% | {:?}",
                    100.0 * (scopeTotal / total),
                    100.0 * (cumulative / total),
                    1000.0 * scopeTotal,
                    1000.0 * scope.min,
                    1000.0 * scope.max,
                    1000.0 * scope.mean,
                    1000.0 * scope.var,
                    100.0 * (scope.var / scope.mean),
                    scope.name,
                );
                // }
            }

            info!("-------------------------------------------------------------------------------------------------------");

            profiler.scopes.clear();
        }

        profiler.is_enabled = false;

        unsafe { Signal_RemoveHandlerAll(Profiler_SignalHandler) };
    }

    pub fn begin(name: &str) {
        let mut profiler = THIS.lock().expect("Cannot lock profiler");
        if !profiler.is_enabled {
            return;
        }

        if profiler.stack.len() >= 128 {
            Self::backtrace_intern(&profiler);
            warn!("Profiler_Begin: Maximum stack depth exceeded");
            return;
        }

        let now = TimeStamp::now();
        let scope_name = profiler
            .stack
            .last()
            .map(|name| name.clone())
            .unwrap_or_default();
        if !scope_name.is_empty() {
            let prev = profiler.scopes.get_mut(&scope_name).expect("Unknown scope");

            prev.frame += prev.last.get_difference(&now);
            prev.last = now;
        }

        let curr = profiler
            .scopes
            .entry(name.into())
            .or_insert_with(|| Scope::new(name));
        curr.last = now;

        profiler.stack.push(name.into());
    }

    pub fn end() {
        let mut profiler = THIS.lock().expect("Cannot lock profiler");
        Self::end_intern(&mut profiler);
    }

    pub fn set_value(_name: &str, _value: i32) {
        // empty
    }

    pub fn loop_marker() {
        let mut profiler = THIS.lock().expect("Cannot lock profiler");
        if !profiler.is_enabled {
            return;
        }

        for (_, scope) in &mut profiler.scopes {
            if scope.frame as f64 > 0.0 {
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

    pub fn backtrace() {
        let profiler = THIS.lock().expect("Cannot lock profiler");
        Self::backtrace_intern(&profiler);
    }
}

impl Profiler {
    fn end_intern(profiler: &mut MutexGuard<Profiler>) {
        if !profiler.is_enabled {
            return;
        }

        if let Some(scope_name) = profiler.stack.pop() {
            let now = TimeStamp::now();
            let scope = profiler.scopes.get_mut(&scope_name).expect("Unknown scope");
            scope.frame += scope.last.get_difference(&now);

            let scope_name = profiler
                .stack
                .last()
                .map(|name| name.clone())
                .unwrap_or_default();

            if !scope_name.is_empty() {
                let scope = profiler.scopes.get_mut(&scope_name).expect("Unknown scope");
                scope.last = now;
            }
        } else {
            Self::backtrace_intern(profiler);
            panic!("Profiler_End: Attempting to pop an empty stack");
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

// unsafe extern "C" fn Profiler_GetScope(name: *const libc::c_char) -> *mut Scope {
//     let mut scope: *mut Scope = HashMap_GetRaw(THIS.map, name as usize as u64) as *mut Scope;
//     if !scope.is_null() {
//         return scope;
//     }
//     scope = Scope_Create(name);
//     HashMap_SetRaw(THIS.map, name as usize as u64, scope as *mut _);
//     scope
// }

extern "C" fn Profiler_SignalHandler(_: Signal) {
    Profiler::backtrace();
}
