-- AUTO GENERATED. DO NOT MODIFY!
-- BSPNodeRel ------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef uint8 BSPNodeRel;
    ]]

    return 2, 'BSPNodeRel'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local BSPNodeRel

    do -- C Definitions
        ffi.cdef [[
            cstr       BSPNodeRel_ToString(BSPNodeRel);
        ]]
    end

    do -- Global Symbol Table
        BSPNodeRel = {
            Parent   = 0,
            Back     = 1,
            Front    = 2,

            ToString = libphx.BSPNodeRel_ToString,
        }

        if onDef_BSPNodeRel then onDef_BSPNodeRel(BSPNodeRel, mt) end
        BSPNodeRel = setmetatable(BSPNodeRel, mt)
    end

    return BSPNodeRel
end

return Loader
