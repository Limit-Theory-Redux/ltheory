-- AUTO GENERATED. DO NOT MODIFY!
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
            cstr          GuiLayoutType_ToString(GuiLayoutType);
        ]]
    end

    do -- Global Symbol Table
        GuiLayoutType = {
            Stack      = 0,
            Horizontal = 1,
            Vertical   = 2,

            ToString   = libphx.GuiLayoutType_ToString,
        }

        if onDef_GuiLayoutType then onDef_GuiLayoutType(GuiLayoutType, mt) end
        GuiLayoutType = setmetatable(GuiLayoutType, mt)
    end

    return GuiLayoutType
end

return Loader
