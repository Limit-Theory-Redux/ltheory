-- ArraysTest ------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef struct ArraysTest {} ArraysTest;
    ]]

    return 1, 'ArraysTest'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local ArraysTest

    do -- C Definitions
        ffi.cdef [[
            void ArraysTest_Free               (ArraysTest*);
            void ArraysTest_SetPrimitiveSlice  (ArraysTest*, float const* data, uint64 data_size);
            void ArraysTest_GetPrimitiveSlice  (ArraysTest const*, float* out, uint64 out_size);
            void ArraysTest_SetCustomSlice     (ArraysTest*, Data const* data, uint64 data_size);
            void ArraysTest_GetCustomSlice     (ArraysTest const*, Data* out, uint64 out_size);
            void ArraysTest_MovePrimitiveArray (ArraysTest*, float const* data, uint64 data_size);
            void ArraysTest_SetPrimitiveArray  (ArraysTest*, float const* data, uint64 data_size);
            void ArraysTest_GetPrimitiveArray  (ArraysTest const*, float* out, uint64 out_size);
            void ArraysTest_MoveCustomArray    (ArraysTest*, Data const* data, uint64 data_size);
            void ArraysTest_SetCustomArray     (ArraysTest*, Data const* data, uint64 data_size);
            void ArraysTest_GetCustomArray     (ArraysTest const*, Data* out, uint64 out_size);
        ]]
    end

    do -- Global Symbol Table
        ArraysTest = {}

        if onDef_ArraysTest then onDef_ArraysTest(ArraysTest, mt) end
        ArraysTest = setmetatable(ArraysTest, mt)
    end

    do -- Metatype for class instances
        local t  = ffi.typeof('ArraysTest')
        local mt = {
            __index = {
                setPrimitiveSlice  = libphx.ArraysTest_SetPrimitiveSlice,
                getPrimitiveSlice  = libphx.ArraysTest_GetPrimitiveSlice,
                setCustomSlice     = libphx.ArraysTest_SetCustomSlice,
                getCustomSlice     = libphx.ArraysTest_GetCustomSlice,
                movePrimitiveArray = libphx.ArraysTest_MovePrimitiveArray,
                setPrimitiveArray  = libphx.ArraysTest_SetPrimitiveArray,
                getPrimitiveArray  = libphx.ArraysTest_GetPrimitiveArray,
                moveCustomArray    = libphx.ArraysTest_MoveCustomArray,
                setCustomArray     = libphx.ArraysTest_SetCustomArray,
                getCustomArray     = libphx.ArraysTest_GetCustomArray,
            },
        }

        if onDef_ArraysTest_t then onDef_ArraysTest_t(t, mt) end
        ArraysTest_t = ffi.metatype(t, mt)
    end

    return ArraysTest
end

return Loader
