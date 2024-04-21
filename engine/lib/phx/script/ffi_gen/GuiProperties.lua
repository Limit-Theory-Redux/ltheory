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
            GuiProperties GuiProperties_Opacity;
            GuiProperties GuiProperties_BackgroundColor;
            GuiProperties GuiProperties_HighlightColor;
            GuiProperties GuiProperties_TextFont;
            GuiProperties GuiProperties_TextColor;
            GuiProperties GuiProperties_ContainerClip;
            GuiProperties GuiProperties_ContainerSpacing;
            GuiProperties GuiProperties_ContainerColorFrame;
            GuiProperties GuiProperties_ContainerColorPrimary;
            GuiProperties GuiProperties_ButtonBorderWidth;
            GuiProperties GuiProperties_ButtonTextColor;
            GuiProperties GuiProperties_ButtonOpacity;
            GuiProperties GuiProperties_ButtonBackgroundColor;
            GuiProperties GuiProperties_ButtonHighlightColor;
            GuiProperties GuiProperties_CheckboxBackgroundColor;
            GuiProperties GuiProperties_CheckboxHighlightColor;
            GuiProperties GuiProperties_ScrollAreaHScrollShow;
            GuiProperties GuiProperties_ScrollAreaVScrollShow;
            GuiProperties GuiProperties_ScrollAreaScrollbarLength;
            GuiProperties GuiProperties_ScrollAreaScrollbarBackgroundColor;
            GuiProperties GuiProperties_ScrollAreaScrollScale;
            GuiProperties GuiProperties_ScrollAreaScrollbarVisibilityStableTime;
            GuiProperties GuiProperties_ScrollAreaScrollbarVisibilityFadeTime;

            cstr          GuiProperties_ToString(GuiProperties);
        ]]
    end

    do -- Global Symbol Table
        GuiProperties = {
            Opacity                                 = libphx.GuiProperties_Opacity,
            BackgroundColor                         = libphx.GuiProperties_BackgroundColor,
            HighlightColor                          = libphx.GuiProperties_HighlightColor,
            TextFont                                = libphx.GuiProperties_TextFont,
            TextColor                               = libphx.GuiProperties_TextColor,
            ContainerClip                           = libphx.GuiProperties_ContainerClip,
            ContainerSpacing                        = libphx.GuiProperties_ContainerSpacing,
            ContainerColorFrame                     = libphx.GuiProperties_ContainerColorFrame,
            ContainerColorPrimary                   = libphx.GuiProperties_ContainerColorPrimary,
            ButtonBorderWidth                       = libphx.GuiProperties_ButtonBorderWidth,
            ButtonTextColor                         = libphx.GuiProperties_ButtonTextColor,
            ButtonOpacity                           = libphx.GuiProperties_ButtonOpacity,
            ButtonBackgroundColor                   = libphx.GuiProperties_ButtonBackgroundColor,
            ButtonHighlightColor                    = libphx.GuiProperties_ButtonHighlightColor,
            CheckboxBackgroundColor                 = libphx.GuiProperties_CheckboxBackgroundColor,
            CheckboxHighlightColor                  = libphx.GuiProperties_CheckboxHighlightColor,
            ScrollAreaHScrollShow                   = libphx.GuiProperties_ScrollAreaHScrollShow,
            ScrollAreaVScrollShow                   = libphx.GuiProperties_ScrollAreaVScrollShow,
            ScrollAreaScrollbarLength               = libphx.GuiProperties_ScrollAreaScrollbarLength,
            ScrollAreaScrollbarBackgroundColor      = libphx.GuiProperties_ScrollAreaScrollbarBackgroundColor,
            ScrollAreaScrollScale                   = libphx.GuiProperties_ScrollAreaScrollScale,
            ScrollAreaScrollbarVisibilityStableTime = libphx.GuiProperties_ScrollAreaScrollbarVisibilityStableTime,
            ScrollAreaScrollbarVisibilityFadeTime   = libphx.GuiProperties_ScrollAreaScrollbarVisibilityFadeTime,

            ToString                                = libphx.GuiProperties_ToString,
        }

        if onDef_GuiProperties then onDef_GuiProperties(GuiProperties, mt) end
        GuiProperties = setmetatable(GuiProperties, mt)
    end

    return GuiProperties
end

return Loader
