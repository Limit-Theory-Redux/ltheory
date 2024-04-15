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
            void               HmGui_BeginScroll                    (HmGui*, float maxSize);
            void               HmGui_EndScroll                      (HmGui*, Input const* input);
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
            bool               HmGui_IsMouseOver                    (HmGui const*, FocusType ty);
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
                -- Begin GUI declaration. Region is limited by [0, 0] - [sx, sy] rectangle.
                ---@param sx float
                ---@param sy float
                ---@param input Input const*
                beginGui                       = libphx.HmGui_BeginGui,
                -- Finish GUI declaration, calculate hierarchy widgets sizes and layout.
                ---@param input Input const*
                endGui                         = libphx.HmGui_EndGui,
                -- Pass information about widgets to the renderer and draw them.
                draw                           = libphx.HmGui_Draw,
                beginHorizontalContainer       = libphx.HmGui_BeginHorizontalContainer,
                beginVerticalContainer         = libphx.HmGui_BeginVerticalContainer,
                beginStackContainer            = libphx.HmGui_BeginStackContainer,
                endContainer                   = libphx.HmGui_EndContainer,
                ---@param _max_size float
                beginScroll                    = libphx.HmGui_BeginScroll,
                ---@param input Input const*
                endScroll                      = libphx.HmGui_EndScroll,
                -- Begins window element.
                ---@param _title cstr
                ---@param input Input const*
                beginWindow                    = libphx.HmGui_BeginWindow,
                -- Ends window element.
                endWindow                      = libphx.HmGui_EndWindow,
                -- Invisible element that stretches in all directions.
                -- Use for pushing neighbor elements to the sides. See [`Self::checkbox`] for example.
                spacer                         = libphx.HmGui_Spacer,
                ---@param label cstr
                ---@return bool
                button                         = libphx.HmGui_Button,
                ---@param label cstr
                ---@param value bool
                ---@return bool
                checkbox                       = libphx.HmGui_Checkbox,
                ---@param _lower float
                ---@param _upper float
                ---@param _value float
                ---@return float
                slider                         = libphx.HmGui_Slider,
                ---@param height float
                ---@param color Color const*
                horizontalDivider              = libphx.HmGui_HorizontalDivider,
                ---@param width float
                ---@param color Color const*
                verticalDivider                = libphx.HmGui_VerticalDivider,
                ---@param image Tex2D*
                image                          = libphx.HmGui_Image,
                ---@param color Color const*
                rect                           = libphx.HmGui_Rect,
                ---@param text cstr
                text                           = libphx.HmGui_Text,
                ---@param text cstr
                ---@param color Color const*
                textColored                    = libphx.HmGui_TextColored,
                ---@param font Font const*
                ---@param text cstr
                ---@param color Color const*
                textEx                         = libphx.HmGui_TextEx,
                ---@param width float
                setMinWidth                    = libphx.HmGui_SetMinWidth,
                ---@param height float
                setMinHeight                   = libphx.HmGui_SetMinHeight,
                ---@param width float
                ---@param height float
                setMinSize                     = libphx.HmGui_SetMinSize,
                ---@param width float
                setFixedWidth                  = libphx.HmGui_SetFixedWidth,
                ---@param height float
                setFixedHeight                 = libphx.HmGui_SetFixedHeight,
                ---@param width float
                ---@param height float
                setFixedSize                   = libphx.HmGui_SetFixedSize,
                ---@param width float
                setPercentWidth                = libphx.HmGui_SetPercentWidth,
                ---@param height float
                setPercentHeight               = libphx.HmGui_SetPercentHeight,
                ---@param width float
                ---@param height float
                setPercentSize                 = libphx.HmGui_SetPercentSize,
                ---@param px float
                ---@param py float
                setMargin                      = libphx.HmGui_SetMargin,
                ---@param left float
                ---@param top float
                ---@param right float
                ---@param bottom float
                setMarginEx                    = libphx.HmGui_SetMarginEx,
                ---@param margin float
                setMarginLeft                  = libphx.HmGui_SetMarginLeft,
                ---@param margin float
                setMarginTop                   = libphx.HmGui_SetMarginTop,
                ---@param margin float
                setMarginRight                 = libphx.HmGui_SetMarginRight,
                ---@param margin float
                setMarginBottom                = libphx.HmGui_SetMarginBottom,
                ---@param width float
                setBorderWidth                 = libphx.HmGui_SetBorderWidth,
                ---@param color Color const*
                setBorderColor                 = libphx.HmGui_SetBorderColor,
                ---@param color Color const*
                setBorderColorV4               = libphx.HmGui_SetBorderColorV4,
                ---@param width float
                ---@param color Color const*
                setBorder                      = libphx.HmGui_SetBorder,
                ---@param width float
                ---@param color Color const*
                setBorderV4                    = libphx.HmGui_SetBorderV4,
                ---@param color Color const*
                setBgColor                     = libphx.HmGui_SetBgColor,
                ---@param color Color const*
                setBgColorV4                   = libphx.HmGui_SetBgColorV4,
                ---@param h AlignHorizontal
                ---@param v AlignVertical
                setAlignment                   = libphx.HmGui_SetAlignment,
                ---@param align AlignHorizontal
                setHorizontalAlignment         = libphx.HmGui_SetHorizontalAlignment,
                ---@param align AlignVertical
                setVerticalAlignment           = libphx.HmGui_SetVerticalAlignment,
                ---@param px float
                ---@param py float
                setPadding                     = libphx.HmGui_SetPadding,
                ---@param left float
                ---@param top float
                ---@param right float
                ---@param bottom float
                setPaddingEx                   = libphx.HmGui_SetPaddingEx,
                ---@param padding float
                setPaddingLeft                 = libphx.HmGui_SetPaddingLeft,
                ---@param padding float
                setPaddingTop                  = libphx.HmGui_SetPaddingTop,
                ---@param padding float
                setPaddingRight                = libphx.HmGui_SetPaddingRight,
                ---@param padding float
                setPaddingBottom               = libphx.HmGui_SetPaddingBottom,
                ---@param spacing float
                setSpacing                     = libphx.HmGui_SetSpacing,
                -- Makes current container `focusable` and returns if it's currently in focus.
                ---@param ty FocusType
                ---@return bool
                isMouseOver                    = libphx.HmGui_IsMouseOver,
                ---@param h AlignHorizontal
                ---@param v AlignVertical
                setChildrenAlignment           = libphx.HmGui_SetChildrenAlignment,
                ---@param align AlignHorizontal
                setChildrenHorizontalAlignment = libphx.HmGui_SetChildrenHorizontalAlignment,
                ---@param align AlignVertical
                setChildrenVerticalAlignment   = libphx.HmGui_SetChildrenVerticalAlignment,
                -- Set a theme by merging it into the default properties.
                ---@param name cstr
                setTheme                       = libphx.HmGui_SetTheme,
                -- Restore default properties.
                clearTheme                     = libphx.HmGui_ClearTheme,
                -- Get style id by its name.
                ---@param name cstr
                ---@return uint64
                getStyleId                     = libphx.HmGui_GetStyleId,
                -- Set a style for the following element.
                ---@param id uint64
                setStyle                       = libphx.HmGui_SetStyle,
                -- Remove element style.
                clearStyle                     = libphx.HmGui_ClearStyle,
                -- Get property type by its id.
                ---@param id uint64
                ---@return HmGuiPropertyType*
                getPropertyType                = function(...)
                    local instance = libphx.HmGui_GetPropertyType(...)
                    return Core.ManagedObject(instance, libphx.HmGuiPropertyType_Free)
                end,
                -- Write property value into the mapped properties in the active element style.
                ---@param property_id uint64
                mapProperty                    = libphx.HmGui_MapProperty,
                -- Remove property by id from the active element style.
                ---@param property_id uint64
                removeProperty                 = libphx.HmGui_RemoveProperty,
                ---@param name cstr
                ---@param value bool
                ---@param map_id cstr
                ---@return uint64
                registerPropertyBool           = libphx.HmGui_RegisterPropertyBool,
                ---@param name cstr
                ---@param value int8
                ---@param map_id cstr
                ---@return uint64
                registerPropertyI8             = libphx.HmGui_RegisterPropertyI8,
                ---@param name cstr
                ---@param value uint8
                ---@param map_id cstr
                ---@return uint64
                registerPropertyU8             = libphx.HmGui_RegisterPropertyU8,
                ---@param name cstr
                ---@param value int16
                ---@param map_id cstr
                ---@return uint64
                registerPropertyI16            = libphx.HmGui_RegisterPropertyI16,
                ---@param name cstr
                ---@param value uint16
                ---@param map_id cstr
                ---@return uint64
                registerPropertyU16            = libphx.HmGui_RegisterPropertyU16,
                ---@param name cstr
                ---@param value int
                ---@param map_id cstr
                ---@return uint64
                registerPropertyI32            = libphx.HmGui_RegisterPropertyI32,
                ---@param name cstr
                ---@param value uint32
                ---@param map_id cstr
                ---@return uint64
                registerPropertyU32            = libphx.HmGui_RegisterPropertyU32,
                ---@param name cstr
                ---@param value int64
                ---@param map_id cstr
                ---@return uint64
                registerPropertyI64            = libphx.HmGui_RegisterPropertyI64,
                ---@param name cstr
                ---@param value uint64
                ---@param map_id cstr
                ---@return uint64
                registerPropertyU64            = libphx.HmGui_RegisterPropertyU64,
                ---@param name cstr
                ---@param value float
                ---@param map_id cstr
                ---@return uint64
                registerPropertyF32            = libphx.HmGui_RegisterPropertyF32,
                ---@param name cstr
                ---@param value double
                ---@param map_id cstr
                ---@return uint64
                registerPropertyF64            = libphx.HmGui_RegisterPropertyF64,
                ---@param name cstr
                ---@param value Vec2f
                ---@param map_id cstr
                ---@return uint64
                registerPropertyVec2           = libphx.HmGui_RegisterPropertyVec2,
                ---@param name cstr
                ---@param value Vec3f const*
                ---@param map_id cstr
                ---@return uint64
                registerPropertyVec3           = libphx.HmGui_RegisterPropertyVec3,
                ---@param name cstr
                ---@param value Vec4f const*
                ---@param map_id cstr
                ---@return uint64
                registerPropertyVec4           = libphx.HmGui_RegisterPropertyVec4,
                ---@param name cstr
                ---@param value Vec2i
                ---@param map_id cstr
                ---@return uint64
                registerPropertyIVec2          = libphx.HmGui_RegisterPropertyIVec2,
                ---@param name cstr
                ---@param value Vec3i const*
                ---@param map_id cstr
                ---@return uint64
                registerPropertyIVec3          = libphx.HmGui_RegisterPropertyIVec3,
                ---@param name cstr
                ---@param value Vec4i const*
                ---@param map_id cstr
                ---@return uint64
                registerPropertyIVec4          = libphx.HmGui_RegisterPropertyIVec4,
                ---@param name cstr
                ---@param value Vec2u
                ---@param map_id cstr
                ---@return uint64
                registerPropertyUVec2          = libphx.HmGui_RegisterPropertyUVec2,
                ---@param name cstr
                ---@param value Vec3u const*
                ---@param map_id cstr
                ---@return uint64
                registerPropertyUVec3          = libphx.HmGui_RegisterPropertyUVec3,
                ---@param name cstr
                ---@param value Vec4u const*
                ---@param map_id cstr
                ---@return uint64
                registerPropertyUVec4          = libphx.HmGui_RegisterPropertyUVec4,
                ---@param name cstr
                ---@param value Vec2d
                ---@param map_id cstr
                ---@return uint64
                registerPropertyDVec2          = libphx.HmGui_RegisterPropertyDVec2,
                ---@param name cstr
                ---@param value Vec3d const*
                ---@param map_id cstr
                ---@return uint64
                registerPropertyDVec3          = libphx.HmGui_RegisterPropertyDVec3,
                ---@param name cstr
                ---@param value Vec4d const*
                ---@param map_id cstr
                ---@return uint64
                registerPropertyDVec4          = libphx.HmGui_RegisterPropertyDVec4,
                ---@param name cstr
                ---@param value Color const*
                ---@param map_id cstr
                ---@return uint64
                registerPropertyColor          = libphx.HmGui_RegisterPropertyColor,
                ---@param name cstr
                ---@param value Box3f const*
                ---@param map_id cstr
                ---@return uint64
                registerPropertyBox3           = libphx.HmGui_RegisterPropertyBox3,
                ---@param name cstr
                ---@param value cstr
                ---@param map_id cstr
                ---@return uint64
                registerPropertyString         = libphx.HmGui_RegisterPropertyString,
                ---@param name cstr
                ---@param value Font const*
                ---@param map_id cstr
                ---@return uint64
                registerPropertyFont           = libphx.HmGui_RegisterPropertyFont,
                ---@param property_id uint64
                ---@param value bool
                setPropertyBool                = libphx.HmGui_SetPropertyBool,
                ---@param property_id uint64
                ---@param value int8
                setPropertyI8                  = libphx.HmGui_SetPropertyI8,
                ---@param property_id uint64
                ---@param value uint8
                setPropertyU8                  = libphx.HmGui_SetPropertyU8,
                ---@param property_id uint64
                ---@param value int16
                setPropertyI16                 = libphx.HmGui_SetPropertyI16,
                ---@param property_id uint64
                ---@param value uint16
                setPropertyU16                 = libphx.HmGui_SetPropertyU16,
                ---@param property_id uint64
                ---@param value int
                setPropertyI32                 = libphx.HmGui_SetPropertyI32,
                ---@param property_id uint64
                ---@param value uint32
                setPropertyU32                 = libphx.HmGui_SetPropertyU32,
                ---@param property_id uint64
                ---@param value int64
                setPropertyI64                 = libphx.HmGui_SetPropertyI64,
                ---@param property_id uint64
                ---@param value uint64
                setPropertyU64                 = libphx.HmGui_SetPropertyU64,
                ---@param property_id uint64
                ---@param value float
                setPropertyF32                 = libphx.HmGui_SetPropertyF32,
                ---@param property_id uint64
                ---@param value double
                setPropertyF64                 = libphx.HmGui_SetPropertyF64,
                ---@param property_id uint64
                ---@param value Vec2f
                setPropertyVec2                = libphx.HmGui_SetPropertyVec2,
                ---@param property_id uint64
                ---@param value Vec3f const*
                setPropertyVec3                = libphx.HmGui_SetPropertyVec3,
                ---@param property_id uint64
                ---@param value Vec4f const*
                setPropertyVec4                = libphx.HmGui_SetPropertyVec4,
                ---@param property_id uint64
                ---@param value Vec2i
                setPropertyIVec2               = libphx.HmGui_SetPropertyIVec2,
                ---@param property_id uint64
                ---@param value Vec3i const*
                setPropertyIVec3               = libphx.HmGui_SetPropertyIVec3,
                ---@param property_id uint64
                ---@param value Vec4i const*
                setPropertyIVec4               = libphx.HmGui_SetPropertyIVec4,
                ---@param property_id uint64
                ---@param value Vec2u
                setPropertyUVec2               = libphx.HmGui_SetPropertyUVec2,
                ---@param property_id uint64
                ---@param value Vec3u const*
                setPropertyUVec3               = libphx.HmGui_SetPropertyUVec3,
                ---@param property_id uint64
                ---@param value Vec4u const*
                setPropertyUVec4               = libphx.HmGui_SetPropertyUVec4,
                ---@param property_id uint64
                ---@param value Vec2d
                setPropertyDVec2               = libphx.HmGui_SetPropertyDVec2,
                ---@param property_id uint64
                ---@param value Vec3d const*
                setPropertyDVec3               = libphx.HmGui_SetPropertyDVec3,
                ---@param property_id uint64
                ---@param value Vec4d const*
                setPropertyDVec4               = libphx.HmGui_SetPropertyDVec4,
                ---@param property_id uint64
                ---@param value Color const*
                setPropertyColor               = libphx.HmGui_SetPropertyColor,
                ---@param property_id uint64
                ---@param value Box3f const*
                setPropertyBox3                = libphx.HmGui_SetPropertyBox3,
                ---@param property_id uint64
                ---@param value cstr
                setPropertyString              = libphx.HmGui_SetPropertyString,
                ---@param property_id uint64
                ---@param value Font const*
                setPropertyFont                = libphx.HmGui_SetPropertyFont,
                ---@param property_id uint64
                ---@return bool
                getPropertyBool                = libphx.HmGui_GetPropertyBool,
                ---@param property_id uint64
                ---@return int8
                getPropertyI8                  = libphx.HmGui_GetPropertyI8,
                ---@param property_id uint64
                ---@return uint8
                getPropertyU8                  = libphx.HmGui_GetPropertyU8,
                ---@param property_id uint64
                ---@return int16
                getPropertyI16                 = libphx.HmGui_GetPropertyI16,
                ---@param property_id uint64
                ---@return uint16
                getPropertyU16                 = libphx.HmGui_GetPropertyU16,
                ---@param property_id uint64
                ---@return int
                getPropertyI32                 = libphx.HmGui_GetPropertyI32,
                ---@param property_id uint64
                ---@return uint32
                getPropertyU32                 = libphx.HmGui_GetPropertyU32,
                ---@param property_id uint64
                ---@return int64
                getPropertyI64                 = libphx.HmGui_GetPropertyI64,
                ---@param property_id uint64
                ---@return uint64
                getPropertyU64                 = libphx.HmGui_GetPropertyU64,
                ---@param property_id uint64
                ---@return float
                getPropertyF32                 = libphx.HmGui_GetPropertyF32,
                ---@param property_id uint64
                ---@return double
                getPropertyF64                 = libphx.HmGui_GetPropertyF64,
                ---@param property_id uint64
                ---@return Vec2f
                getPropertyVec2                = libphx.HmGui_GetPropertyVec2,
                ---@param property_id uint64
                ---@return Vec3f const*
                getPropertyVec3                = libphx.HmGui_GetPropertyVec3,
                ---@param property_id uint64
                ---@return Vec4f const*
                getPropertyVec4                = libphx.HmGui_GetPropertyVec4,
                ---@param property_id uint64
                ---@return Vec2i
                getPropertyIVec2               = libphx.HmGui_GetPropertyIVec2,
                ---@param property_id uint64
                ---@return Vec3i const*
                getPropertyIVec3               = libphx.HmGui_GetPropertyIVec3,
                ---@param property_id uint64
                ---@return Vec4i const*
                getPropertyIVec4               = libphx.HmGui_GetPropertyIVec4,
                ---@param property_id uint64
                ---@return Vec2u
                getPropertyUVec2               = libphx.HmGui_GetPropertyUVec2,
                ---@param property_id uint64
                ---@return Vec3u const*
                getPropertyUVec3               = libphx.HmGui_GetPropertyUVec3,
                ---@param property_id uint64
                ---@return Vec4u const*
                getPropertyUVec4               = libphx.HmGui_GetPropertyUVec4,
                ---@param property_id uint64
                ---@return Vec2d
                getPropertyDVec2               = libphx.HmGui_GetPropertyDVec2,
                ---@param property_id uint64
                ---@return Vec3d const*
                getPropertyDVec3               = libphx.HmGui_GetPropertyDVec3,
                ---@param property_id uint64
                ---@return Vec4d const*
                getPropertyDVec4               = libphx.HmGui_GetPropertyDVec4,
                ---@param property_id uint64
                ---@return Color const*
                getPropertyColor               = libphx.HmGui_GetPropertyColor,
                ---@param property_id uint64
                ---@return Box3f const*
                getPropertyBox3                = libphx.HmGui_GetPropertyBox3,
                ---@param property_id uint64
                ---@return cstr
                getPropertyString              = libphx.HmGui_GetPropertyString,
                ---@param property_id uint64
                ---@return Font const*
                getPropertyFont                = libphx.HmGui_GetPropertyFont,
                -- Prints widgets hierarchy to the console. For testing.
                dumpWidgets                    = libphx.HmGui_DumpWidgets,
            },
        }

        if onDef_HmGui_t then onDef_HmGui_t(t, mt) end
        HmGui_t = ffi.metatype(t, mt)
    end

    return HmGui
end

return Loader
