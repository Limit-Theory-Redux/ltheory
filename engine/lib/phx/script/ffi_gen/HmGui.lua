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
            void  HmGui_Free                           (HmGui*);
            void  HmGui_BeginGui                       (HmGui*, float sx, float sy);
            void  HmGui_EndGui                         (HmGui*, Input const* input);
            void  HmGui_Draw                           (HmGui*);
            void  HmGui_BeginLayer                     (HmGui*);
            void  HmGui_BeginLayerAtPos                (HmGui*, Vec2f pos);
            void  HmGui_BeginLayerBelow                (HmGui*);
            void  HmGui_EndLayer                       (HmGui*);
            void  HmGui_BeginContainer                 (HmGui*, GuiLayoutType layout);
            void  HmGui_BeginStackContainer            (HmGui*);
            void  HmGui_BeginHorizontalContainer       (HmGui*);
            void  HmGui_BeginVerticalContainer         (HmGui*);
            void  HmGui_EndContainer                   (HmGui*);
            Vec2f HmGui_UpdateContainerOffset          (HmGui*, Vec2f offset);
            Vec2f HmGui_ElementSize                    (HmGui*);
            Vec2f HmGui_ContainerSize                  (HmGui*);
            Vec2f HmGui_ContainerMinSize               (HmGui*);
            Vec2f HmGui_ContainerPos                   (HmGui*);
            void  HmGui_UpdateElementOffset            (HmGui*, Vec2f offset);
            void  HmGui_Image                          (HmGui*, Tex2D* image);
            void  HmGui_Rect                           (HmGui*);
            void  HmGui_Text                           (HmGui*, cstr text, Font const* font, Color const* color);
            void  HmGui_TextView                       (HmGui*, TextData* textData, bool editable);
            bool  HmGui_IsMouseOver                    (HmGui const*, FocusType ty);
            void  HmGui_SetFocus                       (HmGui*);
            bool  HmGui_HasFocus                       (HmGui const*);
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
            void  HmGui_SetAlignment                   (HmGui const*, AlignHorizontal h, AlignVertical v);
            void  HmGui_SetHorizontalAlignment         (HmGui const*, AlignHorizontal align);
            void  HmGui_SetVerticalAlignment           (HmGui const*, AlignVertical align);
            void  HmGui_SetBorderColor                 (HmGui const*, Color const* color);
            void  HmGui_SetBackgroundColor             (HmGui const*, Color const* color);
            void  HmGui_SetOpacity                     (HmGui const*, float opacity);
            void  HmGui_SetClipping                    (HmGui const*, bool clip);
            void  HmGui_SetPadding                     (HmGui const*, float px, float py);
            void  HmGui_SetPaddingEx                   (HmGui const*, float left, float top, float right, float bottom);
            void  HmGui_SetPaddingLeft                 (HmGui const*, float padding);
            void  HmGui_SetPaddingTop                  (HmGui const*, float padding);
            void  HmGui_SetPaddingRight                (HmGui const*, float padding);
            void  HmGui_SetPaddingBottom               (HmGui const*, float padding);
            void  HmGui_SetSpacing                     (HmGui const*, float spacing);
            void  HmGui_SetChildrenAlignment           (HmGui const*, AlignHorizontal h, AlignVertical v);
            void  HmGui_SetChildrenHorizontalAlignment (HmGui const*, AlignHorizontal align);
            void  HmGui_SetChildrenVerticalAlignment   (HmGui const*, AlignVertical align);
            void  HmGui_DumpWidgets                    (HmGui const*);
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
                beginLayer                     = libphx.HmGui_BeginLayer,
                beginLayerAtPos                = libphx.HmGui_BeginLayerAtPos,
                beginLayerBelow                = libphx.HmGui_BeginLayerBelow,
                endLayer                       = libphx.HmGui_EndLayer,
                beginContainer                 = libphx.HmGui_BeginContainer,
                beginStackContainer            = libphx.HmGui_BeginStackContainer,
                beginHorizontalContainer       = libphx.HmGui_BeginHorizontalContainer,
                beginVerticalContainer         = libphx.HmGui_BeginVerticalContainer,
                endContainer                   = libphx.HmGui_EndContainer,
                updateContainerOffset          = libphx.HmGui_UpdateContainerOffset,
                elementSize                    = libphx.HmGui_ElementSize,
                containerSize                  = libphx.HmGui_ContainerSize,
                containerMinSize               = libphx.HmGui_ContainerMinSize,
                containerPos                   = libphx.HmGui_ContainerPos,
                updateElementOffset            = libphx.HmGui_UpdateElementOffset,
                image                          = libphx.HmGui_Image,
                rect                           = libphx.HmGui_Rect,
                text                           = libphx.HmGui_Text,
                textView                       = libphx.HmGui_TextView,
                isMouseOver                    = libphx.HmGui_IsMouseOver,
                setFocus                       = libphx.HmGui_SetFocus,
                hasFocus                       = libphx.HmGui_HasFocus,
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
                setBorderColor                 = libphx.HmGui_SetBorderColor,
                setBackgroundColor             = libphx.HmGui_SetBackgroundColor,
                setOpacity                     = libphx.HmGui_SetOpacity,
                setClipping                    = libphx.HmGui_SetClipping,
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
                dumpWidgets                    = libphx.HmGui_DumpWidgets,
            },
        }

        if onDef_HmGui_t then onDef_HmGui_t(t, mt) end
        HmGui_t = ffi.metatype(t, mt)
    end

    return HmGui
end

return Loader
