--[[
    If Flags not set in Main.cpp set them to false or 0
]]
if __debug__ == nil then __debug__ = false end
if __checklevel__ == nil then __checklevel__ = 0 end
if __embedded__ == nil then __embedded__ = false end

--[[
    Import both libphx ffi, jit and lfs into global library.
]]
ffi = require('ffi')
jit = require('jit')
lfs = require('lfs_ffi')

--[[
    Disable JIT on macOS ARM64, as it has significant performance issues.

    Relevant context:
    * https://github.com/LuaJIT/LuaJIT/issues/285
    * https://github.com/minetest/minetest/issues/14611
    * https://love2d.org/forums/viewtopic.php?t=94760
]]
if jit and ffi.os == "OSX" and ffi.arch == "arm64" then
    jit.off()
end

--[[
    Importing all math functions (presumably from ffi and jit? Need Confirmation)
    into Global Table.
]]
for k, v in pairs(math) do
    if type(v) == 'function' then
        _G[k] = v
    end
end

---- Global Variables
require('Globals')

---- Aliases Required for ToString. (Should I require them inside ToString?)
require('Config.Aliases')
require('Core.LuaExtensions.ToString')

---- LuaExtensions IOEx and StringEx required for "RequireAll"
require('Core.LuaExtensions.IOEx')
require('Core.LuaExtensions.StringEx')
require('Core.Util.RequireAll')

---- Finish Requiring all Lua Built-In Class Extensions
require('Core.LuaExtensions.TableEx')

---- Now handle loading Util and Structures into Core and Global Namespace
Core.Struct = requireAll('Core.Structures')
Core.Util = requireAll('Core.Util')
Core.Util.Namespace.Inline(Core.Util, 'Core.Util')
Namespace.Inline(Core.Struct, 'Core.Struct')
Core.Events = requireAll('Systems.Events')
Namespace.Inline(Core.Events, 'Systems.Events')


---- Load in FFI
-- Please note. All of this will need double checking. This is near 1-1 to Josh's.
-- His Reasoning for using requireAll, Inline, Inject. Is unclear.
do -- Basic Typedefs
    ffi.cdef [[
        typedef unsigned long  ulong;
        typedef unsigned int   uint;
        typedef unsigned short ushort;
        typedef unsigned char  uchar;
        typedef char const*    cstr;
        typedef int8_t         int8;
        typedef int16_t        int16;
        typedef int32_t        int32;
        typedef int64_t        int64;
        typedef uint8_t        uint8;
        typedef uint16_t       uint16;
        typedef uint32_t       uint32;
        typedef uint64_t       uint64;
    ]]
end


local genObjects, genFiles, genOpaques, genStructs = requireAllGenerated('ffi_gen')

Core.FFI = {}
Core.FFI.Ext = requireAll('ffi_ext')
Core.FFI.Lib = require('libphx')

Core.FFI.Base = requireAll('ffi_common')
Namespace.Inline(Core.FFI.Base, 'Core.FFI.Base')
Namespace.Inject(Core.FFI, 'Core.FFI', Core.FFI.Base, 'Core.FFI.Base')

-- Load type definitions
for k, v in pairs(genFiles) do
    local obj = v.defineType()

    genObjects[k] = obj
end

Core.FFI.Gen = genObjects
Namespace.Inline(Core.FFI.Gen, 'Core.FFI.Gen')
Namespace.Inject(Core.FFI, 'Core.FFI', Core.FFI.Gen, 'Core.FFI.Gen')

for _, v in ipairs(genOpaques) do
    table.insert(Core.FFI.Lib.Opaques, v)
end

for _, v in ipairs(genStructs) do
    table.insert(Core.FFI.Lib.Structs, v)
end

Core.FFI.CFFI = requireAll('Core.CFFI')
Namespace.Inline(Core.FFI.CFFI, 'Core.FFI.CFFI')

---- Additional Lua Configurations (Not sure how any of it works will comment further later.)

-- Builtins registered with Type library
local builtins = {
    'int8_t',
    'int16_t',
    'int32_t',
    'int64_t',
    'uint8_t',
    'uint16_t',
    'uint32_t',
    'uint64_t',
    'float',
    'double',
    'cstr',
}

-- Typedefs registered with Type library
local lua_typedefs = {
    { 'int8_t',   'Int8' },
    { 'int16_t',  'Int16' },
    { 'int32_t',  'Int32' },
    { 'int64_t',  'Int64' },
    { 'uint8_t',  'Uint8' },
    { 'uint16_t', 'Uint16' },
    { 'uint32_t', 'Uint32' },
    { 'uint64_t', 'Uint64' },
    { 'float',    'Float32' },
    { 'double',   'Float64' },
    { 'cstr',     'String' },
}

for i = 1, #builtins do
    Type.Create(builtins[i], true)
end

for i = 1, #lua_typedefs do
    local src = lua_typedefs[i][1]
    local dst = lua_typedefs[i][2]
    Type.Alias(src, dst)
    CType[dst] = Type.Get(src)
end

for i = 1, #Core.FFI.Lib.Opaques do
    local name = Core.FFI.Lib.Opaques[i]
    local wrapperName = format('Opaque_%s', name)
    ffi.cdef(format('typedef %s %s;', name, wrapperName));
    local type = Type.Create(wrapperName, true)
    local ptr  = CType.Pointer(type)
    Type.Alias(ptr.name, name)
    CType[name] = ptr
end

for i = 1, #Core.FFI.Lib.Structs do
    local name = Core.FFI.Lib.Structs[i]
    local type = Type.Create(name, true)
    CType[name] = type
end

---- Load Renderer into Global Space
Render = requireAll('Render')
Namespace.Inline(Render, 'Render')

-- Call Function for Running main with errorHandler
function Core.Call(fn, ...)
    local status, ret = xpcall(fn, ErrorHandler, ...)
    if not status then
        printf('Error calling: %s(%s). Ret: %s', fn, ..., ret)
        os.exit()
    end
    return ret
end

function Core.ManagedObject(instance, free_func)
    if instance == nil then
        return nil
    end

    return ffi.gc(instance, free_func)
end
