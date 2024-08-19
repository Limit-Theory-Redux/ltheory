-- BoxTest ---------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef struct BoxTest {} BoxTest;
    ]]

    return 1, 'BoxTest'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local BoxTest

    do -- C Definitions
        ffi.cdef [[
            void         BoxTest_Free       (BoxTest*);
            void         BoxTest_SetManaged (BoxTest*, ManagedData* val);
            ManagedData* BoxTest_GetManaged (BoxTest const*);
        ]]
    end

    do -- Global Symbol Table
        BoxTest = {}

        if onDef_BoxTest then onDef_BoxTest(BoxTest, mt) end
        BoxTest = setmetatable(BoxTest, mt)
    end

    do -- Metatype for class instances
        local t  = ffi.typeof('BoxTest')
        local mt = {
            __index = {
                setManaged = libphx.BoxTest_SetManaged,
                getManaged = function(...)
                    local instance = libphx.BoxTest_GetManaged(...)
                    return Core.ManagedObject(instance, libphx.ManagedData_Free)
                end,
            },
        }

        if onDef_BoxTest_t then onDef_BoxTest_t(t, mt) end
        BoxTest_t = ffi.metatype(t, mt)
    end

    return BoxTest
end

return Loader
