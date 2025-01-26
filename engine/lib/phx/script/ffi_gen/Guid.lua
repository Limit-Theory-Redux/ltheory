-- AUTO GENERATED. DO NOT MODIFY!
-- Guid ------------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    return 0, 'Guid'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local Guid

    do -- C Definitions
        ffi.cdef [[
            uint64 Guid_Create ();
            bool   Guid_Exists (uint64 id);
            void   Guid_Reset  ();
        ]]
    end

    do -- Global Symbol Table
        Guid = {
            Create = libphx.Guid_Create,
            Exists = libphx.Guid_Exists,
            Reset  = libphx.Guid_Reset,
        }

        if onDef_Guid then onDef_Guid(Guid, mt) end
        Guid = setmetatable(Guid, mt)
    end

    return Guid
end

return Loader
