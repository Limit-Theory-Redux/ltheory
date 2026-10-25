-- AUTO GENERATED. DO NOT MODIFY!
-- BindGroup -------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef struct BindGroup {} BindGroup;
    ]]

    return 1, 'BindGroup'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local BindGroup

    do -- C Definitions
        ffi.cdef [[
            void   BindGroup_Free (BindGroup*);
            uint32 BindGroup_Id   (BindGroup const*);
        ]]
    end

    do -- Global Symbol Table
        BindGroup = {}

        if onDef_BindGroup then onDef_BindGroup(BindGroup, mt) end
        BindGroup = setmetatable(BindGroup, mt)
    end

    do -- Metatype for class instances
        local t  = ffi.typeof('BindGroup')
        local mt = {
            __index = {
                id   = libphx.BindGroup_Id,
            },
        }

        if onDef_BindGroup_t then onDef_BindGroup_t(t, mt) end
        BindGroup_t = ffi.metatype(t, mt)
    end

    return BindGroup
end

return Loader
