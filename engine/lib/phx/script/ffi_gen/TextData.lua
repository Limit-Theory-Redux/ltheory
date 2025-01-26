-- AUTO GENERATED. DO NOT MODIFY!
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
            void      TextData_Free              (TextData*);
            TextData* TextData_Create            (cstr text, TextStyle const* defaultStyle, Color const* cursorColor, Color const* selectionColor, TextAlignment alignment, bool multiline);
            cstr      TextData_Text              (TextData const*);
            void      TextData_SetText           (TextData*, cstr text);
            void      TextData_SetScaleFactor    (TextData*, float scaleFactor);
            bool      TextData_IsMultiline       (TextData const*);
            void      TextData_SetMultiline      (TextData*, bool multiline);
            void      TextData_SetSectionStyle   (TextData*, uint64 startPos, uint64 endPos, TextStyle const* style);
            void      TextData_SetCursorPos      (TextData*, uint64 pos);
            void      TextData_SetSelectionColor (TextData*, Color const* color);
            void      TextData_SetSelection      (TextData*, uint64 startPos, uint64 endPos);
        ]]
    end

    do -- Global Symbol Table
        TextData = {
            Create            = function(text, defaultStyle, cursorColor, selectionColor, alignment, multiline)
                local _instance = libphx.TextData_Create(text, defaultStyle, cursorColor, selectionColor, alignment, multiline)
                return Core.ManagedObject(_instance, libphx.TextData_Free)
            end,
        }

        if onDef_TextData then onDef_TextData(TextData, mt) end
        TextData = setmetatable(TextData, mt)
    end

    do -- Metatype for class instances
        local t  = ffi.typeof('TextData')
        local mt = {
            __index = {
                text              = libphx.TextData_Text,
                setText           = libphx.TextData_SetText,
                setScaleFactor    = libphx.TextData_SetScaleFactor,
                isMultiline       = libphx.TextData_IsMultiline,
                setMultiline      = libphx.TextData_SetMultiline,
                setSectionStyle   = libphx.TextData_SetSectionStyle,
                setCursorPos      = libphx.TextData_SetCursorPos,
                setSelectionColor = libphx.TextData_SetSelectionColor,
                setSelection      = libphx.TextData_SetSelection,
            },
        }

        if onDef_TextData_t then onDef_TextData_t(t, mt) end
        TextData_t = ffi.metatype(t, mt)
    end

    return TextData
end

return Loader
