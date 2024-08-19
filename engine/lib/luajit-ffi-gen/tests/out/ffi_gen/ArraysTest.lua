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
            void ArraysTest_SetPrimitiveSlice  (ArraysTest*, uint32 const* data, uint64 data_size);
            void ArraysTest_GetPrimitiveSlice  (ArraysTest const*, uint32* out, uint64 out_size);
            void ArraysTest_SetManagedSlice    (ArraysTest*, ManagedData const* data, uint64 data_size);
            void ArraysTest_GetManagedSlice    (ArraysTest const*, ManagedData* out, uint64 out_size);
            void ArraysTest_MovePrimitiveArray (ArraysTest*, uint32 const* data, uint64 data_size);
            void ArraysTest_SetPrimitiveArray  (ArraysTest*, uint32 const* data, uint64 data_size);
            void ArraysTest_GetPrimitiveArray  (ArraysTest const*, uint32* out, uint64 out_size);
            void ArraysTest_MoveManagedArray   (ArraysTest*, ManagedData const* data, uint64 data_size);
            void ArraysTest_SetManagedArray    (ArraysTest*, ManagedData const* data, uint64 data_size);
            void ArraysTest_GetManagedArray    (ArraysTest const*, ManagedData* out, uint64 out_size);
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
                setManagedSlice    = libphx.ArraysTest_SetManagedSlice,
                getManagedSlice    = libphx.ArraysTest_GetManagedSlice,
                movePrimitiveArray = libphx.ArraysTest_MovePrimitiveArray,
                setPrimitiveArray  = libphx.ArraysTest_SetPrimitiveArray,
                getPrimitiveArray  = libphx.ArraysTest_GetPrimitiveArray,
                moveManagedArray   = libphx.ArraysTest_MoveManagedArray,
                setManagedArray    = libphx.ArraysTest_SetManagedArray,
                getManagedArray    = libphx.ArraysTest_GetManagedArray,
            },
        }

        if onDef_ArraysTest_t then onDef_ArraysTest_t(t, mt) end
        ArraysTest_t = ffi.metatype(t, mt)
    end

    return ArraysTest
end

return Loader
