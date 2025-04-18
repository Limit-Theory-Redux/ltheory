-- AUTO GENERATED. DO NOT MODIFY!
-- Viewport --------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    return 0, 'Viewport'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local Viewport

    do -- C Definitions
        ffi.cdef [[
            float Viewport_GetAspect ();
            void  Viewport_GetSize   (Vec2i* out);
            void  Viewport_Push      (int x, int y, int sx, int sy, bool isWindow);
            void  Viewport_Pop       ();
        ]]
    end

    do -- Global Symbol Table
        Viewport = {
            GetAspect = libphx.Viewport_GetAspect,
            GetSize   = libphx.Viewport_GetSize,
            Push      = libphx.Viewport_Push,
            Pop       = libphx.Viewport_Pop,
        }

        if onDef_Viewport then onDef_Viewport(Viewport, mt) end
        Viewport = setmetatable(Viewport, mt)
    end

    return Viewport
end

return Loader
