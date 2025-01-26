-- AUTO GENERATED. DO NOT MODIFY!
-- EnumTest --------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef struct EnumTest {} EnumTest;
    ]]

    return 1, 'EnumTest'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local EnumTest

    do -- C Definitions
        ffi.cdef [[
            void      EnumTest_Free (EnumTest*);
            EnumTest* EnumTest_New  (MyEnum1 const* myEnum);
            MyEnum1*  EnumTest_Get  (EnumTest const*);
        ]]
    end

    do -- Global Symbol Table
        EnumTest = {
            New  = function(myEnum)
                local _instance = libphx.EnumTest_New(myEnum)
                return Core.ManagedObject(_instance, libphx.EnumTest_Free)
            end,
        }

        if onDef_EnumTest then onDef_EnumTest(EnumTest, mt) end
        EnumTest = setmetatable(EnumTest, mt)
    end

    do -- Metatype for class instances
        local t  = ffi.typeof('EnumTest')
        local mt = {
            __index = {
                get  = function(self)
                    local _instance = libphx.EnumTest_Get(self)
                    return Core.ManagedObject(_instance, libphx.MyEnum1_Free)
                end,
            },
        }

        if onDef_EnumTest_t then onDef_EnumTest_t(t, mt) end
        EnumTest_t = ffi.metatype(t, mt)
    end

    return EnumTest
end

return Loader
