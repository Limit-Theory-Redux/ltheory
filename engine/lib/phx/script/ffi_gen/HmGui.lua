-- HmGui -----------------------------------------------------------------------

---@class HmGui
---@field BeginGui fun(self, sx: number, sy: number, input: Input)
---@field EndGui fun(self, input: Input)
---@field Draw fun(self)
---@field BeginHorizontalContainer fun(self)
---@field BeginVerticalContainer fun(self)
---@field BeginStackContainer fun(self)
---@field EndContainer fun(self)
---@field BeginScroll fun(self, _max_size: number)
---@field EndScroll fun(self, input: Input)
---@field BeginWindow fun(self, _title: string, input: Input)
---@field EndWindow fun(self)
---@field Spacer fun(self)
---@field Button fun(self, label: string): boolean
---@field Checkbox fun(self, label: string, value: boolean): boolean
---@field Slider fun(self, _lower: number, _upper: number, _value: number): number
---@field HorizontalDivider fun(self, height: number, color: Color)
---@field VerticalDivider fun(self, width: number, color: Color)
---@field Image fun(self, image: Tex2D)
---@field Rect fun(self, color: Color)
---@field Text fun(self, text: string)
---@field TextColored fun(self, text: string, color: Color)
---@field TextEx fun(self, font: Font, text: string, color: Color)
---@field SetMinWidth fun(self, width: number)
---@field SetMinHeight fun(self, height: number)
---@field SetMinSize fun(self, width: number, height: number)
---@field SetFixedWidth fun(self, width: number)
---@field SetFixedHeight fun(self, height: number)
---@field SetFixedSize fun(self, width: number, height: number)
---@field SetPercentWidth fun(self, width: number)
---@field SetPercentHeight fun(self, height: number)
---@field SetPercentSize fun(self, width: number, height: number)
---@field SetMargin fun(self, px: number, py: number)
---@field SetMarginEx fun(self, left: number, top: number, right: number, bottom: number)
---@field SetMarginLeft fun(self, margin: number)
---@field SetMarginTop fun(self, margin: number)
---@field SetMarginRight fun(self, margin: number)
---@field SetMarginBottom fun(self, margin: number)
---@field SetBorderWidth fun(self, width: number)
---@field SetBorderColor fun(self, color: Color)
---@field SetBorderColorV4 fun(self, color: Color)
---@field SetBorder fun(self, width: number, color: Color)
---@field SetBorderV4 fun(self, width: number, color: Color)
---@field SetBgColor fun(self, color: Color)
---@field SetBgColorV4 fun(self, color: Color)
---@field SetAlignment fun(self, h: AlignHorizontal, v: AlignVertical)
---@field SetHorizontalAlignment fun(self, align: AlignHorizontal)
---@field SetVerticalAlignment fun(self, align: AlignVertical)
---@field SetPadding fun(self, px: number, py: number)
---@field SetPaddingEx fun(self, left: number, top: number, right: number, bottom: number)
---@field SetPaddingLeft fun(self, padding: number)
---@field SetPaddingTop fun(self, padding: number)
---@field SetPaddingRight fun(self, padding: number)
---@field SetPaddingBottom fun(self, padding: number)
---@field SetSpacing fun(self, spacing: number)
---@field IsMouseOver fun(self, ty: FocusType): boolean
---@field SetChildrenAlignment fun(self, h: AlignHorizontal, v: AlignVertical)
---@field SetChildrenHorizontalAlignment fun(self, align: AlignHorizontal)
---@field SetChildrenVerticalAlignment fun(self, align: AlignVertical)
---@field SetTheme fun(self, name: string)
---@field ClearTheme fun(self)
---@field GetStyleId fun(self, name: string): integer
---@field SetStyle fun(self, id: integer)
---@field ClearStyle fun(self)
---@field GetPropertyType fun(self, id: integer): HmGuiPropertyType
---@field MapProperty fun(self, property_id: integer)
---@field RemoveProperty fun(self, property_id: integer)
---@field RegisterPropertyBool fun(self, name: string, value: boolean, map_id: string): integer
---@field RegisterPropertyI8 fun(self, name: string, value: integer, map_id: string): integer
---@field RegisterPropertyU8 fun(self, name: string, value: integer, map_id: string): integer
---@field RegisterPropertyI16 fun(self, name: string, value: integer, map_id: string): integer
---@field RegisterPropertyU16 fun(self, name: string, value: integer, map_id: string): integer
---@field RegisterPropertyI32 fun(self, name: string, value: integer, map_id: string): integer
---@field RegisterPropertyU32 fun(self, name: string, value: integer, map_id: string): integer
---@field RegisterPropertyI64 fun(self, name: string, value: integer, map_id: string): integer
---@field RegisterPropertyU64 fun(self, name: string, value: integer, map_id: string): integer
---@field RegisterPropertyF32 fun(self, name: string, value: number, map_id: string): integer
---@field RegisterPropertyF64 fun(self, name: string, value: number, map_id: string): integer
---@field RegisterPropertyVec2 fun(self, name: string, value: Vec2, map_id: string): integer
---@field RegisterPropertyVec3 fun(self, name: string, value: Vec3, map_id: string): integer
---@field RegisterPropertyVec4 fun(self, name: string, value: Vec4, map_id: string): integer
---@field RegisterPropertyIVec2 fun(self, name: string, value: IVec2, map_id: string): integer
---@field RegisterPropertyIVec3 fun(self, name: string, value: IVec3, map_id: string): integer
---@field RegisterPropertyIVec4 fun(self, name: string, value: IVec4, map_id: string): integer
---@field RegisterPropertyUVec2 fun(self, name: string, value: UVec2, map_id: string): integer
---@field RegisterPropertyUVec3 fun(self, name: string, value: UVec3, map_id: string): integer
---@field RegisterPropertyUVec4 fun(self, name: string, value: UVec4, map_id: string): integer
---@field RegisterPropertyDVec2 fun(self, name: string, value: DVec2, map_id: string): integer
---@field RegisterPropertyDVec3 fun(self, name: string, value: DVec3, map_id: string): integer
---@field RegisterPropertyDVec4 fun(self, name: string, value: DVec4, map_id: string): integer
---@field RegisterPropertyColor fun(self, name: string, value: Color, map_id: string): integer
---@field RegisterPropertyBox3 fun(self, name: string, value: Box3, map_id: string): integer
---@field RegisterPropertyString fun(self, name: string, value: string, map_id: string): integer
---@field RegisterPropertyFont fun(self, name: string, value: Font, map_id: string): integer
---@field SetPropertyBool fun(self, property_id: integer, value: boolean)
---@field SetPropertyI8 fun(self, property_id: integer, value: integer)
---@field SetPropertyU8 fun(self, property_id: integer, value: integer)
---@field SetPropertyI16 fun(self, property_id: integer, value: integer)
---@field SetPropertyU16 fun(self, property_id: integer, value: integer)
---@field SetPropertyI32 fun(self, property_id: integer, value: integer)
---@field SetPropertyU32 fun(self, property_id: integer, value: integer)
---@field SetPropertyI64 fun(self, property_id: integer, value: integer)
---@field SetPropertyU64 fun(self, property_id: integer, value: integer)
---@field SetPropertyF32 fun(self, property_id: integer, value: number)
---@field SetPropertyF64 fun(self, property_id: integer, value: number)
---@field SetPropertyVec2 fun(self, property_id: integer, value: Vec2)
---@field SetPropertyVec3 fun(self, property_id: integer, value: Vec3)
---@field SetPropertyVec4 fun(self, property_id: integer, value: Vec4)
---@field SetPropertyIVec2 fun(self, property_id: integer, value: IVec2)
---@field SetPropertyIVec3 fun(self, property_id: integer, value: IVec3)
---@field SetPropertyIVec4 fun(self, property_id: integer, value: IVec4)
---@field SetPropertyUVec2 fun(self, property_id: integer, value: UVec2)
---@field SetPropertyUVec3 fun(self, property_id: integer, value: UVec3)
---@field SetPropertyUVec4 fun(self, property_id: integer, value: UVec4)
---@field SetPropertyDVec2 fun(self, property_id: integer, value: DVec2)
---@field SetPropertyDVec3 fun(self, property_id: integer, value: DVec3)
---@field SetPropertyDVec4 fun(self, property_id: integer, value: DVec4)
---@field SetPropertyColor fun(self, property_id: integer, value: Color)
---@field SetPropertyBox3 fun(self, property_id: integer, value: Box3)
---@field SetPropertyString fun(self, property_id: integer, value: string)
---@field SetPropertyFont fun(self, property_id: integer, value: Font)
---@field GetPropertyBool fun(self, property_id: integer): boolean
---@field GetPropertyI8 fun(self, property_id: integer): integer
---@field GetPropertyU8 fun(self, property_id: integer): integer
---@field GetPropertyI16 fun(self, property_id: integer): integer
---@field GetPropertyU16 fun(self, property_id: integer): integer
---@field GetPropertyI32 fun(self, property_id: integer): integer
---@field GetPropertyU32 fun(self, property_id: integer): integer
---@field GetPropertyI64 fun(self, property_id: integer): integer
---@field GetPropertyU64 fun(self, property_id: integer): integer
---@field GetPropertyF32 fun(self, property_id: integer): number
---@field GetPropertyF64 fun(self, property_id: integer): number
---@field GetPropertyVec2 fun(self, property_id: integer): Vec2
---@field GetPropertyVec3 fun(self, property_id: integer): Vec3
---@field GetPropertyVec4 fun(self, property_id: integer): Vec4
---@field GetPropertyIVec2 fun(self, property_id: integer): IVec2
---@field GetPropertyIVec3 fun(self, property_id: integer): IVec3
---@field GetPropertyIVec4 fun(self, property_id: integer): IVec4
---@field GetPropertyUVec2 fun(self, property_id: integer): UVec2
---@field GetPropertyUVec3 fun(self, property_id: integer): UVec3
---@field GetPropertyUVec4 fun(self, property_id: integer): UVec4
---@field GetPropertyDVec2 fun(self, property_id: integer): DVec2
---@field GetPropertyDVec3 fun(self, property_id: integer): DVec3
---@field GetPropertyDVec4 fun(self, property_id: integer): DVec4
---@field GetPropertyColor fun(self, property_id: integer): Color
---@field GetPropertyBox3 fun(self, property_id: integer): Box3
---@field GetPropertyString fun(self, property_id: integer): string
---@field GetPropertyFont fun(self, property_id: integer): Font
---@field DumpWidgets fun(self)

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
                ---@param sx number
                ---@param sy number
                ---@param input Input
                beginGui                       = libphx.HmGui_BeginGui,
                -- Finish GUI declaration, calculate hierarchy widgets sizes and layout.
                ---@param input Input
                endGui                         = libphx.HmGui_EndGui,
                -- Pass information about widgets to the renderer and draw them.
                draw                           = libphx.HmGui_Draw,
                beginHorizontalContainer       = libphx.HmGui_BeginHorizontalContainer,
                beginVerticalContainer         = libphx.HmGui_BeginVerticalContainer,
                beginStackContainer            = libphx.HmGui_BeginStackContainer,
                endContainer                   = libphx.HmGui_EndContainer,
                ---@param _max_size number
                beginScroll                    = libphx.HmGui_BeginScroll,
                ---@param input Input
                endScroll                      = libphx.HmGui_EndScroll,
                -- Begins window element.
                ---@param _title string
                ---@param input Input
                beginWindow                    = libphx.HmGui_BeginWindow,
                -- Ends window element.
                endWindow                      = libphx.HmGui_EndWindow,
                -- Invisible element that stretches in all directions.
                -- Use for pushing neighbor elements to the sides. See [`Self::checkbox`] for example.
                spacer                         = libphx.HmGui_Spacer,
                ---@param label string
                ---@return boolean
                button                         = libphx.HmGui_Button,
                ---@param label string
                ---@param value boolean
                ---@return boolean
                checkbox                       = libphx.HmGui_Checkbox,
                ---@param _lower number
                ---@param _upper number
                ---@param _value number
                ---@return number
                slider                         = libphx.HmGui_Slider,
                ---@param height number
                ---@param color Color
                horizontalDivider              = libphx.HmGui_HorizontalDivider,
                ---@param width number
                ---@param color Color
                verticalDivider                = libphx.HmGui_VerticalDivider,
                ---@param image Tex2D
                image                          = libphx.HmGui_Image,
                ---@param color Color
                rect                           = libphx.HmGui_Rect,
                ---@param text string
                text                           = libphx.HmGui_Text,
                ---@param text string
                ---@param color Color
                textColored                    = libphx.HmGui_TextColored,
                ---@param font Font
                ---@param text string
                ---@param color Color
                textEx                         = libphx.HmGui_TextEx,
                ---@param width number
                setMinWidth                    = libphx.HmGui_SetMinWidth,
                ---@param height number
                setMinHeight                   = libphx.HmGui_SetMinHeight,
                ---@param width number
                ---@param height number
                setMinSize                     = libphx.HmGui_SetMinSize,
                ---@param width number
                setFixedWidth                  = libphx.HmGui_SetFixedWidth,
                ---@param height number
                setFixedHeight                 = libphx.HmGui_SetFixedHeight,
                ---@param width number
                ---@param height number
                setFixedSize                   = libphx.HmGui_SetFixedSize,
                ---@param width number
                setPercentWidth                = libphx.HmGui_SetPercentWidth,
                ---@param height number
                setPercentHeight               = libphx.HmGui_SetPercentHeight,
                ---@param width number
                ---@param height number
                setPercentSize                 = libphx.HmGui_SetPercentSize,
                ---@param px number
                ---@param py number
                setMargin                      = libphx.HmGui_SetMargin,
                ---@param left number
                ---@param top number
                ---@param right number
                ---@param bottom number
                setMarginEx                    = libphx.HmGui_SetMarginEx,
                ---@param margin number
                setMarginLeft                  = libphx.HmGui_SetMarginLeft,
                ---@param margin number
                setMarginTop                   = libphx.HmGui_SetMarginTop,
                ---@param margin number
                setMarginRight                 = libphx.HmGui_SetMarginRight,
                ---@param margin number
                setMarginBottom                = libphx.HmGui_SetMarginBottom,
                ---@param width number
                setBorderWidth                 = libphx.HmGui_SetBorderWidth,
                ---@param color Color
                setBorderColor                 = libphx.HmGui_SetBorderColor,
                ---@param color Color
                setBorderColorV4               = libphx.HmGui_SetBorderColorV4,
                ---@param width number
                ---@param color Color
                setBorder                      = libphx.HmGui_SetBorder,
                ---@param width number
                ---@param color Color
                setBorderV4                    = libphx.HmGui_SetBorderV4,
                ---@param color Color
                setBgColor                     = libphx.HmGui_SetBgColor,
                ---@param color Color
                setBgColorV4                   = libphx.HmGui_SetBgColorV4,
                ---@param h AlignHorizontal
                ---@param v AlignVertical
                setAlignment                   = libphx.HmGui_SetAlignment,
                ---@param align AlignHorizontal
                setHorizontalAlignment         = libphx.HmGui_SetHorizontalAlignment,
                ---@param align AlignVertical
                setVerticalAlignment           = libphx.HmGui_SetVerticalAlignment,
                ---@param px number
                ---@param py number
                setPadding                     = libphx.HmGui_SetPadding,
                ---@param left number
                ---@param top number
                ---@param right number
                ---@param bottom number
                setPaddingEx                   = libphx.HmGui_SetPaddingEx,
                ---@param padding number
                setPaddingLeft                 = libphx.HmGui_SetPaddingLeft,
                ---@param padding number
                setPaddingTop                  = libphx.HmGui_SetPaddingTop,
                ---@param padding number
                setPaddingRight                = libphx.HmGui_SetPaddingRight,
                ---@param padding number
                setPaddingBottom               = libphx.HmGui_SetPaddingBottom,
                ---@param spacing number
                setSpacing                     = libphx.HmGui_SetSpacing,
                -- Makes current container `focusable` and returns if it's currently in focus.
                ---@param ty FocusType
                ---@return boolean
                isMouseOver                    = libphx.HmGui_IsMouseOver,
                ---@param h AlignHorizontal
                ---@param v AlignVertical
                setChildrenAlignment           = libphx.HmGui_SetChildrenAlignment,
                ---@param align AlignHorizontal
                setChildrenHorizontalAlignment = libphx.HmGui_SetChildrenHorizontalAlignment,
                ---@param align AlignVertical
                setChildrenVerticalAlignment   = libphx.HmGui_SetChildrenVerticalAlignment,
                -- Set a theme by merging it into the default properties.
                ---@param name string
                setTheme                       = libphx.HmGui_SetTheme,
                -- Restore default properties.
                clearTheme                     = libphx.HmGui_ClearTheme,
                -- Get style id by its name.
                ---@param name string
                ---@return integer
                getStyleId                     = libphx.HmGui_GetStyleId,
                -- Set a style for the following element.
                ---@param id integer
                setStyle                       = libphx.HmGui_SetStyle,
                -- Remove element style.
                clearStyle                     = libphx.HmGui_ClearStyle,
                -- Get property type by its id.
                ---@param id integer
                ---@return HmGuiPropertyType
                getPropertyType                = function(...)
                    local instance = libphx.HmGui_GetPropertyType(...)
                    return Core.ManagedObject(instance, libphx.HmGuiPropertyType_Free)
                end,
                -- Write property value into the mapped properties in the active element style.
                ---@param property_id integer
                mapProperty                    = libphx.HmGui_MapProperty,
                -- Remove property by id from the active element style.
                ---@param property_id integer
                removeProperty                 = libphx.HmGui_RemoveProperty,
                ---@param name string
                ---@param value boolean
                ---@param map_id string
                ---@return integer
                registerPropertyBool           = libphx.HmGui_RegisterPropertyBool,
                ---@param name string
                ---@param value integer
                ---@param map_id string
                ---@return integer
                registerPropertyI8             = libphx.HmGui_RegisterPropertyI8,
                ---@param name string
                ---@param value integer
                ---@param map_id string
                ---@return integer
                registerPropertyU8             = libphx.HmGui_RegisterPropertyU8,
                ---@param name string
                ---@param value integer
                ---@param map_id string
                ---@return integer
                registerPropertyI16            = libphx.HmGui_RegisterPropertyI16,
                ---@param name string
                ---@param value integer
                ---@param map_id string
                ---@return integer
                registerPropertyU16            = libphx.HmGui_RegisterPropertyU16,
                ---@param name string
                ---@param value integer
                ---@param map_id string
                ---@return integer
                registerPropertyI32            = libphx.HmGui_RegisterPropertyI32,
                ---@param name string
                ---@param value integer
                ---@param map_id string
                ---@return integer
                registerPropertyU32            = libphx.HmGui_RegisterPropertyU32,
                ---@param name string
                ---@param value integer
                ---@param map_id string
                ---@return integer
                registerPropertyI64            = libphx.HmGui_RegisterPropertyI64,
                ---@param name string
                ---@param value integer
                ---@param map_id string
                ---@return integer
                registerPropertyU64            = libphx.HmGui_RegisterPropertyU64,
                ---@param name string
                ---@param value number
                ---@param map_id string
                ---@return integer
                registerPropertyF32            = libphx.HmGui_RegisterPropertyF32,
                ---@param name string
                ---@param value number
                ---@param map_id string
                ---@return integer
                registerPropertyF64            = libphx.HmGui_RegisterPropertyF64,
                ---@param name string
                ---@param value Vec2
                ---@param map_id string
                ---@return integer
                registerPropertyVec2           = libphx.HmGui_RegisterPropertyVec2,
                ---@param name string
                ---@param value Vec3
                ---@param map_id string
                ---@return integer
                registerPropertyVec3           = libphx.HmGui_RegisterPropertyVec3,
                ---@param name string
                ---@param value Vec4
                ---@param map_id string
                ---@return integer
                registerPropertyVec4           = libphx.HmGui_RegisterPropertyVec4,
                ---@param name string
                ---@param value IVec2
                ---@param map_id string
                ---@return integer
                registerPropertyIVec2          = libphx.HmGui_RegisterPropertyIVec2,
                ---@param name string
                ---@param value IVec3
                ---@param map_id string
                ---@return integer
                registerPropertyIVec3          = libphx.HmGui_RegisterPropertyIVec3,
                ---@param name string
                ---@param value IVec4
                ---@param map_id string
                ---@return integer
                registerPropertyIVec4          = libphx.HmGui_RegisterPropertyIVec4,
                ---@param name string
                ---@param value UVec2
                ---@param map_id string
                ---@return integer
                registerPropertyUVec2          = libphx.HmGui_RegisterPropertyUVec2,
                ---@param name string
                ---@param value UVec3
                ---@param map_id string
                ---@return integer
                registerPropertyUVec3          = libphx.HmGui_RegisterPropertyUVec3,
                ---@param name string
                ---@param value UVec4
                ---@param map_id string
                ---@return integer
                registerPropertyUVec4          = libphx.HmGui_RegisterPropertyUVec4,
                ---@param name string
                ---@param value DVec2
                ---@param map_id string
                ---@return integer
                registerPropertyDVec2          = libphx.HmGui_RegisterPropertyDVec2,
                ---@param name string
                ---@param value DVec3
                ---@param map_id string
                ---@return integer
                registerPropertyDVec3          = libphx.HmGui_RegisterPropertyDVec3,
                ---@param name string
                ---@param value DVec4
                ---@param map_id string
                ---@return integer
                registerPropertyDVec4          = libphx.HmGui_RegisterPropertyDVec4,
                ---@param name string
                ---@param value Color
                ---@param map_id string
                ---@return integer
                registerPropertyColor          = libphx.HmGui_RegisterPropertyColor,
                ---@param name string
                ---@param value Box3
                ---@param map_id string
                ---@return integer
                registerPropertyBox3           = libphx.HmGui_RegisterPropertyBox3,
                ---@param name string
                ---@param value string
                ---@param map_id string
                ---@return integer
                registerPropertyString         = libphx.HmGui_RegisterPropertyString,
                ---@param name string
                ---@param value Font
                ---@param map_id string
                ---@return integer
                registerPropertyFont           = libphx.HmGui_RegisterPropertyFont,
                ---@param property_id integer
                ---@param value boolean
                setPropertyBool                = libphx.HmGui_SetPropertyBool,
                ---@param property_id integer
                ---@param value integer
                setPropertyI8                  = libphx.HmGui_SetPropertyI8,
                ---@param property_id integer
                ---@param value integer
                setPropertyU8                  = libphx.HmGui_SetPropertyU8,
                ---@param property_id integer
                ---@param value integer
                setPropertyI16                 = libphx.HmGui_SetPropertyI16,
                ---@param property_id integer
                ---@param value integer
                setPropertyU16                 = libphx.HmGui_SetPropertyU16,
                ---@param property_id integer
                ---@param value integer
                setPropertyI32                 = libphx.HmGui_SetPropertyI32,
                ---@param property_id integer
                ---@param value integer
                setPropertyU32                 = libphx.HmGui_SetPropertyU32,
                ---@param property_id integer
                ---@param value integer
                setPropertyI64                 = libphx.HmGui_SetPropertyI64,
                ---@param property_id integer
                ---@param value integer
                setPropertyU64                 = libphx.HmGui_SetPropertyU64,
                ---@param property_id integer
                ---@param value number
                setPropertyF32                 = libphx.HmGui_SetPropertyF32,
                ---@param property_id integer
                ---@param value number
                setPropertyF64                 = libphx.HmGui_SetPropertyF64,
                ---@param property_id integer
                ---@param value Vec2
                setPropertyVec2                = libphx.HmGui_SetPropertyVec2,
                ---@param property_id integer
                ---@param value Vec3
                setPropertyVec3                = libphx.HmGui_SetPropertyVec3,
                ---@param property_id integer
                ---@param value Vec4
                setPropertyVec4                = libphx.HmGui_SetPropertyVec4,
                ---@param property_id integer
                ---@param value IVec2
                setPropertyIVec2               = libphx.HmGui_SetPropertyIVec2,
                ---@param property_id integer
                ---@param value IVec3
                setPropertyIVec3               = libphx.HmGui_SetPropertyIVec3,
                ---@param property_id integer
                ---@param value IVec4
                setPropertyIVec4               = libphx.HmGui_SetPropertyIVec4,
                ---@param property_id integer
                ---@param value UVec2
                setPropertyUVec2               = libphx.HmGui_SetPropertyUVec2,
                ---@param property_id integer
                ---@param value UVec3
                setPropertyUVec3               = libphx.HmGui_SetPropertyUVec3,
                ---@param property_id integer
                ---@param value UVec4
                setPropertyUVec4               = libphx.HmGui_SetPropertyUVec4,
                ---@param property_id integer
                ---@param value DVec2
                setPropertyDVec2               = libphx.HmGui_SetPropertyDVec2,
                ---@param property_id integer
                ---@param value DVec3
                setPropertyDVec3               = libphx.HmGui_SetPropertyDVec3,
                ---@param property_id integer
                ---@param value DVec4
                setPropertyDVec4               = libphx.HmGui_SetPropertyDVec4,
                ---@param property_id integer
                ---@param value Color
                setPropertyColor               = libphx.HmGui_SetPropertyColor,
                ---@param property_id integer
                ---@param value Box3
                setPropertyBox3                = libphx.HmGui_SetPropertyBox3,
                ---@param property_id integer
                ---@param value string
                setPropertyString              = libphx.HmGui_SetPropertyString,
                ---@param property_id integer
                ---@param value Font
                setPropertyFont                = libphx.HmGui_SetPropertyFont,
                ---@param property_id integer
                ---@return boolean
                getPropertyBool                = libphx.HmGui_GetPropertyBool,
                ---@param property_id integer
                ---@return integer
                getPropertyI8                  = libphx.HmGui_GetPropertyI8,
                ---@param property_id integer
                ---@return integer
                getPropertyU8                  = libphx.HmGui_GetPropertyU8,
                ---@param property_id integer
                ---@return integer
                getPropertyI16                 = libphx.HmGui_GetPropertyI16,
                ---@param property_id integer
                ---@return integer
                getPropertyU16                 = libphx.HmGui_GetPropertyU16,
                ---@param property_id integer
                ---@return integer
                getPropertyI32                 = libphx.HmGui_GetPropertyI32,
                ---@param property_id integer
                ---@return integer
                getPropertyU32                 = libphx.HmGui_GetPropertyU32,
                ---@param property_id integer
                ---@return integer
                getPropertyI64                 = libphx.HmGui_GetPropertyI64,
                ---@param property_id integer
                ---@return integer
                getPropertyU64                 = libphx.HmGui_GetPropertyU64,
                ---@param property_id integer
                ---@return number
                getPropertyF32                 = libphx.HmGui_GetPropertyF32,
                ---@param property_id integer
                ---@return number
                getPropertyF64                 = libphx.HmGui_GetPropertyF64,
                ---@param property_id integer
                ---@return Vec2
                getPropertyVec2                = libphx.HmGui_GetPropertyVec2,
                ---@param property_id integer
                ---@return Vec3
                getPropertyVec3                = libphx.HmGui_GetPropertyVec3,
                ---@param property_id integer
                ---@return Vec4
                getPropertyVec4                = libphx.HmGui_GetPropertyVec4,
                ---@param property_id integer
                ---@return IVec2
                getPropertyIVec2               = libphx.HmGui_GetPropertyIVec2,
                ---@param property_id integer
                ---@return IVec3
                getPropertyIVec3               = libphx.HmGui_GetPropertyIVec3,
                ---@param property_id integer
                ---@return IVec4
                getPropertyIVec4               = libphx.HmGui_GetPropertyIVec4,
                ---@param property_id integer
                ---@return UVec2
                getPropertyUVec2               = libphx.HmGui_GetPropertyUVec2,
                ---@param property_id integer
                ---@return UVec3
                getPropertyUVec3               = libphx.HmGui_GetPropertyUVec3,
                ---@param property_id integer
                ---@return UVec4
                getPropertyUVec4               = libphx.HmGui_GetPropertyUVec4,
                ---@param property_id integer
                ---@return DVec2
                getPropertyDVec2               = libphx.HmGui_GetPropertyDVec2,
                ---@param property_id integer
                ---@return DVec3
                getPropertyDVec3               = libphx.HmGui_GetPropertyDVec3,
                ---@param property_id integer
                ---@return DVec4
                getPropertyDVec4               = libphx.HmGui_GetPropertyDVec4,
                ---@param property_id integer
                ---@return Color
                getPropertyColor               = libphx.HmGui_GetPropertyColor,
                ---@param property_id integer
                ---@return Box3
                getPropertyBox3                = libphx.HmGui_GetPropertyBox3,
                ---@param property_id integer
                ---@return string
                getPropertyString              = libphx.HmGui_GetPropertyString,
                ---@param property_id integer
                ---@return Font
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
