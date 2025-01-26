-- TextStyle -------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef struct TextStyle {} TextStyle;
    ]]

    return 1, 'TextStyle'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local TextStyle

    do -- C Definitions
        ffi.cdef [[
            void       TextStyle_Free                   (TextStyle*);
            TextStyle* TextStyle_Create                 ();
            void       TextStyle_SetFontFamily          (TextStyle*, cstr family);
            void       TextStyle_SetFontSize            (TextStyle*, float size);
            void       TextStyle_SetFontStretch         (TextStyle*, float stretch);
            void       TextStyle_SetFontItalic          (TextStyle*, bool italic);
            void       TextStyle_SetFontWeight          (TextStyle*, float weight);
            void       TextStyle_SetLocale              (TextStyle*, cstr locale);
            void       TextStyle_SetBrush               (TextStyle*, Color const* color);
            void       TextStyle_SetUnderline           (TextStyle*, bool underline);
            void       TextStyle_SetUnderlineOffset     (TextStyle*, float offset);
            void       TextStyle_SetUnderlineSize       (TextStyle*, float size);
            void       TextStyle_SetUnderlineBrush      (TextStyle*, Color const* color);
            void       TextStyle_SetStrikethrough       (TextStyle*, bool strikethrough);
            void       TextStyle_SetStrikethroughOffset (TextStyle*, float offset);
            void       TextStyle_SetStrikethroughSize   (TextStyle*, float size);
            void       TextStyle_SetStrikethroughBrush  (TextStyle*, Color const* color);
            void       TextStyle_SetLineHeight          (TextStyle*, float height);
            void       TextStyle_SetWordSpacing         (TextStyle*, float size);
            void       TextStyle_SetLetterSpacing       (TextStyle*, float size);
        ]]
    end

    do -- Global Symbol Table
        TextStyle = {
            Create                 = function()
                local _instance = libphx.TextStyle_Create()
                return Core.ManagedObject(_instance, libphx.TextStyle_Free)
            end,
        }

        if onDef_TextStyle then onDef_TextStyle(TextStyle, mt) end
        TextStyle = setmetatable(TextStyle, mt)
    end

    do -- Metatype for class instances
        local t  = ffi.typeof('TextStyle')
        local mt = {
            __index = {
                setFontFamily          = libphx.TextStyle_SetFontFamily,
                setFontSize            = libphx.TextStyle_SetFontSize,
                setFontStretch         = libphx.TextStyle_SetFontStretch,
                setFontItalic          = libphx.TextStyle_SetFontItalic,
                setFontWeight          = libphx.TextStyle_SetFontWeight,
                setLocale              = libphx.TextStyle_SetLocale,
                setBrush               = libphx.TextStyle_SetBrush,
                setUnderline           = libphx.TextStyle_SetUnderline,
                setUnderlineOffset     = libphx.TextStyle_SetUnderlineOffset,
                setUnderlineSize       = libphx.TextStyle_SetUnderlineSize,
                setUnderlineBrush      = libphx.TextStyle_SetUnderlineBrush,
                setStrikethrough       = libphx.TextStyle_SetStrikethrough,
                setStrikethroughOffset = libphx.TextStyle_SetStrikethroughOffset,
                setStrikethroughSize   = libphx.TextStyle_SetStrikethroughSize,
                setStrikethroughBrush  = libphx.TextStyle_SetStrikethroughBrush,
                setLineHeight          = libphx.TextStyle_SetLineHeight,
                setWordSpacing         = libphx.TextStyle_SetWordSpacing,
                setLetterSpacing       = libphx.TextStyle_SetLetterSpacing,
            },
        }

        if onDef_TextStyle_t then onDef_TextStyle_t(t, mt) end
        TextStyle_t = ffi.metatype(t, mt)
    end

    return TextStyle
end

return Loader
