-- AUTO GENERATED. DO NOT MODIFY!
-- CursorIcon ------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef uint8 CursorIcon;
    ]]

    return 2, 'CursorIcon'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local CursorIcon

    do -- C Definitions
        ffi.cdef [[
            cstr       CursorIcon_ToString(CursorIcon);
        ]]
    end

    do -- Global Symbol Table
        CursorIcon = {
            Default      = 0,
            Crosshair    = 1,
            Pointer      = 2,
            Move         = 3,
            Text         = 4,
            Wait         = 5,
            Help         = 6,
            Progress     = 7,
            NotAllowed   = 8,
            ContextMenu  = 9,
            Cell         = 10,
            VerticalText = 11,
            Alias        = 12,
            Copy         = 13,
            NoDrop       = 14,
            Grab         = 15,
            Grabbing     = 16,
            AllScroll    = 17,
            ZoomIn       = 18,
            ZoomOut      = 19,
            EResize      = 20,
            NResize      = 21,
            NeResize     = 22,
            NwResize     = 23,
            SResize      = 24,
            SeResize     = 25,
            SwResize     = 26,
            WResize      = 27,
            EwResize     = 28,
            NsResize     = 29,
            NeswResize   = 30,
            NwseResize   = 31,
            ColResize    = 32,
            RowResize    = 33,

            ToString     = libphx.CursorIcon_ToString,
        }

        if onDef_CursorIcon then onDef_CursorIcon(CursorIcon, mt) end
        CursorIcon = setmetatable(CursorIcon, mt)
    end

    return CursorIcon
end

return Loader
