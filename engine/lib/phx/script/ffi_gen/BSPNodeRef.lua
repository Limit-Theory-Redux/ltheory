-- AUTO GENERATED. DO NOT MODIFY!
-- BSPNodeRef ------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef struct BSPNodeRef {
            int32 index;
            uint8 triangleCount;
        } BSPNodeRef;
    ]]

    return 1, 'BSPNodeRef'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local BSPNodeRef

    do -- Global Symbol Table
        BSPNodeRef = {}

        local mt = {
            __call = function(t, ...) return BSPNodeRef_t(...) end,
        }

        if onDef_BSPNodeRef then onDef_BSPNodeRef(BSPNodeRef, mt) end
        BSPNodeRef = setmetatable(BSPNodeRef, mt)
    end

    do -- Metatype for class instances
        local t  = ffi.typeof('BSPNodeRef')
        local mt = {
            __index = {
                clone = function(x) return BSPNodeRef_t(x) end,
            },
        }

        if onDef_BSPNodeRef_t then onDef_BSPNodeRef_t(t, mt) end
        BSPNodeRef_t = ffi.metatype(t, mt)
    end

    return BSPNodeRef
end

return Loader
