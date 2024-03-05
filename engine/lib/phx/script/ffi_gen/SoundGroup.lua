-- SoundGroup ------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef uint8 SoundGroup;
    ]]

    return 2, 'SoundGroup'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local SoundGroup

    do -- C Definitions
        ffi.cdef [[
            SoundGroup SoundGroup_Ambient;
            SoundGroup SoundGroup_Effects;
            SoundGroup SoundGroup_Music;

            cstr       SoundGroup_ToString(SoundGroup);
        ]]
    end

    do -- Global Symbol Table
        SoundGroup = {
            Ambient  = libphx.SoundGroup_Ambient,
            Effects  = libphx.SoundGroup_Effects,
            Music    = libphx.SoundGroup_Music,

            ToString = libphx.SoundGroup_ToString,
        }

        if onDef_SoundGroup then onDef_SoundGroup(SoundGroup, mt) end
        SoundGroup = setmetatable(SoundGroup, mt)
    end

    return SoundGroup
end

return Loader
