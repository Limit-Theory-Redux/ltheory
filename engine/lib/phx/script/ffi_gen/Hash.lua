-- AUTO GENERATED. DO NOT MODIFY!
-- Hash ------------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    return 0, 'Hash'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local Hash

    do -- C Definitions
        ffi.cdef [[
            uint32 Hash_FNV32             (uint8 const* buf, uint64 buf_size);
            uint64 Hash_FNV64             (uint8 const* buf, uint64 buf_size);
            uint32 Hash_FNVStr32          (cstr s);
            uint64 Hash_FNVStr64          (cstr s);
            uint64 Hash_FNV64_Init        ();
            uint64 Hash_FNV64_Incremental (uint64 this, uint8 const* buf, uint64 buf_size);
        ]]
    end

    do -- Global Symbol Table
        Hash = {
            FNV32             = libphx.Hash_FNV32,
            FNV64             = libphx.Hash_FNV64,
            FNVStr32          = libphx.Hash_FNVStr32,
            FNVStr64          = libphx.Hash_FNVStr64,
            FNV64_Init        = libphx.Hash_FNV64_Init,
            FNV64_Incremental = libphx.Hash_FNV64_Incremental,
        }

        if onDef_Hash then onDef_Hash(Hash, mt) end
        Hash = setmetatable(Hash, mt)
    end

    return Hash
end

return Loader
