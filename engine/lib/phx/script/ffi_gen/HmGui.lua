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
            void  HmGui_BeginGui                       (HmGui*, float sx, float sy, Input const* input);
            void  HmGui_EndGui                         (HmGui*, Input const* input);
            void  HmGui_Draw                           (HmGui*);
            void  HmGui_BeginHorizontalContainer       (HmGui*);
            void  HmGui_BeginVerticalContainer         (HmGui*);
            void  HmGui_BeginStackContainer            (HmGui*);
            void  HmGui_EndContainer                   (HmGui*);
            void  HmGui_BeginScroll                    (HmGui*, float maxSize);
            void  HmGui_EndScroll                      (HmGui*, Input const* input);
            void  HmGui_BeginWindow                    (HmGui*, cstr title, Input const* input);
            void  HmGui_EndWindow                      (HmGui*);
            void  HmGui_Spacer                         (HmGui*);
            bool  HmGui_Button                         (HmGui*, cstr label);
            bool  HmGui_Checkbox                       (HmGui*, cstr label, bool value);
            float HmGui_Slider                         (HmGui*, float lower, float upper, float value);
            void  HmGui_HorizontalDivider              (HmGui*, float height, float r, float g, float b, float a);
            void  HmGui_VerticalDivider                (HmGui*, float width, float r, float g, float b, float a);
            void  HmGui_Image                          (HmGui*, Tex2D* image);
            void  HmGui_Rect                           (HmGui*, float r, float g, float b, float a);
            void  HmGui_Text                           (HmGui*, cstr text);
            void  HmGui_TextColored                    (HmGui*, cstr text, float r, float g, float b, float a);
            void  HmGui_TextEx                         (HmGui*, Font const* font, cstr text, float r, float g, float b, float a);
            void  HmGui_SetMinWidth                    (HmGui const*, float width);
            void  HmGui_SetMinHeight                   (HmGui const*, float height);
            void  HmGui_SetMinSize                     (HmGui const*, float width, float height);
            void  HmGui_SetFixedWidth                  (HmGui const*, float width);
            void  HmGui_SetFixedHeight                 (HmGui const*, float height);
            void  HmGui_SetFixedSize                   (HmGui const*, float width, float height);
            void  HmGui_SetPercentWidth                (HmGui const*, float width);
            void  HmGui_SetPercentHeight               (HmGui const*, float height);
            void  HmGui_SetPercentSize                 (HmGui const*, float width, float height);
            void  HmGui_SetMargin                      (HmGui const*, float px, float py);
            void  HmGui_SetMarginEx                    (HmGui const*, float left, float top, float right, float bottom);
            void  HmGui_SetMarginLeft                  (HmGui const*, float margin);
            void  HmGui_SetMarginTop                   (HmGui const*, float margin);
            void  HmGui_SetMarginRight                 (HmGui const*, float margin);
            void  HmGui_SetMarginBottom                (HmGui const*, float margin);
            void  HmGui_SetBorderWidth                 (HmGui const*, float width);
            void  HmGui_SetBorderColor                 (HmGui const*, float r, float g, float b, float a);
            void  HmGui_SetBorderColorV4               (HmGui const*, Vec4f const* color);
            void  HmGui_SetBorder                      (HmGui const*, float width, float r, float g, float b, float a);
            void  HmGui_SetBorderV4                    (HmGui const*, float width, Vec4f const* color);
            void  HmGui_SetBgColor                     (HmGui*, float r, float g, float b, float a);
            void  HmGui_SetBgColorV4                   (HmGui*, Vec4f const* color);
            void  HmGui_SetAlignment                   (HmGui const*, AlignHorizontal h, AlignVertical v);
            void  HmGui_SetHorizontalAlignment         (HmGui const*, AlignHorizontal align);
            void  HmGui_SetVerticalAlignment           (HmGui const*, AlignVertical align);
            void  HmGui_SetPadding                     (HmGui const*, float px, float py);
            void  HmGui_SetPaddingEx                   (HmGui const*, float left, float top, float right, float bottom);
            void  HmGui_SetPaddingLeft                 (HmGui const*, float padding);
            void  HmGui_SetPaddingTop                  (HmGui const*, float padding);
            void  HmGui_SetPaddingRight                (HmGui const*, float padding);
            void  HmGui_SetPaddingBottom               (HmGui const*, float padding);
            void  HmGui_SetSpacing                     (HmGui const*, float spacing);
            bool  HmGui_ContainerHasFocus              (HmGui const*, FocusType ty);
            void  HmGui_SetChildrenAlignment           (HmGui const*, AlignHorizontal h, AlignVertical v);
            void  HmGui_SetChildrenHorizontalAlignment (HmGui const*, AlignHorizontal align);
            void  HmGui_SetChildrenVerticalAlignment   (HmGui const*, AlignVertical align);
            void  HmGui_PushStyle                      (HmGui*);
            void  HmGui_PushFont                       (HmGui*, Font const* font);
            void  HmGui_PushTextColor                  (HmGui*, float r, float g, float b, float a);
            void  HmGui_PopStyle                       (HmGui*, int depth);
            void  HmGui_DumpWidgets                    (HmGui const*);
        ]]
    end

    do -- Global Symbol Table
        HmGui = {
            BeginGui                       = libphx.HmGui_BeginGui,
            EndGui                         = libphx.HmGui_EndGui,
            Draw                           = libphx.HmGui_Draw,
            BeginHorizontalContainer       = libphx.HmGui_BeginHorizontalContainer,
            BeginVerticalContainer         = libphx.HmGui_BeginVerticalContainer,
            BeginStackContainer            = libphx.HmGui_BeginStackContainer,
            EndContainer                   = libphx.HmGui_EndContainer,
            BeginScroll                    = libphx.HmGui_BeginScroll,
            EndScroll                      = libphx.HmGui_EndScroll,
            BeginWindow                    = libphx.HmGui_BeginWindow,
            EndWindow                      = libphx.HmGui_EndWindow,
            Spacer                         = libphx.HmGui_Spacer,
            Button                         = libphx.HmGui_Button,
            Checkbox                       = libphx.HmGui_Checkbox,
            Slider                         = libphx.HmGui_Slider,
            HorizontalDivider              = libphx.HmGui_HorizontalDivider,
            VerticalDivider                = libphx.HmGui_VerticalDivider,
            Image                          = libphx.HmGui_Image,
            Rect                           = libphx.HmGui_Rect,
            Text                           = libphx.HmGui_Text,
            TextColored                    = libphx.HmGui_TextColored,
            TextEx                         = libphx.HmGui_TextEx,
            SetMinWidth                    = libphx.HmGui_SetMinWidth,
            SetMinHeight                   = libphx.HmGui_SetMinHeight,
            SetMinSize                     = libphx.HmGui_SetMinSize,
            SetFixedWidth                  = libphx.HmGui_SetFixedWidth,
            SetFixedHeight                 = libphx.HmGui_SetFixedHeight,
            SetFixedSize                   = libphx.HmGui_SetFixedSize,
            SetPercentWidth                = libphx.HmGui_SetPercentWidth,
            SetPercentHeight               = libphx.HmGui_SetPercentHeight,
            SetPercentSize                 = libphx.HmGui_SetPercentSize,
            SetMargin                      = libphx.HmGui_SetMargin,
            SetMarginEx                    = libphx.HmGui_SetMarginEx,
            SetMarginLeft                  = libphx.HmGui_SetMarginLeft,
            SetMarginTop                   = libphx.HmGui_SetMarginTop,
            SetMarginRight                 = libphx.HmGui_SetMarginRight,
            SetMarginBottom                = libphx.HmGui_SetMarginBottom,
            SetBorderWidth                 = libphx.HmGui_SetBorderWidth,
            SetBorderColor                 = libphx.HmGui_SetBorderColor,
            SetBorderColorV4               = libphx.HmGui_SetBorderColorV4,
            SetBorder                      = libphx.HmGui_SetBorder,
            SetBorderV4                    = libphx.HmGui_SetBorderV4,
            SetBgColor                     = libphx.HmGui_SetBgColor,
            SetBgColorV4                   = libphx.HmGui_SetBgColorV4,
            SetAlignment                   = libphx.HmGui_SetAlignment,
            SetHorizontalAlignment         = libphx.HmGui_SetHorizontalAlignment,
            SetVerticalAlignment           = libphx.HmGui_SetVerticalAlignment,
            SetPadding                     = libphx.HmGui_SetPadding,
            SetPaddingEx                   = libphx.HmGui_SetPaddingEx,
            SetPaddingLeft                 = libphx.HmGui_SetPaddingLeft,
            SetPaddingTop                  = libphx.HmGui_SetPaddingTop,
            SetPaddingRight                = libphx.HmGui_SetPaddingRight,
            SetPaddingBottom               = libphx.HmGui_SetPaddingBottom,
            SetSpacing                     = libphx.HmGui_SetSpacing,
            ContainerHasFocus              = libphx.HmGui_ContainerHasFocus,
            SetChildrenAlignment           = libphx.HmGui_SetChildrenAlignment,
            SetChildrenHorizontalAlignment = libphx.HmGui_SetChildrenHorizontalAlignment,
            SetChildrenVerticalAlignment   = libphx.HmGui_SetChildrenVerticalAlignment,
            PushStyle                      = libphx.HmGui_PushStyle,
            PushFont                       = libphx.HmGui_PushFont,
            PushTextColor                  = libphx.HmGui_PushTextColor,
            PopStyle                       = libphx.HmGui_PopStyle,
            DumpWidgets                    = libphx.HmGui_DumpWidgets,
        }

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
                beginHorizontalContainer       = libphx.HmGui_BeginHorizontalContainer,
                beginVerticalContainer         = libphx.HmGui_BeginVerticalContainer,
                beginStackContainer            = libphx.HmGui_BeginStackContainer,
                endContainer                   = libphx.HmGui_EndContainer,
                beginScroll                    = libphx.HmGui_BeginScroll,
                endScroll                      = libphx.HmGui_EndScroll,
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
                setBorderColor                 = libphx.HmGui_SetBorderColor,
                setBorderColorV4               = libphx.HmGui_SetBorderColorV4,
                setBorder                      = libphx.HmGui_SetBorder,
                setBorderV4                    = libphx.HmGui_SetBorderV4,
                setBgColor                     = libphx.HmGui_SetBgColor,
                setBgColorV4                   = libphx.HmGui_SetBgColorV4,
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
                containerHasFocus              = libphx.HmGui_ContainerHasFocus,
                setChildrenAlignment           = libphx.HmGui_SetChildrenAlignment,
                setChildrenHorizontalAlignment = libphx.HmGui_SetChildrenHorizontalAlignment,
                setChildrenVerticalAlignment   = libphx.HmGui_SetChildrenVerticalAlignment,
                pushStyle                      = libphx.HmGui_PushStyle,
                pushFont                       = libphx.HmGui_PushFont,
                pushTextColor                  = libphx.HmGui_PushTextColor,
                popStyle                       = libphx.HmGui_PopStyle,
                dumpWidgets                    = libphx.HmGui_DumpWidgets,
            },
        }

        if onDef_HmGui_t then onDef_HmGui_t(t, mt) end
        HmGui_t = ffi.metatype(t, mt)
    end

    return HmGui
end

return Loader
