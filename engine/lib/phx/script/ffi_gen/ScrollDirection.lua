-- ScrollDirection -------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef uint8 ScrollDirection;
    ]]

    return 2, 'ScrollDirection'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local ScrollDirection

    do -- C Definitions
        ffi.cdef [[
            ScrollDirection ScrollDirection_All;
            ScrollDirection ScrollDirection_Horizontal;
            ScrollDirection ScrollDirection_Vertical;

            cstr            ScrollDirection_ToString(ScrollDirection);
        ]]
    end

    do -- Global Symbol Table
        ScrollDirection = {
            All        = libphx.ScrollDirection_All,
            Horizontal = libphx.ScrollDirection_Horizontal,
            Vertical   = libphx.ScrollDirection_Vertical,

            ToString   = libphx.ScrollDirection_ToString,
        }

        if onDef_ScrollDirection then onDef_ScrollDirection(ScrollDirection, mt) end
        ScrollDirection = setmetatable(ScrollDirection, mt)
    end

    return ScrollDirection
end

return Loader
