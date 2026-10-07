use std::path::PathBuf;
use std::time::Duration;

use crossbeam::channel::{Receiver, RecvTimeoutError, Sender};
use mlua::{Function, Lua, Value};
use tracing::{debug, error};

use super::{TaskQueueError, WorkerInData, WorkerOutData};
use crate::engine::Payload;

/// Output of a Lua worker task: a payload, nothing (the `Run` function returned `nil`) or an error
/// message (with traceback) when `Run` raised an error.
pub type LuaTaskOutput = Result<Option<Box<Payload>>, String>;

/// Lua side wrapper: runs `Run` under xpcall so that errors carry a traceback and never unwind
/// into (and kill) the worker thread.
const RUN_WRAPPER: &str = r#"
local run = ...
return function(p)
    return xpcall(run, debug.traceback, p)
end
"#;

/// Builds the body of one Lua worker instance thread.
/// `setup` runs on the fresh Lua state before the script is loaded (used by tests to inject helpers).
pub fn lua_worker_body(
    worker_name: String,
    script_path: PathBuf,
    setup: impl Fn(&Lua) -> mlua::Result<()> + Send + Sync + 'static,
) -> impl Fn(
    Receiver<WorkerInData<Payload>>,
    Sender<WorkerOutData<LuaTaskOutput>>,
) -> Result<(), TaskQueueError>
+ Send
+ Sync
+ 'static {
    move |in_receiver, out_sender| {
        debug!("Starting instance of Lua worker: {worker_name:?}");

        #[allow(unsafe_code)] // TODO: remove
        let lua = unsafe { Lua::unsafe_new() };

        setup(&lua)?;
        lua.load(script_path.as_path()).exec()?;

        let run_func: Function = lua.globals().get("Run")?;
        let safe_run: Function = lua.load(RUN_WRAPPER).call(run_func)?;

        loop {
            match in_receiver.recv_timeout(Duration::from_millis(500)) {
                Ok(in_data) => {
                    let data = match in_data {
                        WorkerInData::Ping => WorkerOutData::Pong,
                        WorkerInData::Data(task_id, data) => {
                            WorkerOutData::Data(task_id, run_task(&safe_run, data))
                        }
                        WorkerInData::Stop => {
                            debug!("Worker {worker_name:?} received stop signal");
                            break;
                        }
                    };

                    if out_sender.send(data).is_err() {
                        error!("Cannot send response. Worker: {worker_name}");
                        break;
                    }
                }
                Err(RecvTimeoutError::Timeout) => continue,
                Err(RecvTimeoutError::Disconnected) => break,
            }
        }

        debug!("Lua worker {worker_name:?} instance stopped");
        Ok(())
    }
}

fn run_task(safe_run: &Function, data: Payload) -> LuaTaskOutput {
    // Ownership of the input payload moves to the script (it frees it).
    let in_ptr = Box::into_raw(Box::new(data)) as usize;

    let (ok, value): (bool, Value) = safe_run
        .call(in_ptr)
        .map_err(|e| format!("Worker call failed: {e}"))?;

    if !ok {
        return Err(match value {
            Value::String(s) => s.to_string_lossy().to_string(),
            other => format!("{other:?}"),
        });
    }

    let ptr = match value {
        Value::Nil => return Ok(None),
        Value::Integer(i) => i as usize,
        Value::Number(n) => n as usize,
        other => {
            return Err(format!(
                "Worker function returned an unsupported value of type {}",
                other.type_name()
            ));
        }
    };
    if ptr == 0 {
        return Ok(None);
    }

    // Ownership of the output payload moves from the script to the engine.
    #[allow(unsafe_code)] // TODO: remove
    Ok(Some(unsafe { Box::from_raw(ptr as *mut Payload) }))
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::*;
    use crate::engine::{Worker, WorkerBase};

    fn script(name: &str, body: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "ltr_lua_worker_{name}_{}.lua",
            std::process::id()
        ));
        std::fs::File::create(&path)
            .unwrap()
            .write_all(body.as_bytes())
            .unwrap();
        path
    }

    /// `mk_payload(n)` builds an I64 payload, `take_payload(ptr)` consumes one and returns its value.
    fn setup(lua: &Lua) -> mlua::Result<()> {
        let mk = lua
            .create_function(|_, n: i64| Ok(Box::into_raw(Box::new(Payload::I64(n))) as usize))?;
        let take = lua.create_function(|_, p: usize| {
            #[allow(unsafe_code)]
            let b = unsafe { Box::from_raw(p as *mut Payload) };
            match *b {
                Payload::I64(v) => Ok(v),
                _ => Ok(-1),
            }
        })?;
        lua.globals().set("mk_payload", mk)?;
        lua.globals().set("take_payload", take)?;
        Ok(())
    }

    const SCRIPT: &str = r#"
function Run(p)
    local v = take_payload(p)
    if v == 1 then error("boom") end
    if v == 2 then return nil end
    return mk_payload(v * 10)
end
"#;

    fn recv(worker: &mut Worker<Payload, LuaTaskOutput>) -> (usize, LuaTaskOutput) {
        worker
            .recv_timeout(Duration::from_secs(10))
            .unwrap()
            .expect("no result in time")
    }

    #[test]
    fn lua_error_and_nil_do_not_kill_worker() {
        let path = script("err", SCRIPT);
        let mut worker = Worker::new(
            "TestLua",
            1,
            lua_worker_body("TestLua".into(), path, setup),
        );

        let ids: Vec<_> = [3, 1, 2, 4]
            .iter()
            .map(|v| worker.send(Payload::I64(*v)).unwrap())
            .collect();

        let (id, r) = recv(&mut worker);
        assert_eq!(id, ids[0]);
        assert_eq!(*r.unwrap().unwrap(), Payload::I64(30));

        let (id, r) = recv(&mut worker);
        assert_eq!(id, ids[1]);
        let msg = r.unwrap_err();
        assert!(msg.contains("boom"), "{msg}");
        assert!(msg.contains("stack traceback"), "{msg}");

        let (id, r) = recv(&mut worker);
        assert_eq!(id, ids[2]);
        assert!(r.unwrap().is_none());

        // the worker survived both
        let (id, r) = recv(&mut worker);
        assert_eq!(id, ids[3]);
        assert_eq!(*r.unwrap().unwrap(), Payload::I64(40));
        assert!(!worker.is_finished());
        assert_eq!(0, worker.tasks_in_work());

        worker.stop().unwrap();
    }

    #[test]
    fn payload_array_bulk_access() {
        let p = Payload::from_f64_array(&[1.0, 2.5, -3.0]);
        assert_eq!(3, p.array_len());
        let addr = p.get_f64_array_addr();
        #[allow(unsafe_code)]
        let slice = unsafe { std::slice::from_raw_parts(addr as *const f64, 3) };
        assert_eq!(&[1.0, 2.5, -3.0], slice);

        let p = Payload::from_string_array(&["a", "bb"]);
        assert_eq!(2, p.array_len());
        assert_eq!("bb", p.get_string_array_item(1));

        let p = Payload::from_bool_array(&[true, false]);
        #[allow(unsafe_code)]
        let first = unsafe { *(p.get_bool_array_addr() as *const u8) };
        assert_eq!(1, first);

        assert_eq!(0, Payload::from_i64(3).array_len());
        assert_eq!(0, Payload::from_i64_array(&[]).array_len());
    }

    // The FFI (ffi.C -> libphx) is not available in unit tests, so only the pure-Lua part of
    // PayloadConverter is tested here; the FFI round trip is covered by the WorkerTest state.
    fn kind(code: &str) -> (Option<String>, Option<i64>) {
        let lua = Lua::new();
        lua.load("rawtype = type").exec().unwrap();
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../script/Core/Util/PayloadConverter.lua");
        let pc: mlua::Table = lua.load(path.as_path()).eval().unwrap();
        let f: mlua::Function = pc.get("SequenceKind").unwrap();
        let t: mlua::Table = lua.load(format!("return {code}")).eval().unwrap();
        f.call(t).unwrap()
    }

    #[test]
    fn sequence_detection() {
        assert_eq!((Some("number".into()), Some(3)), kind("{1, 2.5, 3}"));
        assert_eq!((Some("boolean".into()), Some(2)), kind("{true, false}"));
        assert_eq!((Some("string".into()), Some(2)), kind("{'a', 'b'}"));
        // later elements are checked, not only element 1
        assert_eq!((Some("mixed".into()), Some(3)), kind("{1, 'two', 3}"));
        // not sequences
        assert_eq!((None, None), kind("{}"));
        assert_eq!((None, None), kind("{a = 1}"));
        assert_eq!((None, None), kind("{1, 2, x = 3}"));
        assert_eq!((None, None), kind("{[2] = 5}"));
        assert_eq!((None, None), kind("{[1] = 1, [2] = 2, [4] = 4}"));
    }
}
