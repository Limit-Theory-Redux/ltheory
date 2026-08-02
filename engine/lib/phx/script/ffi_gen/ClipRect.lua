-- AUTO GENERATED. DO NOT MODIFY!
-- ClipRect --------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    return 0, 'ClipRect'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local ClipRect

    do -- C Definitions
        ffi.cdef [[
            void ClipRect_Push          (Renderer* r, float x, float y, float sx, float sy);
            void ClipRect_PushCombined  (Renderer* r, float x, float y, float sx, float sy);
            void ClipRect_PushDisabled  (Renderer* r);
            void ClipRect_PushTransform (Renderer* r, float tx, float ty, float sx, float sy);
            void ClipRect_Pop           (Renderer* r);
            void ClipRect_PopTransform  (Renderer* r);
        ]]
    end

    do -- Global Symbol Table
        ClipRect = {
            Push          = libphx.ClipRect_Push,
            PushCombined  = libphx.ClipRect_PushCombined,
            PushDisabled  = libphx.ClipRect_PushDisabled,
            PushTransform = libphx.ClipRect_PushTransform,
            Pop           = libphx.ClipRect_Pop,
            PopTransform  = libphx.ClipRect_PopTransform,
        }

        if onDef_ClipRect then onDef_ClipRect(ClipRect, mt) end
        ClipRect = setmetatable(ClipRect, mt)
    end

    return ClipRect
end

return Loader
