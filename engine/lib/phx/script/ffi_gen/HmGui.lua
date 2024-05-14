-- HmGui -----------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef struct HmGui {} HmGui;
    ]]

    return 1, 'HmGui'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local HmGui

    do -- C Definitions
        ffi.cdef [[
            void                    HmGui_Free                           (HmGui*);
            void                    HmGui_BeginGui                       (HmGui*, float sx, float sy, Input const* input);
            void                    HmGui_EndGui                         (HmGui*, Input const* input);
            void                    HmGui_Draw                           (HmGui*);
            void                    HmGui_BeginContainer                 (HmGui*, GuiLayoutType layout);
            void                    HmGui_BeginStackContainer            (HmGui*);
            void                    HmGui_BeginHorizontalContainer       (HmGui*);
            void                    HmGui_BeginVerticalContainer         (HmGui*);
            void                    HmGui_EndContainer                   (HmGui*);
            void                    HmGui_BeginScrollArea                (HmGui*, ScrollDirection dir);
            void                    HmGui_EndScrollArea                  (HmGui*, Input const* input);
            void                    HmGui_BeginWindow                    (HmGui*, cstr title, Input const* input);
            void                    HmGui_EndWindow                      (HmGui*);
            void                    HmGui_Spacer                         (HmGui*);
            bool                    HmGui_Button                         (HmGui*, cstr label);
            bool                    HmGui_Checkbox                       (HmGui*, cstr label, bool value);
            float                   HmGui_Slider                         (HmGui*, float lower, float upper, float value);
            void                    HmGui_HorizontalDivider              (HmGui*, float height);
            void                    HmGui_VerticalDivider                (HmGui*, float width);
            void                    HmGui_Image                          (HmGui*, Tex2D* image);
            void                    HmGui_Rect                           (HmGui*);
            void                    HmGui_Text                           (HmGui*, cstr text);
            void                    HmGui_TextColored                    (HmGui*, cstr text, Color const* color);
            void                    HmGui_TextEx                         (HmGui*, Font const* font, cstr text, Color const* color);
            bool                    HmGui_IsMouseOver                    (HmGui const*, FocusType ty);
            void                    HmGui_SetMinWidth                    (HmGui const*, float width);
            void                    HmGui_SetMinHeight                   (HmGui const*, float height);
            void                    HmGui_SetMinSize                     (HmGui const*, float width, float height);
            void                    HmGui_SetFixedWidth                  (HmGui const*, float width);
            void                    HmGui_SetFixedHeight                 (HmGui const*, float height);
            void                    HmGui_SetFixedSize                   (HmGui const*, float width, float height);
            void                    HmGui_SetPercentWidth                (HmGui const*, float width);
            void                    HmGui_SetPercentHeight               (HmGui const*, float height);
            void                    HmGui_SetPercentSize                 (HmGui const*, float width, float height);
            void                    HmGui_SetMargin                      (HmGui const*, float px, float py);
            void                    HmGui_SetMarginEx                    (HmGui const*, float left, float top, float right, float bottom);
            void                    HmGui_SetMarginLeft                  (HmGui const*, float margin);
            void                    HmGui_SetMarginTop                   (HmGui const*, float margin);
            void                    HmGui_SetMarginRight                 (HmGui const*, float margin);
            void                    HmGui_SetMarginBottom                (HmGui const*, float margin);
            void                    HmGui_SetBorderWidth                 (HmGui const*, float width);
            void                    HmGui_SetAlignment                   (HmGui const*, AlignHorizontal h, AlignVertical v);
            void                    HmGui_SetHorizontalAlignment         (HmGui const*, AlignHorizontal align);
            void                    HmGui_SetVerticalAlignment           (HmGui const*, AlignVertical align);
            void                    HmGui_SetPadding                     (HmGui const*, float px, float py);
            void                    HmGui_SetPaddingEx                   (HmGui const*, float left, float top, float right, float bottom);
            void                    HmGui_SetPaddingLeft                 (HmGui const*, float padding);
            void                    HmGui_SetPaddingTop                  (HmGui const*, float padding);
            void                    HmGui_SetPaddingRight                (HmGui const*, float padding);
            void                    HmGui_SetPaddingBottom               (HmGui const*, float padding);
            void                    HmGui_SetSpacing                     (HmGui const*, float spacing);
            void                    HmGui_SetChildrenAlignment           (HmGui const*, AlignHorizontal h, AlignVertical v);
            void                    HmGui_SetChildrenHorizontalAlignment (HmGui const*, AlignHorizontal align);
            void                    HmGui_SetChildrenVerticalAlignment   (HmGui const*, AlignVertical align);
            void                    HmGui_SetTheme                       (HmGui*, cstr name);
            void                    HmGui_ClearTheme                     (HmGui*);
            uint64                  HmGui_GetStyleId                     (HmGui const*, cstr name);
            void                    HmGui_SetStyle                       (HmGui*, uint64 id);
            void                    HmGui_SetStyleByName                 (HmGui*, cstr name);
            void                    HmGui_ClearStyle                     (HmGui*);
            GuiPropertyType         HmGui_GetPropertyType                (HmGui const*, uint64 id);
            void                    HmGui_MapProperty                    (HmGui*, uint64 propertyId);
            void                    HmGui_MapPropertyGroup               (HmGui*, cstr group);
            void                    HmGui_RemoveProperty                 (HmGui*, uint64 propertyId);
            uint64                  HmGui_RegisterProperty               (HmGui*, cstr name, GuiPropertyValue const* value, cstr mapId);
            void                    HmGui_SetProperty                    (HmGui*, uint64 id, GuiPropertyValue const* value);
            GuiPropertyValue const* HmGui_GetProperty                    (HmGui const*, uint64 id);
            uint64                  HmGui_GetPropertiesCount             (HmGui const*);
            void                    HmGui_DumpWidgets                    (HmGui const*);
        ]]
    end

    do -- Global Symbol Table
        HmGui = {}

        if onDef_HmGui then onDef_HmGui(HmGui, mt) end
        HmGui = setmetatable(HmGui, mt)
    end

    do -- Metatype for class instances
        local t  = ffi.typeof('HmGui')
        local mt = {
            __index = {
                beginGui                       = libphx.HmGui_BeginGui,
                endGui                         = libphx.HmGui_EndGui,
                draw                           = libphx.HmGui_Draw,
                beginContainer                 = libphx.HmGui_BeginContainer,
                beginStackContainer            = libphx.HmGui_BeginStackContainer,
                beginHorizontalContainer       = libphx.HmGui_BeginHorizontalContainer,
                beginVerticalContainer         = libphx.HmGui_BeginVerticalContainer,
                endContainer                   = libphx.HmGui_EndContainer,
                beginScrollArea                = libphx.HmGui_BeginScrollArea,
                endScrollArea                  = libphx.HmGui_EndScrollArea,
                beginWindow                    = libphx.HmGui_BeginWindow,
                endWindow                      = libphx.HmGui_EndWindow,
                spacer                         = libphx.HmGui_Spacer,
                button                         = libphx.HmGui_Button,
                checkbox                       = libphx.HmGui_Checkbox,
                slider                         = libphx.HmGui_Slider,
                horizontalDivider              = libphx.HmGui_HorizontalDivider,
                verticalDivider                = libphx.HmGui_VerticalDivider,
                image                          = libphx.HmGui_Image,
                rect                           = libphx.HmGui_Rect,
                text                           = libphx.HmGui_Text,
                textColored                    = libphx.HmGui_TextColored,
                textEx                         = libphx.HmGui_TextEx,
                isMouseOver                    = libphx.HmGui_IsMouseOver,
                setMinWidth                    = libphx.HmGui_SetMinWidth,
                setMinHeight                   = libphx.HmGui_SetMinHeight,
                setMinSize                     = libphx.HmGui_SetMinSize,
                setFixedWidth                  = libphx.HmGui_SetFixedWidth,
                setFixedHeight                 = libphx.HmGui_SetFixedHeight,
                setFixedSize                   = libphx.HmGui_SetFixedSize,
                setPercentWidth                = libphx.HmGui_SetPercentWidth,
                setPercentHeight               = libphx.HmGui_SetPercentHeight,
                setPercentSize                 = libphx.HmGui_SetPercentSize,
                setMargin                      = libphx.HmGui_SetMargin,
                setMarginEx                    = libphx.HmGui_SetMarginEx,
                setMarginLeft                  = libphx.HmGui_SetMarginLeft,
                setMarginTop                   = libphx.HmGui_SetMarginTop,
                setMarginRight                 = libphx.HmGui_SetMarginRight,
                setMarginBottom                = libphx.HmGui_SetMarginBottom,
                setBorderWidth                 = libphx.HmGui_SetBorderWidth,
                setAlignment                   = libphx.HmGui_SetAlignment,
                setHorizontalAlignment         = libphx.HmGui_SetHorizontalAlignment,
                setVerticalAlignment           = libphx.HmGui_SetVerticalAlignment,
                setPadding                     = libphx.HmGui_SetPadding,
                setPaddingEx                   = libphx.HmGui_SetPaddingEx,
                setPaddingLeft                 = libphx.HmGui_SetPaddingLeft,
                setPaddingTop                  = libphx.HmGui_SetPaddingTop,
                setPaddingRight                = libphx.HmGui_SetPaddingRight,
                setPaddingBottom               = libphx.HmGui_SetPaddingBottom,
                setSpacing                     = libphx.HmGui_SetSpacing,
                setChildrenAlignment           = libphx.HmGui_SetChildrenAlignment,
                setChildrenHorizontalAlignment = libphx.HmGui_SetChildrenHorizontalAlignment,
                setChildrenVerticalAlignment   = libphx.HmGui_SetChildrenVerticalAlignment,
                setTheme                       = libphx.HmGui_SetTheme,
                clearTheme                     = libphx.HmGui_ClearTheme,
                getStyleId                     = libphx.HmGui_GetStyleId,
                setStyle                       = libphx.HmGui_SetStyle,
                setStyleByName                 = libphx.HmGui_SetStyleByName,
                clearStyle                     = libphx.HmGui_ClearStyle,
                getPropertyType                = libphx.HmGui_GetPropertyType,
                mapProperty                    = libphx.HmGui_MapProperty,
                mapPropertyGroup               = libphx.HmGui_MapPropertyGroup,
                removeProperty                 = libphx.HmGui_RemoveProperty,
                registerProperty               = libphx.HmGui_RegisterProperty,
                setProperty                    = libphx.HmGui_SetProperty,
                getProperty                    = libphx.HmGui_GetProperty,
                getPropertiesCount             = libphx.HmGui_GetPropertiesCount,
                dumpWidgets                    = libphx.HmGui_DumpWidgets,
            },
        }

        if onDef_HmGui_t then onDef_HmGui_t(t, mt) end
        HmGui_t = ffi.metatype(t, mt)
    end

    return HmGui
end

return Loader
