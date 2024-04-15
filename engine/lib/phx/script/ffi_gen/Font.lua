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
            ---@param name cstr
            ---@param size uint32
            ---@return Font*
            Load          = function(...)
                local instance = libphx.Font_Load(...)
                return Core.ManagedObject(instance, libphx.Font_Free)
            end,
        }

        if onDef_Font then onDef_Font(Font, mt) end
        Font = setmetatable(Font, mt)
    end

    do -- Metatype for class instances
        local t  = ffi.typeof('Font')
        local mt = {
            __index = {
                ---@param text cstr
                ---@param x float
                ---@param y float
                ---@param r float
                ---@param g float
                ---@param b float
                ---@param a float
                draw          = libphx.Font_Draw,
                ---@param text cstr
                ---@param x float
                ---@param y float
                drawShaded    = libphx.Font_DrawShaded,
                ---@return int
                getLineHeight = libphx.Font_GetLineHeight,
                ---@param text cstr
                ---@param out Vec4i*
                getSize       = libphx.Font_GetSize,
                ---@param text cstr
                ---@return Vec2i
                getSize2      = libphx.Font_GetSize2,
            },
        }

        if onDef_Font_t then onDef_Font_t(t, mt) end
        Font_t = ffi.metatype(t, mt)
    end

    return Font
end

return Loader
