-- Font ------------------------------------------------------------------------

---@class Font
---@field Load fun(name: string, size: integer): Font
---@field Draw fun(self, text: string, x: number, y: number, r: number, g: number, b: number, a: number)
---@field DrawShaded fun(self, text: string, x: number, y: number)
---@field GetLineHeight fun(self): integer
---@field GetSize fun(self, text: string, out: IVec4)
---@field GetSize2 fun(self, text: string): IVec2

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
            ---@param name string
            ---@param size integer
            ---@return Font
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
                ---@param text string
                ---@param x number
                ---@param y number
                ---@param r number
                ---@param g number
                ---@param b number
                ---@param a number
                draw          = libphx.Font_Draw,
                ---@param text string
                ---@param x number
                ---@param y number
                drawShaded    = libphx.Font_DrawShaded,
                ---@return integer
                getLineHeight = libphx.Font_GetLineHeight,
                ---@param text string
                ---@param out IVec4
                getSize       = libphx.Font_GetSize,
                ---@param text string
                ---@return IVec2
                getSize2      = libphx.Font_GetSize2,
            },
        }

        if onDef_Font_t then onDef_Font_t(t, mt) end
        Font_t = ffi.metatype(t, mt)
    end

    return Font
end

return Loader
