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
            Solid        = 0,
            Image        = 1,
            Text         = 2,
            TextAdditive = 3,
            Box          = 4,
            Circle       = 5,
            Grid         = 6,
            Hex          = 7,
            Icon         = 8,
            Panel        = 9,
            PanelGlow    = 10,
            Point        = 11,
            PointGlow    = 12,
            Ring         = 13,
            RingGlow     = 14,
            RingDim      = 15,
            Triangle     = 16,
            Wedge        = 17,
            Annulus      = 18,
            LineGlow     = 19,

            ToString     = libphx.Shape_ToString,
        }

        if onDef_Shape then onDef_Shape(Shape, mt) end
        Shape = setmetatable(Shape, mt)
    end

    return Shape
end

return Loader
