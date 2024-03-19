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
            GuiProperties GuiProperties_OpacityId;
            GuiProperties GuiProperties_BackgroundColorId;
            GuiProperties GuiProperties_HighlightColorId;
            GuiProperties GuiProperties_TextFontId;
            GuiProperties GuiProperties_TextColorId;
            GuiProperties GuiProperties_ContainerSpacingId;
            GuiProperties GuiProperties_ContainerColorFrameId;
            GuiProperties GuiProperties_ContainerColorPrimaryId;
            GuiProperties GuiProperties_ButtonBorderWidthId;
            GuiProperties GuiProperties_ButtonTextColorId;
            GuiProperties GuiProperties_ButtonOpacityId;
            GuiProperties GuiProperties_ButtonBackgroundColorId;
            GuiProperties GuiProperties_ButtonHighlightColorId;
            GuiProperties GuiProperties_CheckboxBackgroundColorId;
            GuiProperties GuiProperties_CheckboxHighlightColorId;

            cstr          GuiProperties_ToString(GuiProperties);
        ]]
    end

    do -- Global Symbol Table
        GuiProperties = {
            OpacityId                 = libphx.GuiProperties_OpacityId,
            BackgroundColorId         = libphx.GuiProperties_BackgroundColorId,
            HighlightColorId          = libphx.GuiProperties_HighlightColorId,
            TextFontId                = libphx.GuiProperties_TextFontId,
            TextColorId               = libphx.GuiProperties_TextColorId,
            ContainerSpacingId        = libphx.GuiProperties_ContainerSpacingId,
            ContainerColorFrameId     = libphx.GuiProperties_ContainerColorFrameId,
            ContainerColorPrimaryId   = libphx.GuiProperties_ContainerColorPrimaryId,
            ButtonBorderWidthId       = libphx.GuiProperties_ButtonBorderWidthId,
            ButtonTextColorId         = libphx.GuiProperties_ButtonTextColorId,
            ButtonOpacityId           = libphx.GuiProperties_ButtonOpacityId,
            ButtonBackgroundColorId   = libphx.GuiProperties_ButtonBackgroundColorId,
            ButtonHighlightColorId    = libphx.GuiProperties_ButtonHighlightColorId,
            CheckboxBackgroundColorId = libphx.GuiProperties_CheckboxBackgroundColorId,
            CheckboxHighlightColorId  = libphx.GuiProperties_CheckboxHighlightColorId,

            ToString                  = libphx.GuiProperties_ToString,
        }

        if onDef_GuiProperties then onDef_GuiProperties(GuiProperties, mt) end
        GuiProperties = setmetatable(GuiProperties, mt)
    end

    return GuiProperties
end

return Loader
