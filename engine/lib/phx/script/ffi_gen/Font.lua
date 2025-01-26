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
            Font* Font_Load          (cstr name, uint32 size);
            void  Font_Draw          (Font const*, cstr text, float x, float y, Color const* color);
            int   Font_GetLineHeight (Font const*);
            void  Font_GetSize       (Font const*, cstr text, Vec4i* out);
            Vec2i Font_GetSize2      (Font const*, cstr text);
        ]]
    end

    do -- Global Symbol Table
        Font = {
            Load          = function(name, size)
                local _instance = libphx.Font_Load(name, size)
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
