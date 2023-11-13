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
            CursorIcon CursorIcon_Default;
            CursorIcon CursorIcon_Crosshair;
            CursorIcon CursorIcon_Hand;
            CursorIcon CursorIcon_Arrow;
            CursorIcon CursorIcon_Move;
            CursorIcon CursorIcon_Text;
            CursorIcon CursorIcon_Wait;
            CursorIcon CursorIcon_Help;
            CursorIcon CursorIcon_Progress;
            CursorIcon CursorIcon_NotAllowed;
            CursorIcon CursorIcon_ContextMenu;
            CursorIcon CursorIcon_Cell;
            CursorIcon CursorIcon_VerticalText;
            CursorIcon CursorIcon_Alias;
            CursorIcon CursorIcon_Copy;
            CursorIcon CursorIcon_NoDrop;
            CursorIcon CursorIcon_Grab;
            CursorIcon CursorIcon_Grabbing;
            CursorIcon CursorIcon_AllScroll;
            CursorIcon CursorIcon_ZoomIn;
            CursorIcon CursorIcon_ZoomOut;
            CursorIcon CursorIcon_EResize;
            CursorIcon CursorIcon_NResize;
            CursorIcon CursorIcon_NeResize;
            CursorIcon CursorIcon_NwResize;
            CursorIcon CursorIcon_SResize;
            CursorIcon CursorIcon_SeResize;
            CursorIcon CursorIcon_SwResize;
            CursorIcon CursorIcon_WResize;
            CursorIcon CursorIcon_EwResize;
            CursorIcon CursorIcon_NsResize;
            CursorIcon CursorIcon_NeswResize;
            CursorIcon CursorIcon_NwseResize;
            CursorIcon CursorIcon_ColResize;
            CursorIcon CursorIcon_RowResize;

            cstr       CursorIcon_ToString(CursorIcon);
        ]]
    end

    do -- Global Symbol Table
        CursorIcon = {
            Default      = libphx.CursorIcon_Default,
            Crosshair    = libphx.CursorIcon_Crosshair,
            Hand         = libphx.CursorIcon_Hand,
            Arrow        = libphx.CursorIcon_Arrow,
            Move         = libphx.CursorIcon_Move,
            Text         = libphx.CursorIcon_Text,
            Wait         = libphx.CursorIcon_Wait,
            Help         = libphx.CursorIcon_Help,
            Progress     = libphx.CursorIcon_Progress,
            NotAllowed   = libphx.CursorIcon_NotAllowed,
            ContextMenu  = libphx.CursorIcon_ContextMenu,
            Cell         = libphx.CursorIcon_Cell,
            VerticalText = libphx.CursorIcon_VerticalText,
            Alias        = libphx.CursorIcon_Alias,
            Copy         = libphx.CursorIcon_Copy,
            NoDrop       = libphx.CursorIcon_NoDrop,
            Grab         = libphx.CursorIcon_Grab,
            Grabbing     = libphx.CursorIcon_Grabbing,
            AllScroll    = libphx.CursorIcon_AllScroll,
            ZoomIn       = libphx.CursorIcon_ZoomIn,
            ZoomOut      = libphx.CursorIcon_ZoomOut,
            EResize      = libphx.CursorIcon_EResize,
            NResize      = libphx.CursorIcon_NResize,
            NeResize     = libphx.CursorIcon_NeResize,
            NwResize     = libphx.CursorIcon_NwResize,
            SResize      = libphx.CursorIcon_SResize,
            SeResize     = libphx.CursorIcon_SeResize,
            SwResize     = libphx.CursorIcon_SwResize,
            WResize      = libphx.CursorIcon_WResize,
            EwResize     = libphx.CursorIcon_EwResize,
            NsResize     = libphx.CursorIcon_NsResize,
            NeswResize   = libphx.CursorIcon_NeswResize,
            NwseResize   = libphx.CursorIcon_NwseResize,
            ColResize    = libphx.CursorIcon_ColResize,
            RowResize    = libphx.CursorIcon_RowResize,

            ToString     = libphx.CursorIcon_ToString,
        }

        if onDef_CursorIcon then onDef_CursorIcon(CursorIcon, mt) end
        CursorIcon = setmetatable(CursorIcon, mt)
    end

    return CursorIcon
end

return Loader
