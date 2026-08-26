-- AUTO GENERATED. DO NOT MODIFY!
-- Font ------------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef struct Font {} Font;
    ]]

    return 1, 'Font'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local Font

    do -- C Definitions
        ffi.cdef [[
            void  Font_Free          (Font*);
            Font* Font_Load          (Renderer* r, cstr name, uint32 size);
            void  Font_Draw          (Font const*, Renderer* r, cstr text, float x, float y, Color const* color);
            int   Font_GetLineHeight (Font const*);
            void  Font_GetSize       (Font const*, Renderer* r, cstr text, Vec4i* out);
            Vec2i Font_GetSize2      (Font const*, Renderer* r, cstr text);
        ]]
    end

    do -- Global Symbol Table
        Font = {
            Load          = function(r, name, size)
                local _instance = libphx.Font_Load(r, name, size)
                return Core.ManagedObject(_instance, libphx.Font_Free)
            end,
        }

        if onDef_Font then onDef_Font(Font, mt) end
        Font = setmetatable(Font, mt)
    end

    do -- Metatype for class instances
        local t  = ffi.typeof('Font')
        local mt = {
            __index = {
                draw          = libphx.Font_Draw,
                getLineHeight = libphx.Font_GetLineHeight,
                getSize       = libphx.Font_GetSize,
                getSize2      = libphx.Font_GetSize2,
            },
        }

        if onDef_Font_t then onDef_Font_t(t, mt) end
        Font_t = ffi.metatype(t, mt)
    end

    return Font
end

return Loader
