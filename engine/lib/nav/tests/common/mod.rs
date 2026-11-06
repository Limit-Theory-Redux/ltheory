//! Harness that loads a Lua kernel from the repository into a LuaJIT state, for tests.
//!
//! The state has no engine globals, and `_G` is locked before the kernel loads: reading an
//! undefined global or writing a new one raises an error, so a kernel that depends on a global
//! fails its test. Buffers cross the boundary as `double[?]` cdata, the layout kernels use.

use std::path::Path;

use mlua::{Function, Lua, Result, Table, Value};

/// Builds a `double[?]` from a Lua array, indexed from 1 in the table and from 0 in the buffer.
const TO_BUFFER: &str = "
    local ffi = require('ffi')
    return function(t)
        local n = #t
        local buffer = ffi.new('double[?]', n)
        for i = 1, n do buffer[i - 1] = t[i] end
        return buffer
    end
";

/// Copies the first `n` elements of a `double[?]` into a Lua array.
const FROM_BUFFER: &str = "
    return function(buffer, n)
        local t = {}
        for i = 1, n do t[i] = buffer[i - 1] end
        return t
    end
";

/// Makes `_G` raise on a read of an undefined name and on a write of a new one.
const LOCK_GLOBALS: &str = "
    setmetatable(_G, {
        __index = function(_, name)
            error('undefined global read: ' .. tostring(name), 2)
        end,
        __newindex = function(_, name)
            error('global write: ' .. tostring(name), 2)
        end,
    })
";

/// The repository's `script/` directory, where kernels live.
fn script_dir() -> String {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../script")
        .to_string_lossy()
        .replace('\\', "/")
}

/// A LuaJIT state with `require` paths set and the globals locked.
pub struct KernelVm {
    lua: Lua,
    to_buffer: Function,
    from_buffer: Function,
}

impl KernelVm {
    /// Creates a state that finds modules under `script/`, and under `extra_dir` when given.
    pub fn new(extra_dir: Option<&str>) -> Result<Self> {
        // SAFETY: the unsafe constructor is needed for LuaJIT's `ffi` library, which kernel
        // buffers use. The state belongs to this value, is used from one thread, and loads only
        // Lua files of this repository, so no foreign code runs through the `ffi` library.
        #[allow(unsafe_code)]
        let lua = unsafe { Lua::unsafe_new() };

        let mut path = format!("{0}/?.lua;{0}/?/init.lua", script_dir());
        if let Some(dir) = extra_dir {
            path = format!("{dir}/?.lua;{path}");
        }
        lua.globals().get::<Table>("package")?.set("path", path)?;

        let to_buffer = lua.load(TO_BUFFER).eval()?;
        let from_buffer = lua.load(FROM_BUFFER).eval()?;
        lua.load(LOCK_GLOBALS).exec()?;

        Ok(Self {
            lua,
            to_buffer,
            from_buffer,
        })
    }

    /// Loads a module by its `require` name and returns what it returns.
    pub fn require(&self, module: &str) -> Result<Table> {
        self.lua.globals().get::<Function>("require")?.call(module)
    }

    /// Makes a `double[?]` holding `data`.
    pub fn buffer(&self, data: &[f64]) -> Result<Value> {
        self.to_buffer.call(data.to_vec())
    }

    /// Reads the first `len` elements of a `double[?]`.
    pub fn read_buffer(&self, buffer: &Value, len: usize) -> Result<Vec<f64>> {
        self.from_buffer.call((buffer, len))
    }
}
