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
            void              HmGui_BeginGui                       (HmGui*, float sx, float sy, Input const* input);
            void              HmGui_EndGui                         (HmGui*, Input const* input);
            void              HmGui_Draw                           (HmGui*);
            void              HmGui_BeginHorizontalContainer       (HmGui*);
            void              HmGui_BeginVerticalContainer         (HmGui*);
            void              HmGui_BeginStackContainer            (HmGui*);
            void              HmGui_EndContainer                   (HmGui*);
            void              HmGui_BeginScroll                    (HmGui*, float maxSize);
            void              HmGui_EndScroll                      (HmGui*, Input const* input);
            void              HmGui_BeginWindow                    (HmGui*, cstr title, Input const* input);
            void              HmGui_EndWindow                      (HmGui*);
            void              HmGui_Spacer                         (HmGui*);
            bool              HmGui_Button                         (HmGui*, cstr label);
            bool              HmGui_Checkbox                       (HmGui*, cstr label, bool value);
            float             HmGui_Slider                         (HmGui*, float lower, float upper, float value);
            void              HmGui_HorizontalDivider              (HmGui*, float height, float r, float g, float b, float a);
            void              HmGui_VerticalDivider                (HmGui*, float width, float r, float g, float b, float a);
            void              HmGui_Image                          (HmGui*, Tex2D* image);
            void              HmGui_Rect                           (HmGui*, float r, float g, float b, float a);
            void              HmGui_Text                           (HmGui*, cstr text);
            void              HmGui_TextColored                    (HmGui*, cstr text, float r, float g, float b, float a);
            void              HmGui_TextEx                         (HmGui*, Font const* font, cstr text, float r, float g, float b, float a);
            void              HmGui_SetMinWidth                    (HmGui const*, float width);
            void              HmGui_SetMinHeight                   (HmGui const*, float height);
            void              HmGui_SetMinSize                     (HmGui const*, float width, float height);
            void              HmGui_SetFixedWidth                  (HmGui const*, float width);
            void              HmGui_SetFixedHeight                 (HmGui const*, float height);
            void              HmGui_SetFixedSize                   (HmGui const*, float width, float height);
            void              HmGui_SetPercentWidth                (HmGui const*, float width);
            void              HmGui_SetPercentHeight               (HmGui const*, float height);
            void              HmGui_SetPercentSize                 (HmGui const*, float width, float height);
            void              HmGui_SetMargin                      (HmGui const*, float px, float py);
            void              HmGui_SetMarginEx                    (HmGui const*, float left, float top, float right, float bottom);
            void              HmGui_SetMarginLeft                  (HmGui const*, float margin);
            void              HmGui_SetMarginTop                   (HmGui const*, float margin);
            void              HmGui_SetMarginRight                 (HmGui const*, float margin);
            void              HmGui_SetMarginBottom                (HmGui const*, float margin);
            void              HmGui_SetBorderWidth                 (HmGui const*, float width);
            void              HmGui_SetBorderColor                 (HmGui const*, float r, float g, float b, float a);
            void              HmGui_SetBorderColorV4               (HmGui const*, Vec4f const* color);
            void              HmGui_SetBorder                      (HmGui const*, float width, float r, float g, float b, float a);
            void              HmGui_SetBorderV4                    (HmGui const*, float width, Vec4f const* color);
            void              HmGui_SetBgColor                     (HmGui*, float r, float g, float b, float a);
            void              HmGui_SetBgColorV4                   (HmGui*, Vec4f const* color);
            void              HmGui_SetAlignment                   (HmGui const*, AlignHorizontal h, AlignVertical v);
            void              HmGui_SetHorizontalAlignment         (HmGui const*, AlignHorizontal align);
            void              HmGui_SetVerticalAlignment           (HmGui const*, AlignVertical align);
            void              HmGui_SetPadding                     (HmGui const*, float px, float py);
            void              HmGui_SetPaddingEx                   (HmGui const*, float left, float top, float right, float bottom);
            void              HmGui_SetPaddingLeft                 (HmGui const*, float padding);
            void              HmGui_SetPaddingTop                  (HmGui const*, float padding);
            void              HmGui_SetPaddingRight                (HmGui const*, float padding);
            void              HmGui_SetPaddingBottom               (HmGui const*, float padding);
            void              HmGui_SetSpacing                     (HmGui const*, float spacing);
            bool              HmGui_ContainerHasFocus              (HmGui const*, FocusType ty);
            void              HmGui_SetChildrenAlignment           (HmGui const*, AlignHorizontal h, AlignVertical v);
            void              HmGui_SetChildrenHorizontalAlignment (HmGui const*, AlignHorizontal align);
            void              HmGui_SetChildrenVerticalAlignment   (HmGui const*, AlignVertical align);
            HmGuiPropertyType HmGui_GetPropertyType                (HmGui const*, uint64 id);
            void              HmGui_ClearStyle                     (HmGui*);
            void              HmGui_RemoveProperty                 (HmGui*, uint64 propertyId);
            void              HmGui_SetPropertyBool                (HmGui*, uint64 propertyId, bool value);
            void              HmGui_SetPropertyI8                  (HmGui*, uint64 propertyId, int8 value);
            void              HmGui_SetPropertyU8                  (HmGui*, uint64 propertyId, uint8 value);
            void              HmGui_SetPropertyI16                 (HmGui*, uint64 propertyId, int16 value);
            void              HmGui_SetPropertyU16                 (HmGui*, uint64 propertyId, uint16 value);
            void              HmGui_SetPropertyI32                 (HmGui*, uint64 propertyId, int value);
            void              HmGui_SetPropertyU32                 (HmGui*, uint64 propertyId, uint32 value);
            void              HmGui_SetPropertyI64                 (HmGui*, uint64 propertyId, int64 value);
            void              HmGui_SetPropertyU64                 (HmGui*, uint64 propertyId, uint64 value);
            void              HmGui_SetPropertyF32                 (HmGui*, uint64 propertyId, float value);
            void              HmGui_SetPropertyF64                 (HmGui*, uint64 propertyId, double value);
            void              HmGui_SetPropertyVec2                (HmGui*, uint64 propertyId, Vec2f value);
            void              HmGui_SetPropertyVec3                (HmGui*, uint64 propertyId, Vec3f const* value);
            void              HmGui_SetPropertyVec4                (HmGui*, uint64 propertyId, Vec4f const* value);
            void              HmGui_SetPropertyIVec2               (HmGui*, uint64 propertyId, Vec2i value);
            void              HmGui_SetPropertyIVec3               (HmGui*, uint64 propertyId, Vec3i const* value);
            void              HmGui_SetPropertyIVec4               (HmGui*, uint64 propertyId, Vec4i const* value);
            void              HmGui_SetPropertyUVec2               (HmGui*, uint64 propertyId, Vec2u value);
            void              HmGui_SetPropertyUVec3               (HmGui*, uint64 propertyId, Vec3u const* value);
            void              HmGui_SetPropertyUVec4               (HmGui*, uint64 propertyId, Vec4u const* value);
            void              HmGui_SetPropertyDVec2               (HmGui*, uint64 propertyId, Vec2d value);
            void              HmGui_SetPropertyDVec3               (HmGui*, uint64 propertyId, Vec3d const* value);
            void              HmGui_SetPropertyDVec4               (HmGui*, uint64 propertyId, Vec4d const* value);
            void              HmGui_SetPropertyBox3                (HmGui*, uint64 propertyId, Box3f const* value);
            void              HmGui_SetPropertyString              (HmGui*, uint64 propertyId, cstr value);
            void              HmGui_SetPropertyFont                (HmGui*, uint64 propertyId, Font const* value);
            bool              HmGui_GetPropertyBool                (HmGui const*, uint64 propertyId);
            int8              HmGui_GetPropertyI8                  (HmGui const*, uint64 propertyId);
            uint8             HmGui_GetPropertyU8                  (HmGui const*, uint64 propertyId);
            int16             HmGui_GetPropertyI16                 (HmGui const*, uint64 propertyId);
            uint16            HmGui_GetPropertyU16                 (HmGui const*, uint64 propertyId);
            int               HmGui_GetPropertyI32                 (HmGui const*, uint64 propertyId);
            uint32            HmGui_GetPropertyU32                 (HmGui const*, uint64 propertyId);
            int64             HmGui_GetPropertyI64                 (HmGui const*, uint64 propertyId);
            uint64            HmGui_GetPropertyU64                 (HmGui const*, uint64 propertyId);
            float             HmGui_GetPropertyF32                 (HmGui const*, uint64 propertyId);
            double            HmGui_GetPropertyF64                 (HmGui const*, uint64 propertyId);
            Vec2f             HmGui_GetPropertyVec2                (HmGui const*, uint64 propertyId);
            Vec3f const*      HmGui_GetPropertyVec3                (HmGui const*, uint64 propertyId);
            Vec4f const*      HmGui_GetPropertyVec4                (HmGui const*, uint64 propertyId);
            Vec2i             HmGui_GetPropertyIVec2               (HmGui const*, uint64 propertyId);
            Vec3i const*      HmGui_GetPropertyIVec3               (HmGui const*, uint64 propertyId);
            Vec4i const*      HmGui_GetPropertyIVec4               (HmGui const*, uint64 propertyId);
            Vec2u             HmGui_GetPropertyUVec2               (HmGui const*, uint64 propertyId);
            Vec3u const*      HmGui_GetPropertyUVec3               (HmGui const*, uint64 propertyId);
            Vec4u const*      HmGui_GetPropertyUVec4               (HmGui const*, uint64 propertyId);
            Vec2d             HmGui_GetPropertyDVec2               (HmGui const*, uint64 propertyId);
            Vec3d const*      HmGui_GetPropertyDVec3               (HmGui const*, uint64 propertyId);
            Vec4d const*      HmGui_GetPropertyDVec4               (HmGui const*, uint64 propertyId);
            Box3f const*      HmGui_GetPropertyBox3                (HmGui const*, uint64 propertyId);
            cstr              HmGui_GetPropertyString              (HmGui const*, uint64 propertyId);
            Font const*       HmGui_GetPropertyFont                (HmGui const*, uint64 propertyId);
            void              HmGui_DumpWidgets                    (HmGui const*);
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
            GetPropertyType                = libphx.HmGui_GetPropertyType,
            ClearStyle                     = libphx.HmGui_ClearStyle,
            RemoveProperty                 = libphx.HmGui_RemoveProperty,
            SetPropertyBool                = libphx.HmGui_SetPropertyBool,
            SetPropertyI8                  = libphx.HmGui_SetPropertyI8,
            SetPropertyU8                  = libphx.HmGui_SetPropertyU8,
            SetPropertyI16                 = libphx.HmGui_SetPropertyI16,
            SetPropertyU16                 = libphx.HmGui_SetPropertyU16,
            SetPropertyI32                 = libphx.HmGui_SetPropertyI32,
            SetPropertyU32                 = libphx.HmGui_SetPropertyU32,
            SetPropertyI64                 = libphx.HmGui_SetPropertyI64,
            SetPropertyU64                 = libphx.HmGui_SetPropertyU64,
            SetPropertyF32                 = libphx.HmGui_SetPropertyF32,
            SetPropertyF64                 = libphx.HmGui_SetPropertyF64,
            SetPropertyVec2                = libphx.HmGui_SetPropertyVec2,
            SetPropertyVec3                = libphx.HmGui_SetPropertyVec3,
            SetPropertyVec4                = libphx.HmGui_SetPropertyVec4,
            SetPropertyIVec2               = libphx.HmGui_SetPropertyIVec2,
            SetPropertyIVec3               = libphx.HmGui_SetPropertyIVec3,
            SetPropertyIVec4               = libphx.HmGui_SetPropertyIVec4,
            SetPropertyUVec2               = libphx.HmGui_SetPropertyUVec2,
            SetPropertyUVec3               = libphx.HmGui_SetPropertyUVec3,
            SetPropertyUVec4               = libphx.HmGui_SetPropertyUVec4,
            SetPropertyDVec2               = libphx.HmGui_SetPropertyDVec2,
            SetPropertyDVec3               = libphx.HmGui_SetPropertyDVec3,
            SetPropertyDVec4               = libphx.HmGui_SetPropertyDVec4,
            SetPropertyBox3                = libphx.HmGui_SetPropertyBox3,
            SetPropertyString              = libphx.HmGui_SetPropertyString,
            SetPropertyFont                = libphx.HmGui_SetPropertyFont,
            GetPropertyBool                = libphx.HmGui_GetPropertyBool,
            GetPropertyI8                  = libphx.HmGui_GetPropertyI8,
            GetPropertyU8                  = libphx.HmGui_GetPropertyU8,
            GetPropertyI16                 = libphx.HmGui_GetPropertyI16,
            GetPropertyU16                 = libphx.HmGui_GetPropertyU16,
            GetPropertyI32                 = libphx.HmGui_GetPropertyI32,
            GetPropertyU32                 = libphx.HmGui_GetPropertyU32,
            GetPropertyI64                 = libphx.HmGui_GetPropertyI64,
            GetPropertyU64                 = libphx.HmGui_GetPropertyU64,
            GetPropertyF32                 = libphx.HmGui_GetPropertyF32,
            GetPropertyF64                 = libphx.HmGui_GetPropertyF64,
            GetPropertyVec2                = libphx.HmGui_GetPropertyVec2,
            GetPropertyVec3                = libphx.HmGui_GetPropertyVec3,
            GetPropertyVec4                = libphx.HmGui_GetPropertyVec4,
            GetPropertyIVec2               = libphx.HmGui_GetPropertyIVec2,
            GetPropertyIVec3               = libphx.HmGui_GetPropertyIVec3,
            GetPropertyIVec4               = libphx.HmGui_GetPropertyIVec4,
            GetPropertyUVec2               = libphx.HmGui_GetPropertyUVec2,
            GetPropertyUVec3               = libphx.HmGui_GetPropertyUVec3,
            GetPropertyUVec4               = libphx.HmGui_GetPropertyUVec4,
            GetPropertyDVec2               = libphx.HmGui_GetPropertyDVec2,
            GetPropertyDVec3               = libphx.HmGui_GetPropertyDVec3,
            GetPropertyDVec4               = libphx.HmGui_GetPropertyDVec4,
            GetPropertyBox3                = libphx.HmGui_GetPropertyBox3,
            GetPropertyString              = libphx.HmGui_GetPropertyString,
            GetPropertyFont                = libphx.HmGui_GetPropertyFont,
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
                getPropertyType                = libphx.HmGui_GetPropertyType,
                clearStyle                     = libphx.HmGui_ClearStyle,
                removeProperty                 = libphx.HmGui_RemoveProperty,
                setPropertyBool                = libphx.HmGui_SetPropertyBool,
                setPropertyI8                  = libphx.HmGui_SetPropertyI8,
                setPropertyU8                  = libphx.HmGui_SetPropertyU8,
                setPropertyI16                 = libphx.HmGui_SetPropertyI16,
                setPropertyU16                 = libphx.HmGui_SetPropertyU16,
                setPropertyI32                 = libphx.HmGui_SetPropertyI32,
                setPropertyU32                 = libphx.HmGui_SetPropertyU32,
                setPropertyI64                 = libphx.HmGui_SetPropertyI64,
                setPropertyU64                 = libphx.HmGui_SetPropertyU64,
                setPropertyF32                 = libphx.HmGui_SetPropertyF32,
                setPropertyF64                 = libphx.HmGui_SetPropertyF64,
                setPropertyVec2                = libphx.HmGui_SetPropertyVec2,
                setPropertyVec3                = libphx.HmGui_SetPropertyVec3,
                setPropertyVec4                = libphx.HmGui_SetPropertyVec4,
                setPropertyIVec2               = libphx.HmGui_SetPropertyIVec2,
                setPropertyIVec3               = libphx.HmGui_SetPropertyIVec3,
                setPropertyIVec4               = libphx.HmGui_SetPropertyIVec4,
                setPropertyUVec2               = libphx.HmGui_SetPropertyUVec2,
                setPropertyUVec3               = libphx.HmGui_SetPropertyUVec3,
                setPropertyUVec4               = libphx.HmGui_SetPropertyUVec4,
                setPropertyDVec2               = libphx.HmGui_SetPropertyDVec2,
                setPropertyDVec3               = libphx.HmGui_SetPropertyDVec3,
                setPropertyDVec4               = libphx.HmGui_SetPropertyDVec4,
                setPropertyBox3                = libphx.HmGui_SetPropertyBox3,
                setPropertyString              = libphx.HmGui_SetPropertyString,
                setPropertyFont                = libphx.HmGui_SetPropertyFont,
                getPropertyBool                = libphx.HmGui_GetPropertyBool,
                getPropertyI8                  = libphx.HmGui_GetPropertyI8,
                getPropertyU8                  = libphx.HmGui_GetPropertyU8,
                getPropertyI16                 = libphx.HmGui_GetPropertyI16,
                getPropertyU16                 = libphx.HmGui_GetPropertyU16,
                getPropertyI32                 = libphx.HmGui_GetPropertyI32,
                getPropertyU32                 = libphx.HmGui_GetPropertyU32,
                getPropertyI64                 = libphx.HmGui_GetPropertyI64,
                getPropertyU64                 = libphx.HmGui_GetPropertyU64,
                getPropertyF32                 = libphx.HmGui_GetPropertyF32,
                getPropertyF64                 = libphx.HmGui_GetPropertyF64,
                getPropertyVec2                = libphx.HmGui_GetPropertyVec2,
                getPropertyVec3                = libphx.HmGui_GetPropertyVec3,
                getPropertyVec4                = libphx.HmGui_GetPropertyVec4,
                getPropertyIVec2               = libphx.HmGui_GetPropertyIVec2,
                getPropertyIVec3               = libphx.HmGui_GetPropertyIVec3,
                getPropertyIVec4               = libphx.HmGui_GetPropertyIVec4,
                getPropertyUVec2               = libphx.HmGui_GetPropertyUVec2,
                getPropertyUVec3               = libphx.HmGui_GetPropertyUVec3,
                getPropertyUVec4               = libphx.HmGui_GetPropertyUVec4,
                getPropertyDVec2               = libphx.HmGui_GetPropertyDVec2,
                getPropertyDVec3               = libphx.HmGui_GetPropertyDVec3,
                getPropertyDVec4               = libphx.HmGui_GetPropertyDVec4,
                getPropertyBox3                = libphx.HmGui_GetPropertyBox3,
                getPropertyString              = libphx.HmGui_GetPropertyString,
                getPropertyFont                = libphx.HmGui_GetPropertyFont,
                dumpWidgets                    = libphx.HmGui_DumpWidgets,
            },
        }

        if onDef_HmGui_t then onDef_HmGui_t(t, mt) end
        HmGui_t = ffi.metatype(t, mt)
    end

    return HmGui
end

return Loader
