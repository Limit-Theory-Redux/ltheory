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
            void  Font_Draw          (Font const*, cstr text, float x, float y, float r, float g, float b, float a);
            void  Font_DrawShaded    (Font const*, cstr text, float x, float y);
            int   Font_GetLineHeight (Font const*);
            void  Font_GetSize       (Font const*, cstr text, Vec4i* out);
            Vec2i Font_GetSize2      (Font const*, cstr text);
        ]]
    end

    do -- Global Symbol Table
        Font = {
            Load          = function(...)
                local instance = libphx.Font_Load(...)
                ffi.gc(instance, libphx.Font_Free)
                return instance
            end,
            Draw          = libphx.Font_Draw,
            DrawShaded    = libphx.Font_DrawShaded,
            GetLineHeight = libphx.Font_GetLineHeight,
            GetSize       = libphx.Font_GetSize,
            GetSize2      = libphx.Font_GetSize2,
        }

        if onDef_Font then onDef_Font(Font, mt) end
        Font = setmetatable(Font, mt)
    end

    do -- Metatype for class instances
        local t  = ffi.typeof('Font')
        local mt = {
            __index = {
                draw          = libphx.Font_Draw,
                drawShaded    = libphx.Font_DrawShaded,
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
