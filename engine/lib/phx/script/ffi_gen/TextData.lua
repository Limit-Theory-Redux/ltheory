-- TextData --------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef struct TextData {} TextData;
    ]]

    return 1, 'TextData'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local TextData

    do -- C Definitions
        ffi.cdef [[
            void      TextData_Free            (TextData*);
            TextData* TextData_Create          (cstr text, TextStyle const* defaultStyle, TextAlignment alignment);
            void      TextData_SetSectionStyle (TextData*, uint64 startPos, uint64 endPos, TextStyle const* style);
        ]]
    end

    do -- Global Symbol Table
        TextData = {
            Create          = function(...)
                local instance = libphx.TextData_Create(...)
                return Core.ManagedObject(instance, libphx.TextData_Free)
            end,
        }

        if onDef_TextData then onDef_TextData(TextData, mt) end
        TextData = setmetatable(TextData, mt)
    end

    do -- Metatype for class instances
        local t  = ffi.typeof('TextData')
        local mt = {
            __index = {
                setSectionStyle = libphx.TextData_SetSectionStyle,
            },
        }

        if onDef_TextData_t then onDef_TextData_t(t, mt) end
        TextData_t = ffi.metatype(t, mt)
    end

    return TextData
end

return Loader
