-- AUTO GENERATED. DO NOT MODIFY!
-- TriangleTest ----------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef struct TriangleTest {
            struct Triangle* triangle;
            bool             hit;
        } TriangleTest;
    ]]

    return 1, 'TriangleTest'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local TriangleTest

    do -- C Definitions
        ffi.cdef [[
            void          TriangleTest_Free   (TriangleTest*);
            TriangleTest* TriangleTest_Create ();
        ]]
    end

    do -- Global Symbol Table
        TriangleTest = {
            Create = function()
                local _instance = libphx.TriangleTest_Create()
                return Core.ManagedObject(_instance, libphx.TriangleTest_Free)
            end,
        }

        if onDef_TriangleTest then onDef_TriangleTest(TriangleTest, mt) end
        TriangleTest = setmetatable(TriangleTest, mt)
    end

    return TriangleTest
end

return Loader
