-- AUTO GENERATED. DO NOT MODIFY!
-- Shape -----------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef uint32 Shape;
    ]]

    return 2, 'Shape'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local Shape

    do -- C Definitions
        ffi.cdef [[
            cstr  Shape_ToString(Shape);
        ]]
    end

    do -- Global Symbol Table
        Shape = {
            Solid     = 0,
            Image     = 1,
            Text      = 2,
            Box       = 3,
            Circle    = 4,
            Grid      = 5,
            Hex       = 6,
            Icon      = 7,
            Panel     = 8,
            PanelGlow = 9,
            Point     = 10,
            PointGlow = 11,
            Ring      = 12,
            RingGlow  = 13,
            RingDim   = 14,
            Triangle  = 15,
            Wedge     = 16,
            Annulus   = 17,
            LineGlow  = 18,

            ToString  = libphx.Shape_ToString,
        }

        if onDef_Shape then onDef_Shape(Shape, mt) end
        Shape = setmetatable(Shape, mt)
    end

    return Shape
end

return Loader
