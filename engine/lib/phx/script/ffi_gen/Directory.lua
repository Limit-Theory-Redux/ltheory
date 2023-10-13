-- Directory -------------------------------------------------------------------
local ffi = require('ffi')
local libphx = require('libphx').lib
local Directory

function declareType()
    ffi.cdef [[
        typedef struct Directory {} Directory;
    ]]

    return 1, 'Directory'
end

do -- C Definitions
    ffi.cdef [[
        void       Directory_Free        (Directory*);
        Directory* Directory_Open        (cstr path);
        cstr       Directory_GetNext     (Directory*);
        bool       Directory_Change      (cstr cwd);
        bool       Directory_Create      (cstr path);
        cstr       Directory_GetCurrent  ();
        cstr       Directory_GetPrefPath (cstr org, cstr app);
        bool       Directory_Remove      (cstr path);
    ]]
end

do -- Global Symbol Table
    Directory = {
        Free        = libphx.Directory_Free,
        Open        = libphx.Directory_Open,
        GetNext     = libphx.Directory_GetNext,
        Change      = libphx.Directory_Change,
        Create      = libphx.Directory_Create,
        GetCurrent  = libphx.Directory_GetCurrent,
        GetPrefPath = libphx.Directory_GetPrefPath,
        Remove      = libphx.Directory_Remove,
    }

    if onDef_Directory then onDef_Directory(Directory, mt) end
    Directory = setmetatable(Directory, mt)
end

do -- Metatype for class instances
    local t  = ffi.typeof('Directory')
    local mt = {
        __index = {
            managed = function(self) return ffi.gc(self, libphx.Directory_Free) end,
            free    = libphx.Directory_Free,
            getNext = libphx.Directory_GetNext,
        },
    }

    if onDef_Directory_t then onDef_Directory_t(t, mt) end
    Directory_t = ffi.metatype(t, mt)
end

return Directory
