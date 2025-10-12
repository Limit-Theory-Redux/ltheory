-- AUTO GENERATED. DO NOT MODIFY!
-- Bit -------------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    return 0, 'Bit'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local Bit

    do -- C Definitions
        ffi.cdef [[
            uint32 Bit_And32     (uint32 x, uint32 y);
            uint32 Bit_Or32      (uint32 x, uint32 y);
            uint32 Bit_Xor32     (uint32 x, uint32 y);
            uint32 Bit_Not32     (uint32 x);
            bool   Bit_Has32     (uint32 x, uint32 y);
            bool   Bit_HasAny32  (uint32 x, uint32 y);
            uint32 Bit_Lshift32  (uint32 x, uint32 n);
            uint32 Bit_Rshift32  (uint32 x, uint32 n);
            uint32 Bit_Bitmask32 (uint32 pos);
            uint64 Bit_And64     (uint64 x, uint64 y);
            uint64 Bit_Or64      (uint64 x, uint64 y);
            uint64 Bit_Xor64     (uint64 x, uint64 y);
            uint64 Bit_Not64     (uint64 x);
            bool   Bit_Has64     (uint64 x, uint64 y);
            bool   Bit_HasAny64  (uint64 x, uint64 y);
            uint64 Bit_Lshift64  (uint64 x, uint32 n);
            uint64 Bit_Rshift64  (uint64 x, uint32 n);
            uint64 Bit_Bitmask64 (uint32 pos);
        ]]
    end

    do -- Global Symbol Table
        Bit = {
            And32     = libphx.Bit_And32,
            Or32      = libphx.Bit_Or32,
            Xor32     = libphx.Bit_Xor32,
            Not32     = libphx.Bit_Not32,
            Has32     = libphx.Bit_Has32,
            HasAny32  = libphx.Bit_HasAny32,
            Lshift32  = libphx.Bit_Lshift32,
            Rshift32  = libphx.Bit_Rshift32,
            Bitmask32 = libphx.Bit_Bitmask32,
            And64     = libphx.Bit_And64,
            Or64      = libphx.Bit_Or64,
            Xor64     = libphx.Bit_Xor64,
            Not64     = libphx.Bit_Not64,
            Has64     = libphx.Bit_Has64,
            HasAny64  = libphx.Bit_HasAny64,
            Lshift64  = libphx.Bit_Lshift64,
            Rshift64  = libphx.Bit_Rshift64,
            Bitmask64 = libphx.Bit_Bitmask64,
        }

        if onDef_Bit then onDef_Bit(Bit, mt) end
        Bit = setmetatable(Bit, mt)
    end

    return Bit
end

return Loader
