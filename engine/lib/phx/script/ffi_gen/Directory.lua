-- Directory -------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef struct Directory {} Directory;
    ]]

    return 1, 'Directory'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local Directory

    do -- C Definitions
        ffi.cdef [[
            void             Directory_Free        (Directory*);
            Directory const* Directory_Open        (cstr path);
            cstr             Directory_GetNext     (Directory*);
            bool             Directory_Change      (cstr cwd);
            bool             Directory_Create      (cstr path);
            cstr             Directory_GetCurrent  ();
            cstr             Directory_GetPrefPath (cstr org, cstr app);
            bool             Directory_Remove      (cstr path);
        ]]
    end

    do -- Global Symbol Table
        Directory = {
            ---@param path cstr
            ---@return Directory const*
            Open        = function(...)
                local instance = libphx.Directory_Open(...)
                return Core.ManagedObject(instance, libphx.Directory_Free)
            end,
            ---@param cwd cstr
            ---@return bool
            Change      = libphx.Directory_Change,
            ---@param path cstr
            ---@return bool
            Create      = libphx.Directory_Create,
            ---@return cstr
            GetCurrent  = libphx.Directory_GetCurrent,
            ---@param org cstr
            ---@param app cstr
            ---@return cstr
            GetPrefPath = libphx.Directory_GetPrefPath,
            ---@param path cstr
            ---@return bool
            Remove      = libphx.Directory_Remove,
        }

        if onDef_Directory then onDef_Directory(Directory, mt) end
        Directory = setmetatable(Directory, mt)
    end

    do -- Metatype for class instances
        local t  = ffi.typeof('Directory')
        local mt = {
            __index = {
                ---@return cstr
                getNext = libphx.Directory_GetNext,
            },
        }

        if onDef_Directory_t then onDef_Directory_t(t, mt) end
        Directory_t = ffi.metatype(t, mt)
    end

    return Directory
end

return Loader
