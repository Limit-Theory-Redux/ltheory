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
            void               HmGui_Free                           (HmGui*);
            void               HmGui_BeginGui                       (HmGui*, float sx, float sy, Input const* input);
            void               HmGui_EndGui                         (HmGui*, Input const* input);
            void               HmGui_Draw                           (HmGui*);
            void               HmGui_BeginHorizontalContainer       (HmGui*);
            void               HmGui_BeginVerticalContainer         (HmGui*);
            void               HmGui_BeginStackContainer            (HmGui*);
            void               HmGui_EndContainer                   (HmGui*);
            void               HmGui_BeginScrollArea                (HmGui*, ScrollDirection dir);
            void               HmGui_EndScrollArea                  (HmGui*, Input const* input);
            void               HmGui_BeginWindow                    (HmGui*, cstr title, Input const* input);
            void               HmGui_EndWindow                      (HmGui*);
            void               HmGui_Spacer                         (HmGui*);
            bool               HmGui_Button                         (HmGui*, cstr label);
            bool               HmGui_Checkbox                       (HmGui*, cstr label, bool value);
            float              HmGui_Slider                         (HmGui*, float lower, float upper, float value);
            void               HmGui_HorizontalDivider              (HmGui*, float height, Color const* color);
            void               HmGui_VerticalDivider                (HmGui*, float width, Color const* color);
            void               HmGui_Image                          (HmGui*, Tex2D* image);
            void               HmGui_Rect                           (HmGui*, Color const* color);
            void               HmGui_Text                           (HmGui*, cstr text);
            void               HmGui_TextColored                    (HmGui*, cstr text, Color const* color);
            void               HmGui_TextEx                         (HmGui*, Font const* font, cstr text, Color const* color);
            bool               HmGui_IsMouseOver                    (HmGui const*, FocusType ty);
            void               HmGui_SetMinWidth                    (HmGui const*, float width);
            void               HmGui_SetMinHeight                   (HmGui const*, float height);
            void               HmGui_SetMinSize                     (HmGui const*, float width, float height);
            void               HmGui_SetFixedWidth                  (HmGui const*, float width);
            void               HmGui_SetFixedHeight                 (HmGui const*, float height);
            void               HmGui_SetFixedSize                   (HmGui const*, float width, float height);
            void               HmGui_SetPercentWidth                (HmGui const*, float width);
            void               HmGui_SetPercentHeight               (HmGui const*, float height);
            void               HmGui_SetPercentSize                 (HmGui const*, float width, float height);
            void               HmGui_SetMargin                      (HmGui const*, float px, float py);
            void               HmGui_SetMarginEx                    (HmGui const*, float left, float top, float right, float bottom);
            void               HmGui_SetMarginLeft                  (HmGui const*, float margin);
            void               HmGui_SetMarginTop                   (HmGui const*, float margin);
            void               HmGui_SetMarginRight                 (HmGui const*, float margin);
            void               HmGui_SetMarginBottom                (HmGui const*, float margin);
            void               HmGui_SetBorderWidth                 (HmGui const*, float width);
            void               HmGui_SetBorderColor                 (HmGui const*, Color const* color);
            void               HmGui_SetBorderColorV4               (HmGui const*, Color const* color);
            void               HmGui_SetBorder                      (HmGui const*, float width, Color const* color);
            void               HmGui_SetBorderV4                    (HmGui const*, float width, Color const* color);
            void               HmGui_SetBgColor                     (HmGui*, Color const* color);
            void               HmGui_SetBgColorV4                   (HmGui*, Color const* color);
            void               HmGui_SetAlignment                   (HmGui const*, AlignHorizontal h, AlignVertical v);
            void               HmGui_SetHorizontalAlignment         (HmGui const*, AlignHorizontal align);
            void               HmGui_SetVerticalAlignment           (HmGui const*, AlignVertical align);
            void               HmGui_SetPadding                     (HmGui const*, float px, float py);
            void               HmGui_SetPaddingEx                   (HmGui const*, float left, float top, float right, float bottom);
            void               HmGui_SetPaddingLeft                 (HmGui const*, float padding);
            void               HmGui_SetPaddingTop                  (HmGui const*, float padding);
            void               HmGui_SetPaddingRight                (HmGui const*, float padding);
            void               HmGui_SetPaddingBottom               (HmGui const*, float padding);
            void               HmGui_SetSpacing                     (HmGui const*, float spacing);
            void               HmGui_SetChildrenAlignment           (HmGui const*, AlignHorizontal h, AlignVertical v);
            void               HmGui_SetChildrenHorizontalAlignment (HmGui const*, AlignHorizontal align);
            void               HmGui_SetChildrenVerticalAlignment   (HmGui const*, AlignVertical align);
            void               HmGui_SetTheme                       (HmGui*, cstr name);
            void               HmGui_ClearTheme                     (HmGui*);
            uint64             HmGui_GetStyleId                     (HmGui const*, cstr name);
            void               HmGui_SetStyle                       (HmGui*, uint64 id);
            void               HmGui_ClearStyle                     (HmGui*);
            HmGuiPropertyType* HmGui_GetPropertyType                (HmGui const*, uint64 id);
            void               HmGui_MapProperty                    (HmGui*, uint64 propertyId);
            void               HmGui_MapPropertyGroup               (HmGui*, cstr group);
            void               HmGui_RemoveProperty                 (HmGui*, uint64 propertyId);
            uint64             HmGui_RegisterPropertyBool           (HmGui*, cstr name, bool value, cstr mapId);
            uint64             HmGui_RegisterPropertyI8             (HmGui*, cstr name, int8 value, cstr mapId);
            uint64             HmGui_RegisterPropertyU8             (HmGui*, cstr name, uint8 value, cstr mapId);
            uint64             HmGui_RegisterPropertyI16            (HmGui*, cstr name, int16 value, cstr mapId);
            uint64             HmGui_RegisterPropertyU16            (HmGui*, cstr name, uint16 value, cstr mapId);
            uint64             HmGui_RegisterPropertyI32            (HmGui*, cstr name, int value, cstr mapId);
            uint64             HmGui_RegisterPropertyU32            (HmGui*, cstr name, uint32 value, cstr mapId);
            uint64             HmGui_RegisterPropertyI64            (HmGui*, cstr name, int64 value, cstr mapId);
            uint64             HmGui_RegisterPropertyU64            (HmGui*, cstr name, uint64 value, cstr mapId);
            uint64             HmGui_RegisterPropertyF32            (HmGui*, cstr name, float value, cstr mapId);
            uint64             HmGui_RegisterPropertyF64            (HmGui*, cstr name, double value, cstr mapId);
            uint64             HmGui_RegisterPropertyVec2           (HmGui*, cstr name, Vec2f value, cstr mapId);
            uint64             HmGui_RegisterPropertyVec3           (HmGui*, cstr name, Vec3f const* value, cstr mapId);
            uint64             HmGui_RegisterPropertyVec4           (HmGui*, cstr name, Vec4f const* value, cstr mapId);
            uint64             HmGui_RegisterPropertyIVec2          (HmGui*, cstr name, Vec2i value, cstr mapId);
            uint64             HmGui_RegisterPropertyIVec3          (HmGui*, cstr name, Vec3i const* value, cstr mapId);
            uint64             HmGui_RegisterPropertyIVec4          (HmGui*, cstr name, Vec4i const* value, cstr mapId);
            uint64             HmGui_RegisterPropertyUVec2          (HmGui*, cstr name, Vec2u value, cstr mapId);
            uint64             HmGui_RegisterPropertyUVec3          (HmGui*, cstr name, Vec3u const* value, cstr mapId);
            uint64             HmGui_RegisterPropertyUVec4          (HmGui*, cstr name, Vec4u const* value, cstr mapId);
            uint64             HmGui_RegisterPropertyDVec2          (HmGui*, cstr name, Vec2d value, cstr mapId);
            uint64             HmGui_RegisterPropertyDVec3          (HmGui*, cstr name, Vec3d const* value, cstr mapId);
            uint64             HmGui_RegisterPropertyDVec4          (HmGui*, cstr name, Vec4d const* value, cstr mapId);
            uint64             HmGui_RegisterPropertyColor          (HmGui*, cstr name, Color const* value, cstr mapId);
            uint64             HmGui_RegisterPropertyBox3           (HmGui*, cstr name, Box3f const* value, cstr mapId);
            uint64             HmGui_RegisterPropertyString         (HmGui*, cstr name, cstr value, cstr mapId);
            uint64             HmGui_RegisterPropertyFont           (HmGui*, cstr name, Font const* value, cstr mapId);
            void               HmGui_SetPropertyBool                (HmGui*, uint64 propertyId, bool value);
            void               HmGui_SetPropertyI8                  (HmGui*, uint64 propertyId, int8 value);
            void               HmGui_SetPropertyU8                  (HmGui*, uint64 propertyId, uint8 value);
            void               HmGui_SetPropertyI16                 (HmGui*, uint64 propertyId, int16 value);
            void               HmGui_SetPropertyU16                 (HmGui*, uint64 propertyId, uint16 value);
            void               HmGui_SetPropertyI32                 (HmGui*, uint64 propertyId, int value);
            void               HmGui_SetPropertyU32                 (HmGui*, uint64 propertyId, uint32 value);
            void               HmGui_SetPropertyI64                 (HmGui*, uint64 propertyId, int64 value);
            void               HmGui_SetPropertyU64                 (HmGui*, uint64 propertyId, uint64 value);
            void               HmGui_SetPropertyF32                 (HmGui*, uint64 propertyId, float value);
            void               HmGui_SetPropertyF64                 (HmGui*, uint64 propertyId, double value);
            void               HmGui_SetPropertyVec2                (HmGui*, uint64 propertyId, Vec2f value);
            void               HmGui_SetPropertyVec3                (HmGui*, uint64 propertyId, Vec3f const* value);
            void               HmGui_SetPropertyVec4                (HmGui*, uint64 propertyId, Vec4f const* value);
            void               HmGui_SetPropertyIVec2               (HmGui*, uint64 propertyId, Vec2i value);
            void               HmGui_SetPropertyIVec3               (HmGui*, uint64 propertyId, Vec3i const* value);
            void               HmGui_SetPropertyIVec4               (HmGui*, uint64 propertyId, Vec4i const* value);
            void               HmGui_SetPropertyUVec2               (HmGui*, uint64 propertyId, Vec2u value);
            void               HmGui_SetPropertyUVec3               (HmGui*, uint64 propertyId, Vec3u const* value);
            void               HmGui_SetPropertyUVec4               (HmGui*, uint64 propertyId, Vec4u const* value);
            void               HmGui_SetPropertyDVec2               (HmGui*, uint64 propertyId, Vec2d value);
            void               HmGui_SetPropertyDVec3               (HmGui*, uint64 propertyId, Vec3d const* value);
            void               HmGui_SetPropertyDVec4               (HmGui*, uint64 propertyId, Vec4d const* value);
            void               HmGui_SetPropertyColor               (HmGui*, uint64 propertyId, Color const* value);
            void               HmGui_SetPropertyBox3                (HmGui*, uint64 propertyId, Box3f const* value);
            void               HmGui_SetPropertyString              (HmGui*, uint64 propertyId, cstr value);
            void               HmGui_SetPropertyFont                (HmGui*, uint64 propertyId, Font const* value);
            bool               HmGui_GetPropertyBool                (HmGui const*, uint64 propertyId);
            int8               HmGui_GetPropertyI8                  (HmGui const*, uint64 propertyId);
            uint8              HmGui_GetPropertyU8                  (HmGui const*, uint64 propertyId);
            int16              HmGui_GetPropertyI16                 (HmGui const*, uint64 propertyId);
            uint16             HmGui_GetPropertyU16                 (HmGui const*, uint64 propertyId);
            int                HmGui_GetPropertyI32                 (HmGui const*, uint64 propertyId);
            uint32             HmGui_GetPropertyU32                 (HmGui const*, uint64 propertyId);
            int64              HmGui_GetPropertyI64                 (HmGui const*, uint64 propertyId);
            uint64             HmGui_GetPropertyU64                 (HmGui const*, uint64 propertyId);
            float              HmGui_GetPropertyF32                 (HmGui const*, uint64 propertyId);
            double             HmGui_GetPropertyF64                 (HmGui const*, uint64 propertyId);
            Vec2f              HmGui_GetPropertyVec2                (HmGui const*, uint64 propertyId);
            Vec3f const*       HmGui_GetPropertyVec3                (HmGui const*, uint64 propertyId);
            Vec4f const*       HmGui_GetPropertyVec4                (HmGui const*, uint64 propertyId);
            Vec2i              HmGui_GetPropertyIVec2               (HmGui const*, uint64 propertyId);
            Vec3i const*       HmGui_GetPropertyIVec3               (HmGui const*, uint64 propertyId);
            Vec4i const*       HmGui_GetPropertyIVec4               (HmGui const*, uint64 propertyId);
            Vec2u              HmGui_GetPropertyUVec2               (HmGui const*, uint64 propertyId);
            Vec3u const*       HmGui_GetPropertyUVec3               (HmGui const*, uint64 propertyId);
            Vec4u const*       HmGui_GetPropertyUVec4               (HmGui const*, uint64 propertyId);
            Vec2d              HmGui_GetPropertyDVec2               (HmGui const*, uint64 propertyId);
            Vec3d const*       HmGui_GetPropertyDVec3               (HmGui const*, uint64 propertyId);
            Vec4d const*       HmGui_GetPropertyDVec4               (HmGui const*, uint64 propertyId);
            Color const*       HmGui_GetPropertyColor               (HmGui const*, uint64 propertyId);
            Box3f const*       HmGui_GetPropertyBox3                (HmGui const*, uint64 propertyId);
            cstr               HmGui_GetPropertyString              (HmGui const*, uint64 propertyId);
            Font const*        HmGui_GetPropertyFont                (HmGui const*, uint64 propertyId);
            void               HmGui_DumpWidgets                    (HmGui const*);
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
                beginHorizontalContainer       = libphx.HmGui_BeginHorizontalContainer,
                beginVerticalContainer         = libphx.HmGui_BeginVerticalContainer,
                beginStackContainer            = libphx.HmGui_BeginStackContainer,
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
                setChildrenAlignment           = libphx.HmGui_SetChildrenAlignment,
                setChildrenHorizontalAlignment = libphx.HmGui_SetChildrenHorizontalAlignment,
                setChildrenVerticalAlignment   = libphx.HmGui_SetChildrenVerticalAlignment,
                setTheme                       = libphx.HmGui_SetTheme,
                clearTheme                     = libphx.HmGui_ClearTheme,
                getStyleId                     = libphx.HmGui_GetStyleId,
                setStyle                       = libphx.HmGui_SetStyle,
                clearStyle                     = libphx.HmGui_ClearStyle,
                getPropertyType                = function(...)
                    local instance = libphx.HmGui_GetPropertyType(...)
                    return Core.ManagedObject(instance, libphx.HmGuiPropertyType_Free)
                end,
                mapProperty                    = libphx.HmGui_MapProperty,
                mapPropertyGroup               = libphx.HmGui_MapPropertyGroup,
                removeProperty                 = libphx.HmGui_RemoveProperty,
                registerPropertyBool           = libphx.HmGui_RegisterPropertyBool,
                registerPropertyI8             = libphx.HmGui_RegisterPropertyI8,
                registerPropertyU8             = libphx.HmGui_RegisterPropertyU8,
                registerPropertyI16            = libphx.HmGui_RegisterPropertyI16,
                registerPropertyU16            = libphx.HmGui_RegisterPropertyU16,
                registerPropertyI32            = libphx.HmGui_RegisterPropertyI32,
                registerPropertyU32            = libphx.HmGui_RegisterPropertyU32,
                registerPropertyI64            = libphx.HmGui_RegisterPropertyI64,
                registerPropertyU64            = libphx.HmGui_RegisterPropertyU64,
                registerPropertyF32            = libphx.HmGui_RegisterPropertyF32,
                registerPropertyF64            = libphx.HmGui_RegisterPropertyF64,
                registerPropertyVec2           = libphx.HmGui_RegisterPropertyVec2,
                registerPropertyVec3           = libphx.HmGui_RegisterPropertyVec3,
                registerPropertyVec4           = libphx.HmGui_RegisterPropertyVec4,
                registerPropertyIVec2          = libphx.HmGui_RegisterPropertyIVec2,
                registerPropertyIVec3          = libphx.HmGui_RegisterPropertyIVec3,
                registerPropertyIVec4          = libphx.HmGui_RegisterPropertyIVec4,
                registerPropertyUVec2          = libphx.HmGui_RegisterPropertyUVec2,
                registerPropertyUVec3          = libphx.HmGui_RegisterPropertyUVec3,
                registerPropertyUVec4          = libphx.HmGui_RegisterPropertyUVec4,
                registerPropertyDVec2          = libphx.HmGui_RegisterPropertyDVec2,
                registerPropertyDVec3          = libphx.HmGui_RegisterPropertyDVec3,
                registerPropertyDVec4          = libphx.HmGui_RegisterPropertyDVec4,
                registerPropertyColor          = libphx.HmGui_RegisterPropertyColor,
                registerPropertyBox3           = libphx.HmGui_RegisterPropertyBox3,
                registerPropertyString         = libphx.HmGui_RegisterPropertyString,
                registerPropertyFont           = libphx.HmGui_RegisterPropertyFont,
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
                setPropertyColor               = libphx.HmGui_SetPropertyColor,
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
                getPropertyColor               = libphx.HmGui_GetPropertyColor,
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
