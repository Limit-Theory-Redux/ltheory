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
            void   CallbackTest_NthPrimitiveRef       (CallbackTest const*, uint64 index, void (*)(float));
            void   CallbackTest_NthPrimitiveMut       (CallbackTest*, uint64 index, void (*)(float*));
            void   CallbackTest_NthPrimitiveValOpt    (CallbackTest const*, uint64 index, void (*)(float const*));
            void   CallbackTest_NthPrimitiveRefOpt    (CallbackTest const*, uint64 index, void (*)(float const*));
            void   CallbackTest_NthPrimitiveMutOpt    (CallbackTest*, uint64 index, void (*)(float*));
            void   CallbackTest_InsertPrimitive       (CallbackTest*, float (*)());
            void   CallbackTest_NthNoncopyable        (CallbackTest const*, uint64 index, void (*)(Data*));
            void   CallbackTest_NthNoncopyableRef     (CallbackTest const*, uint64 index, void (*)(Data const*));
            void   CallbackTest_NthNoncopyableMut     (CallbackTest*, uint64 index, void (*)(Data*));
            void   CallbackTest_NthNoncopyableValOpt  (CallbackTest const*, uint64 index, void (*)(Data const*));
            void   CallbackTest_NthNoncopyableRefOpt  (CallbackTest const*, uint64 index, void (*)(Data const*));
            void   CallbackTest_NthNoncopyableMutOpt  (CallbackTest*, uint64 index, void (*)(Data*));
            void   CallbackTest_InsertNoncopyable     (CallbackTest*, Data* (*)());
            void   CallbackTest_ReadPrimitiveArray    (CallbackTest const*, void (*)(float const*, uint64));
            void   CallbackTest_LockPrimitiveArray    (CallbackTest*, void (*)(float*, uint64));
            void   CallbackTest_ReadNoncopyableArray  (CallbackTest const*, void (*)(Data const*, uint64));
            void   CallbackTest_LockNoncopyableArray  (CallbackTest*, void (*)(Data*, uint64));
            cstr   CallbackTest_TransformString       (cstr s, cstr (*)(cstr));
            cstr   CallbackTest_TransformStr          (cstr s, cstr (*)(cstr));
            void   CallbackTest_GetMultipleAndReplace (CallbackTest*, uint64 index, float (*)(float, Data const*));
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
                nthNoncopyable        = libphx.CallbackTest_NthNoncopyable,
                nthNoncopyableRef     = libphx.CallbackTest_NthNoncopyableRef,
                nthNoncopyableMut     = libphx.CallbackTest_NthNoncopyableMut,
                nthNoncopyableValOpt  = libphx.CallbackTest_NthNoncopyableValOpt,
                nthNoncopyableRefOpt  = libphx.CallbackTest_NthNoncopyableRefOpt,
                nthNoncopyableMutOpt  = libphx.CallbackTest_NthNoncopyableMutOpt,
                insertNoncopyable     = libphx.CallbackTest_InsertNoncopyable,
                readPrimitiveArray    = libphx.CallbackTest_ReadPrimitiveArray,
                lockPrimitiveArray    = libphx.CallbackTest_LockPrimitiveArray,
                readNoncopyableArray  = libphx.CallbackTest_ReadNoncopyableArray,
                lockNoncopyableArray  = libphx.CallbackTest_LockNoncopyableArray,
                getMultipleAndReplace = libphx.CallbackTest_GetMultipleAndReplace,
            },
        }

        if onDef_CallbackTest_t then onDef_CallbackTest_t(t, mt) end
        CallbackTest_t = ffi.metatype(t, mt)
    end

    return CallbackTest
end

return Loader
