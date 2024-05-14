-- GuiLayoutType ---------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef uint8 GuiLayoutType;
    ]]

    return 2, 'GuiLayoutType'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local GuiLayoutType

    do -- C Definitions
        ffi.cdef [[
            GuiLayoutType GuiLayoutType_Stack;
            GuiLayoutType GuiLayoutType_Horizontal;
            GuiLayoutType GuiLayoutType_Vertical;

            cstr          GuiLayoutType_ToString(GuiLayoutType);
        ]]
    end

    do -- Global Symbol Table
        GuiLayoutType = {
            Stack      = libphx.GuiLayoutType_Stack,
            Horizontal = libphx.GuiLayoutType_Horizontal,
            Vertical   = libphx.GuiLayoutType_Vertical,

            ToString   = libphx.GuiLayoutType_ToString,
        }

        if onDef_GuiLayoutType then onDef_GuiLayoutType(GuiLayoutType, mt) end
        GuiLayoutType = setmetatable(GuiLayoutType, mt)
    end

    return GuiLayoutType
end

return Loader
