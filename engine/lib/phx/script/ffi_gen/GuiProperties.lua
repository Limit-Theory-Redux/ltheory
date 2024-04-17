-- GuiProperties ---------------------------------------------------------------

local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef uint8 GuiProperties;
    ]]

    return 2, 'GuiProperties'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local GuiProperties

    do -- C Definitions
        ffi.cdef [[
            GuiProperties GuiProperties_ContainerSpacingId;
            GuiProperties GuiProperties_ContainerColorFrameId;
            GuiProperties GuiProperties_ContainerColorPrimaryId;
            GuiProperties GuiProperties_TextFontId;
            GuiProperties GuiProperties_TextColorId;
            GuiProperties GuiProperties_ButtonBorderWidthId;
            GuiProperties GuiProperties_ButtonTextColorId;

            cstr          GuiProperties_ToString(GuiProperties);
        ]]
    end

    do -- Global Symbol Table
        GuiProperties = {
            ContainerSpacingId      = libphx.GuiProperties_ContainerSpacingId,
            ContainerColorFrameId   = libphx.GuiProperties_ContainerColorFrameId,
            ContainerColorPrimaryId = libphx.GuiProperties_ContainerColorPrimaryId,
            TextFontId              = libphx.GuiProperties_TextFontId,
            TextColorId             = libphx.GuiProperties_TextColorId,
            ButtonBorderWidthId     = libphx.GuiProperties_ButtonBorderWidthId,
            ButtonTextColorId       = libphx.GuiProperties_ButtonTextColorId,

            ToString                = libphx.GuiProperties_ToString,
        }

        if onDef_GuiProperties then onDef_GuiProperties(GuiProperties, mt) end
        GuiProperties = setmetatable(GuiProperties, mt)
    end

    return GuiProperties
end

return Loader
