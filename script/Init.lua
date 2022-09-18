--[[
    If Flags not set in Main.cpp set them to false or 0
]]
if __debug__      == nil then __debug__      = false end
if __checklevel__ == nil then __checklevel__ = 0     end
if __embedded__   == nil then __embedded__   = false end

--[[
    Import both libphx ffi and jit into global library.
]]
ffi = require('ffi')
jit = require('jit')

--[[
    Importing all math functions (presumabely from ffi and jit? Need Confirmation)
    into Global Table.
]]
for k, v in pairs(math) do
  if type(v) == 'function' then
    _G[k] = v
  end
end

---- Will be the global variable to access all functions and classes within Core/
Core = {}
Renderer = {}
Config = {}

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
Core.Event = requireAll('Core.Event')
Namespace.Inline(Core.Event, 'Core.Event')


---- Load in FFI
-- Please note. All of this will need double checking. This is near 1-1 to Josh's.
-- His Reasoning for using requireAll, Inline, Inject. Is unclear.
Core.FFI = {}
Core.FFI.Ext = requireAll('ffiext')
Core.FFI.Lib = require('ffi.libphx')

Core.FFI.Base = requireAll('ffi')
Namespace.Inline(Core.FFI.Base, 'Core.FFI.Base')
Namespace.Inject(Core.FFI, 'Core.FFI', Core.FFI.Base, 'Core.FFI.Base')

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
  { 'int8_t',   'Int8'    },
  { 'int16_t',  'Int16'   },
  { 'int32_t',  'Int32'   },
  { 'int64_t',  'Int64'   },
  { 'uint8_t',  'Uint8'   },
  { 'uint16_t', 'Uint16'  },
  { 'uint32_t', 'Uint32'  },
  { 'uint64_t', 'Uint64'  },
  { 'float',    'Float32' },
  { 'double',   'Float64' },
  { 'cstr',     'String'  },
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
Renderer = requireAll('Renderer')
Namespace.Inline(Renderer, 'Renderer')

-- Call Function for Running main with errorHandler
function Core.Call(fn)
  local _, err = xpcall(fn, ErrorHandler)
  if err then print(err) end
end
