-- HmGuiFfi --------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef struct HmGuiFfi {} HmGuiFfi;
    ]]

    return 1, 'HmGuiFfi'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local HmGuiFfi

    do -- C Definitions
        ffi.cdef [[
            void  HmGuiFfi_Begin            (HmGuiFfi*, float sx, float sy, Input const* input);
            void  HmGuiFfi_End              (HmGuiFfi*, Input const* input);
            void  HmGuiFfi_Draw             (HmGuiFfi*);
            void  HmGuiFfi_BeginGroupX      (HmGuiFfi*);
            void  HmGuiFfi_BeginGroupY      (HmGuiFfi*);
            void  HmGuiFfi_BeginGroupStack  (HmGuiFfi*);
            void  HmGuiFfi_EndGroup         (HmGuiFfi*);
            void  HmGuiFfi_BeginScroll      (HmGuiFfi*, float maxSize);
            void  HmGuiFfi_EndScroll        (HmGuiFfi*, Input const* input);
            void  HmGuiFfi_BeginWindow      (HmGuiFfi*, cstr title, Input const* input);
            void  HmGuiFfi_EndWindow        (HmGuiFfi*);
            bool  HmGuiFfi_Button           (HmGuiFfi*, cstr label);
            bool  HmGuiFfi_Checkbox         (HmGuiFfi*, cstr label, bool value);
            float HmGuiFfi_Slider           (HmGuiFfi*, float lower, float upper, float value);
            void  HmGuiFfi_Image            (HmGuiFfi*, Tex2D* image);
            void  HmGuiFfi_Rect             (HmGuiFfi*, float sx, float sy, float r, float g, float b, float a);
            void  HmGuiFfi_Text             (HmGuiFfi*, cstr text);
            void  HmGuiFfi_TextColored      (HmGuiFfi*, cstr text, float r, float g, float b, float a);
            void  HmGuiFfi_TextEx           (HmGuiFfi*, Font* font, cstr text, float r, float g, float b, float a);
            void  HmGuiFfi_SetAlign         (HmGuiFfi*, float ax, float ay);
            void  HmGuiFfi_SetPadding       (HmGuiFfi*, float px, float py);
            void  HmGuiFfi_SetPaddingEx     (HmGuiFfi*, float left, float top, float right, float bottom);
            void  HmGuiFfi_SetPaddingLeft   (HmGuiFfi*, float padding);
            void  HmGuiFfi_SetPaddingTop    (HmGuiFfi*, float padding);
            void  HmGuiFfi_SetPaddingRight  (HmGuiFfi*, float padding);
            void  HmGuiFfi_SetPaddingBottom (HmGuiFfi*, float padding);
            void  HmGuiFfi_SetSpacing       (HmGuiFfi*, float spacing);
            void  HmGuiFfi_SetStretch       (HmGuiFfi*, float x, float y);
            bool  HmGuiFfi_GroupHasFocus    (HmGuiFfi*, FocusType ty);
            void  HmGuiFfi_PushStyle        (HmGuiFfi*);
            void  HmGuiFfi_PushFont         (HmGuiFfi*, Font* font);
            void  HmGuiFfi_PushTextColor    (HmGuiFfi*, float r, float g, float b, float a);
            void  HmGuiFfi_PopStyle         (HmGuiFfi*, int depth);
        ]]
    end

    do -- Global Symbol Table
        HmGuiFfi = {
            Begin            = libphx.HmGuiFfi_Begin,
            End              = libphx.HmGuiFfi_End,
            Draw             = libphx.HmGuiFfi_Draw,
            BeginGroupX      = libphx.HmGuiFfi_BeginGroupX,
            BeginGroupY      = libphx.HmGuiFfi_BeginGroupY,
            BeginGroupStack  = libphx.HmGuiFfi_BeginGroupStack,
            EndGroup         = libphx.HmGuiFfi_EndGroup,
            BeginScroll      = libphx.HmGuiFfi_BeginScroll,
            EndScroll        = libphx.HmGuiFfi_EndScroll,
            BeginWindow      = libphx.HmGuiFfi_BeginWindow,
            EndWindow        = libphx.HmGuiFfi_EndWindow,
            Button           = libphx.HmGuiFfi_Button,
            Checkbox         = libphx.HmGuiFfi_Checkbox,
            Slider           = libphx.HmGuiFfi_Slider,
            Image            = libphx.HmGuiFfi_Image,
            Rect             = libphx.HmGuiFfi_Rect,
            Text             = libphx.HmGuiFfi_Text,
            TextColored      = libphx.HmGuiFfi_TextColored,
            TextEx           = libphx.HmGuiFfi_TextEx,
            SetAlign         = libphx.HmGuiFfi_SetAlign,
            SetPadding       = libphx.HmGuiFfi_SetPadding,
            SetPaddingEx     = libphx.HmGuiFfi_SetPaddingEx,
            SetPaddingLeft   = libphx.HmGuiFfi_SetPaddingLeft,
            SetPaddingTop    = libphx.HmGuiFfi_SetPaddingTop,
            SetPaddingRight  = libphx.HmGuiFfi_SetPaddingRight,
            SetPaddingBottom = libphx.HmGuiFfi_SetPaddingBottom,
            SetSpacing       = libphx.HmGuiFfi_SetSpacing,
            SetStretch       = libphx.HmGuiFfi_SetStretch,
            GroupHasFocus    = libphx.HmGuiFfi_GroupHasFocus,
            PushStyle        = libphx.HmGuiFfi_PushStyle,
            PushFont         = libphx.HmGuiFfi_PushFont,
            PushTextColor    = libphx.HmGuiFfi_PushTextColor,
            PopStyle         = libphx.HmGuiFfi_PopStyle,
        }

        if onDef_HmGuiFfi then onDef_HmGuiFfi(HmGuiFfi, mt) end
        HmGuiFfi = setmetatable(HmGuiFfi, mt)
    end

    do -- Metatype for class instances
        local t  = ffi.typeof('HmGuiFfi')
        local mt = {
            __index = {
                begin            = libphx.HmGuiFfi_Begin,
                end              = libphx.HmGuiFfi_End,
                draw             = libphx.HmGuiFfi_Draw,
                beginGroupX      = libphx.HmGuiFfi_BeginGroupX,
                beginGroupY      = libphx.HmGuiFfi_BeginGroupY,
                beginGroupStack  = libphx.HmGuiFfi_BeginGroupStack,
                endGroup         = libphx.HmGuiFfi_EndGroup,
                beginScroll      = libphx.HmGuiFfi_BeginScroll,
                endScroll        = libphx.HmGuiFfi_EndScroll,
                beginWindow      = libphx.HmGuiFfi_BeginWindow,
                endWindow        = libphx.HmGuiFfi_EndWindow,
                button           = libphx.HmGuiFfi_Button,
                checkbox         = libphx.HmGuiFfi_Checkbox,
                slider           = libphx.HmGuiFfi_Slider,
                image            = libphx.HmGuiFfi_Image,
                rect             = libphx.HmGuiFfi_Rect,
                text             = libphx.HmGuiFfi_Text,
                textColored      = libphx.HmGuiFfi_TextColored,
                textEx           = libphx.HmGuiFfi_TextEx,
                setAlign         = libphx.HmGuiFfi_SetAlign,
                setPadding       = libphx.HmGuiFfi_SetPadding,
                setPaddingEx     = libphx.HmGuiFfi_SetPaddingEx,
                setPaddingLeft   = libphx.HmGuiFfi_SetPaddingLeft,
                setPaddingTop    = libphx.HmGuiFfi_SetPaddingTop,
                setPaddingRight  = libphx.HmGuiFfi_SetPaddingRight,
                setPaddingBottom = libphx.HmGuiFfi_SetPaddingBottom,
                setSpacing       = libphx.HmGuiFfi_SetSpacing,
                setStretch       = libphx.HmGuiFfi_SetStretch,
                groupHasFocus    = libphx.HmGuiFfi_GroupHasFocus,
                pushStyle        = libphx.HmGuiFfi_PushStyle,
                pushFont         = libphx.HmGuiFfi_PushFont,
                pushTextColor    = libphx.HmGuiFfi_PushTextColor,
                popStyle         = libphx.HmGuiFfi_PopStyle,
            },
        }

        if onDef_HmGuiFfi_t then onDef_HmGuiFfi_t(t, mt) end
        HmGuiFfi_t = ffi.metatype(t, mt)
    end

    return HmGuiFfi
end

return Loader
