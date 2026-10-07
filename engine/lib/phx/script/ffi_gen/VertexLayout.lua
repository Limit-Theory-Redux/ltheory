-- AUTO GENERATED. DO NOT MODIFY!
-- VertexLayout ----------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef uint32 VertexLayout;
    ]]

    return 2, 'VertexLayout'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local VertexLayout

    do -- C Definitions
        ffi.cdef [[
            cstr         VertexLayout_ToString(VertexLayout);
        ]]
    end

    do -- Global Symbol Table
        VertexLayout = {
            Mesh       = 0,
            Fullscreen = 1,

            ToString   = libphx.VertexLayout_ToString,
        }

        if onDef_VertexLayout then onDef_VertexLayout(VertexLayout, mt) end
        VertexLayout = setmetatable(VertexLayout, mt)
    end

    return VertexLayout
end

return Loader
