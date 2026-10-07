-- AUTO GENERATED. DO NOT MODIFY!
-- ReadbackTicket --------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef struct ReadbackTicket {} ReadbackTicket;
    ]]

    return 1, 'ReadbackTicket'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local ReadbackTicket

    do -- C Definitions
        ffi.cdef [[
            void   ReadbackTicket_Free      (ReadbackTicket*);
            bool   ReadbackTicket_Ready     (ReadbackTicket const*);
            bool   ReadbackTicket_Failed    (ReadbackTicket const*);
            Bytes* ReadbackTicket_Data      (ReadbackTicket const*);
            int    ReadbackTicket_GetWidth  (ReadbackTicket const*);
            int    ReadbackTicket_GetHeight (ReadbackTicket const*);
            void   ReadbackTicket_Release   (ReadbackTicket*);
        ]]
    end

    do -- Global Symbol Table
        ReadbackTicket = {}

        if onDef_ReadbackTicket then onDef_ReadbackTicket(ReadbackTicket, mt) end
        ReadbackTicket = setmetatable(ReadbackTicket, mt)
    end

    do -- Metatype for class instances
        local t  = ffi.typeof('ReadbackTicket')
        local mt = {
            __index = {
                ready     = libphx.ReadbackTicket_Ready,
                failed    = libphx.ReadbackTicket_Failed,
                data      = function(self)
                    local _instance = libphx.ReadbackTicket_Data(self)
                    return Core.ManagedObject(_instance, libphx.Bytes_Free)
                end,
                getWidth  = libphx.ReadbackTicket_GetWidth,
                getHeight = libphx.ReadbackTicket_GetHeight,
                release   = libphx.ReadbackTicket_Release,
            },
        }

        if onDef_ReadbackTicket_t then onDef_ReadbackTicket_t(t, mt) end
        ReadbackTicket_t = ffi.metatype(t, mt)
    end

    return ReadbackTicket
end

return Loader
