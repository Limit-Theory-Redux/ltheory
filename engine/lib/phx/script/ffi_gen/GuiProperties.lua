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
            GuiProperties GuiProperties_BorderColor;
            GuiProperties GuiProperties_BackgroundColor;
            GuiProperties GuiProperties_HighlightColor;
            GuiProperties GuiProperties_TextFont;
            GuiProperties GuiProperties_TextColor;
            GuiProperties GuiProperties_ContainerClip;
            GuiProperties GuiProperties_ContainerSpacing;
            GuiProperties GuiProperties_ContainerColorPrimary;
            GuiProperties GuiProperties_ButtonRectOpacity;
            GuiProperties GuiProperties_ButtonRectBorderColor;
            GuiProperties GuiProperties_ButtonRectBackgroundColor;
            GuiProperties GuiProperties_ButtonRectHighlightColor;
            GuiProperties GuiProperties_ButtonTextOpacity;
            GuiProperties GuiProperties_ButtonTextBackgroundColor;
            GuiProperties GuiProperties_ButtonTextHighlightColor;
            GuiProperties GuiProperties_ButtonTextFont;
            GuiProperties GuiProperties_ButtonTextColor;
            GuiProperties GuiProperties_CheckboxRectOpacity;
            GuiProperties GuiProperties_CheckboxRectBorderColor;
            GuiProperties GuiProperties_CheckboxRectBackgroundColor;
            GuiProperties GuiProperties_CheckboxRectHighlightColor;
            GuiProperties GuiProperties_CheckboxTextOpacity;
            GuiProperties GuiProperties_CheckboxTextBackgroundColor;
            GuiProperties GuiProperties_CheckboxTextHighlightColor;
            GuiProperties GuiProperties_CheckboxTextFont;
            GuiProperties GuiProperties_CheckboxTextColor;
            GuiProperties GuiProperties_CheckboxClickAreaBorderColor;
            GuiProperties GuiProperties_CheckboxClickAreaBackgroundColor;
            GuiProperties GuiProperties_CheckboxClickAreaHighlightColor;
            GuiProperties GuiProperties_CheckboxClickAreaSelectedColor;
            GuiProperties GuiProperties_ScrollAreaHScrollShow;
            GuiProperties GuiProperties_ScrollAreaVScrollShow;
            GuiProperties GuiProperties_ScrollAreaScrollScale;
            GuiProperties GuiProperties_ScrollAreaScrollbarLength;
            GuiProperties GuiProperties_ScrollAreaScrollbarBackgroundColor;
            GuiProperties GuiProperties_ScrollAreaScrollbarVisibilityFading;
            GuiProperties GuiProperties_ScrollAreaScrollbarVisibilityStableTime;
            GuiProperties GuiProperties_ScrollAreaScrollbarVisibilityFadeTime;
            GuiProperties GuiProperties_ScrollAreaScrollbarKnobColor;

            cstr          GuiProperties_ToString(GuiProperties);
        ]]
    end

    do -- Global Symbol Table
        GuiProperties = {
            Opacity                                 = libphx.GuiProperties_Opacity,
            BorderColor                             = libphx.GuiProperties_BorderColor,
            BackgroundColor                         = libphx.GuiProperties_BackgroundColor,
            HighlightColor                          = libphx.GuiProperties_HighlightColor,
            TextFont                                = libphx.GuiProperties_TextFont,
            TextColor                               = libphx.GuiProperties_TextColor,
            ContainerClip                           = libphx.GuiProperties_ContainerClip,
            ContainerSpacing                        = libphx.GuiProperties_ContainerSpacing,
            ContainerColorPrimary                   = libphx.GuiProperties_ContainerColorPrimary,
            ButtonRectOpacity                       = libphx.GuiProperties_ButtonRectOpacity,
            ButtonRectBorderColor                   = libphx.GuiProperties_ButtonRectBorderColor,
            ButtonRectBackgroundColor               = libphx.GuiProperties_ButtonRectBackgroundColor,
            ButtonRectHighlightColor                = libphx.GuiProperties_ButtonRectHighlightColor,
            ButtonTextOpacity                       = libphx.GuiProperties_ButtonTextOpacity,
            ButtonTextBackgroundColor               = libphx.GuiProperties_ButtonTextBackgroundColor,
            ButtonTextHighlightColor                = libphx.GuiProperties_ButtonTextHighlightColor,
            ButtonTextFont                          = libphx.GuiProperties_ButtonTextFont,
            ButtonTextColor                         = libphx.GuiProperties_ButtonTextColor,
            CheckboxRectOpacity                     = libphx.GuiProperties_CheckboxRectOpacity,
            CheckboxRectBorderColor                 = libphx.GuiProperties_CheckboxRectBorderColor,
            CheckboxRectBackgroundColor             = libphx.GuiProperties_CheckboxRectBackgroundColor,
            CheckboxRectHighlightColor              = libphx.GuiProperties_CheckboxRectHighlightColor,
            CheckboxTextOpacity                     = libphx.GuiProperties_CheckboxTextOpacity,
            CheckboxTextBackgroundColor             = libphx.GuiProperties_CheckboxTextBackgroundColor,
            CheckboxTextHighlightColor              = libphx.GuiProperties_CheckboxTextHighlightColor,
            CheckboxTextFont                        = libphx.GuiProperties_CheckboxTextFont,
            CheckboxTextColor                       = libphx.GuiProperties_CheckboxTextColor,
            CheckboxClickAreaBorderColor            = libphx.GuiProperties_CheckboxClickAreaBorderColor,
            CheckboxClickAreaBackgroundColor        = libphx.GuiProperties_CheckboxClickAreaBackgroundColor,
            CheckboxClickAreaHighlightColor         = libphx.GuiProperties_CheckboxClickAreaHighlightColor,
            CheckboxClickAreaSelectedColor          = libphx.GuiProperties_CheckboxClickAreaSelectedColor,
            ScrollAreaHScrollShow                   = libphx.GuiProperties_ScrollAreaHScrollShow,
            ScrollAreaVScrollShow                   = libphx.GuiProperties_ScrollAreaVScrollShow,
            ScrollAreaScrollScale                   = libphx.GuiProperties_ScrollAreaScrollScale,
            ScrollAreaScrollbarLength               = libphx.GuiProperties_ScrollAreaScrollbarLength,
            ScrollAreaScrollbarBackgroundColor      = libphx.GuiProperties_ScrollAreaScrollbarBackgroundColor,
            ScrollAreaScrollbarVisibilityFading     = libphx.GuiProperties_ScrollAreaScrollbarVisibilityFading,
            ScrollAreaScrollbarVisibilityStableTime = libphx.GuiProperties_ScrollAreaScrollbarVisibilityStableTime,
            ScrollAreaScrollbarVisibilityFadeTime   = libphx.GuiProperties_ScrollAreaScrollbarVisibilityFadeTime,
            ScrollAreaScrollbarKnobColor            = libphx.GuiProperties_ScrollAreaScrollbarKnobColor,

            ToString                                = libphx.GuiProperties_ToString,
        }

        if onDef_GuiProperties then onDef_GuiProperties(GuiProperties, mt) end
        GuiProperties = setmetatable(GuiProperties, mt)
    end

    return GuiProperties
end

return Loader
