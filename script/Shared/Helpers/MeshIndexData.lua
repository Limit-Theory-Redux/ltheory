---Reads a mesh's index data without leaking memory.
---
---Mesh:lockIndexData takes a C function pointer. A Lua function passed there
---is turned into an FFI callback that is never freed, and it keeps everything
---its closure holds alive (vertex tables, result tables) for the rest of the
---run. Here one permanent callback serves every read and forgets the reader
---afterwards.
local ffi = require("ffi")

local MeshIndexData = {}

local reader
local readError

local callback = ffi.cast("void (*)(int*, uint64)", function(indices, count)
    -- No Lua error may unwind through the engine's C frames: keep it for later
    local ok, err = pcall(reader, indices, count)
    if not ok then
        readError = err
    end
end)

---Call `fn(indices, count)` with the mesh's index array (0-based `int*`),
---which is only valid during the call.
---@param mesh Mesh
---@param fn fun(indices: ffi.cdata*, count: integer)
function MeshIndexData.read(mesh, fn)
    assert(reader == nil, "MeshIndexData.read calls cannot be nested")
    reader = fn
    readError = nil
    mesh:lockIndexData(callback)
    reader = nil
    if readError then
        local err = readError
        readError = nil
        error(err, 0)
    end
end

return MeshIndexData
