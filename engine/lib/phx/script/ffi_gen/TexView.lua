-- AUTO GENERATED. DO NOT MODIFY!
-- TexView ---------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef struct TexView {} TexView;
    ]]

    return 1, 'TexView'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local TexView

    do -- C Definitions
        ffi.cdef [[
            void TexView_Free       (TexView*);
            int  TexView_GetWidth   (TexView const*);
            int  TexView_GetHeight  (TexView const*);
            int  TexView_GetBaseMip (TexView const*);
        ]]
    end

    do -- Global Symbol Table
        TexView = {}

        if onDef_TexView then onDef_TexView(TexView, mt) end
        TexView = setmetatable(TexView, mt)
    end

    do -- Metatype for class instances
        local t  = ffi.typeof('TexView')
        local mt = {
            __index = {
                getWidth   = libphx.TexView_GetWidth,
                getHeight  = libphx.TexView_GetHeight,
                getBaseMip = libphx.TexView_GetBaseMip,
            },
        }

        if onDef_TexView_t then onDef_TexView_t(t, mt) end
        TexView_t = ffi.metatype(t, mt)
    end

    return TexView
end

return Loader
