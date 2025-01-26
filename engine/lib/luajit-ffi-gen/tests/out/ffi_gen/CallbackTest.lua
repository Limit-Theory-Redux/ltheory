-- AUTO GENERATED. DO NOT MODIFY!
-- CallbackTest ----------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef struct CallbackTest {} CallbackTest;
    ]]

    return 1, 'CallbackTest'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local CallbackTest

    do -- C Definitions
        ffi.cdef [[
            void   CallbackTest_Free                  (CallbackTest*);
            void   CallbackTest_NthPrimitive          (CallbackTest const*, uint64 index, void (*)(float));
            void   CallbackTest_NthPrimitiveRef       (CallbackTest const*, uint64 index, void (*)(float const*));
            void   CallbackTest_NthPrimitiveMut       (CallbackTest*, uint64 index, void (*)(float*));
            void   CallbackTest_NthPrimitiveValOpt    (CallbackTest const*, uint64 index, void (*)(float const*));
            void   CallbackTest_NthPrimitiveRefOpt    (CallbackTest const*, uint64 index, void (*)(float const*));
            void   CallbackTest_NthPrimitiveMutOpt    (CallbackTest*, uint64 index, void (*)(float*));
            void   CallbackTest_InsertPrimitive       (CallbackTest*, float (*)());
            void   CallbackTest_NthManaged            (CallbackTest const*, uint64 index, void (*)(ManagedData*));
            void   CallbackTest_NthManagedRef         (CallbackTest const*, uint64 index, void (*)(ManagedData const*));
            void   CallbackTest_NthManagedMut         (CallbackTest*, uint64 index, void (*)(ManagedData*));
            void   CallbackTest_NthManagedValOpt      (CallbackTest const*, uint64 index, void (*)(ManagedData*));
            void   CallbackTest_NthManagedRefOpt      (CallbackTest const*, uint64 index, void (*)(ManagedData const*));
            void   CallbackTest_NthManagedMutOpt      (CallbackTest*, uint64 index, void (*)(ManagedData*));
            void   CallbackTest_InsertManaged         (CallbackTest*, ManagedData* (*)());
            void   CallbackTest_ReadPrimitiveArray    (CallbackTest const*, void (*)(float const*, uint64));
            void   CallbackTest_LockPrimitiveArray    (CallbackTest*, void (*)(float*, uint64));
            void   CallbackTest_ReadManagedArray      (CallbackTest const*, void (*)(ManagedData const*, uint64));
            void   CallbackTest_LockManagedArray      (CallbackTest*, void (*)(ManagedData*, uint64));
            cstr   CallbackTest_TransformString       (cstr s, cstr (*)(cstr));
            cstr   CallbackTest_TransformStr          (cstr s, cstr (*)(cstr));
            void   CallbackTest_GetMultipleAndReplace (CallbackTest*, uint64 index, float (*)(float, ManagedData const*));
            uint32 CallbackTest_Passthrough           (uint32 input, uint32 (*)(uint32));
        ]]
    end

    do -- Global Symbol Table
        CallbackTest = {
            TransformString       = libphx.CallbackTest_TransformString,
            TransformStr          = libphx.CallbackTest_TransformStr,
            Passthrough           = libphx.CallbackTest_Passthrough,
        }

        if onDef_CallbackTest then onDef_CallbackTest(CallbackTest, mt) end
        CallbackTest = setmetatable(CallbackTest, mt)
    end

    do -- Metatype for class instances
        local t  = ffi.typeof('CallbackTest')
        local mt = {
            __index = {
                nthPrimitive          = libphx.CallbackTest_NthPrimitive,
                nthPrimitiveRef       = libphx.CallbackTest_NthPrimitiveRef,
                nthPrimitiveMut       = libphx.CallbackTest_NthPrimitiveMut,
                nthPrimitiveValOpt    = libphx.CallbackTest_NthPrimitiveValOpt,
                nthPrimitiveRefOpt    = libphx.CallbackTest_NthPrimitiveRefOpt,
                nthPrimitiveMutOpt    = libphx.CallbackTest_NthPrimitiveMutOpt,
                insertPrimitive       = libphx.CallbackTest_InsertPrimitive,
                nthManaged            = libphx.CallbackTest_NthManaged,
                nthManagedRef         = libphx.CallbackTest_NthManagedRef,
                nthManagedMut         = libphx.CallbackTest_NthManagedMut,
                nthManagedValOpt      = libphx.CallbackTest_NthManagedValOpt,
                nthManagedRefOpt      = libphx.CallbackTest_NthManagedRefOpt,
                nthManagedMutOpt      = libphx.CallbackTest_NthManagedMutOpt,
                insertManaged         = libphx.CallbackTest_InsertManaged,
                readPrimitiveArray    = libphx.CallbackTest_ReadPrimitiveArray,
                lockPrimitiveArray    = libphx.CallbackTest_LockPrimitiveArray,
                readManagedArray      = libphx.CallbackTest_ReadManagedArray,
                lockManagedArray      = libphx.CallbackTest_LockManagedArray,
                getMultipleAndReplace = libphx.CallbackTest_GetMultipleAndReplace,
            },
        }

        if onDef_CallbackTest_t then onDef_CallbackTest_t(t, mt) end
        CallbackTest_t = ffi.metatype(t, mt)
    end

    return CallbackTest
end

return Loader
