-- OptionTest ------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef struct OptionTest {} OptionTest;
    ]]

    return 1, 'OptionTest'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local OptionTest

    do -- C Definitions
        ffi.cdef [[
            void                OptionTest_Free            (OptionTest*);
            void                OptionTest_SetPrimitive    (OptionTest*, uint32 const* val);
            void                OptionTest_SetPrimitiveRef (OptionTest*, uint32 const* val);
            void                OptionTest_SetPrimitiveMut (OptionTest*, uint32* val);
            uint32 const*       OptionTest_GetPrimitive    (OptionTest const*);
            uint32 const*       OptionTest_GetPrimitiveRef (OptionTest const*);
            uint32*             OptionTest_GetPrimitiveMut (OptionTest*);
            void                OptionTest_SetManaged      (OptionTest*, ManagedData const* val);
            void                OptionTest_SetManagedRef   (OptionTest*, ManagedData const* val);
            void                OptionTest_SetManagedMut   (OptionTest*, ManagedData* val);
            ManagedData const*  OptionTest_GetManaged      (OptionTest const*);
            ManagedData const*  OptionTest_GetManagedRef   (OptionTest const*);
            ManagedData*        OptionTest_GetManagedMut   (OptionTest*);
            void                OptionTest_SetCopyable     (OptionTest*, CopyableData const* val);
            void                OptionTest_SetCopyableRef  (OptionTest*, CopyableData const* val);
            void                OptionTest_SetCopyableMut  (OptionTest*, CopyableData* val);
            CopyableData const* OptionTest_GetCopyable     (OptionTest const*);
            CopyableData const* OptionTest_GetCopyableRef  (OptionTest const*);
            CopyableData*       OptionTest_GetCopyableMut  (OptionTest*);
            void                OptionTest_SetStr          (OptionTest*, cstr val);
            void                OptionTest_SetString       (OptionTest*, cstr val);
            void                OptionTest_SetStringRef    (OptionTest*, cstr val);
            cstr                OptionTest_GetStr          (OptionTest const*);
            cstr                OptionTest_GetString       (OptionTest const*);
            cstr                OptionTest_GetStringRef    (OptionTest const*);
        ]]
    end

    do -- Global Symbol Table
        OptionTest = {}

        if onDef_OptionTest then onDef_OptionTest(OptionTest, mt) end
        OptionTest = setmetatable(OptionTest, mt)
    end

    do -- Metatype for class instances
        local t  = ffi.typeof('OptionTest')
        local mt = {
            __index = {
                setPrimitive    = libphx.OptionTest_SetPrimitive,
                setPrimitiveRef = libphx.OptionTest_SetPrimitiveRef,
                setPrimitiveMut = libphx.OptionTest_SetPrimitiveMut,
                getPrimitive    = libphx.OptionTest_GetPrimitive,
                getPrimitiveRef = libphx.OptionTest_GetPrimitiveRef,
                getPrimitiveMut = libphx.OptionTest_GetPrimitiveMut,
                setManaged      = libphx.OptionTest_SetManaged,
                setManagedRef   = libphx.OptionTest_SetManagedRef,
                setManagedMut   = libphx.OptionTest_SetManagedMut,
                getManaged      = function(...)
                    local instance = libphx.OptionTest_GetManaged(...)
                    return Core.ManagedObject(instance, libphx.ManagedData_Free)
                end,
                getManagedRef   = libphx.OptionTest_GetManagedRef,
                getManagedMut   = libphx.OptionTest_GetManagedMut,
                setCopyable     = libphx.OptionTest_SetCopyable,
                setCopyableRef  = libphx.OptionTest_SetCopyableRef,
                setCopyableMut  = libphx.OptionTest_SetCopyableMut,
                getCopyable     = libphx.OptionTest_GetCopyable,
                getCopyableRef  = libphx.OptionTest_GetCopyableRef,
                getCopyableMut  = libphx.OptionTest_GetCopyableMut,
                setStr          = libphx.OptionTest_SetStr,
                setString       = libphx.OptionTest_SetString,
                setStringRef    = libphx.OptionTest_SetStringRef,
                getStr          = libphx.OptionTest_GetStr,
                getString       = libphx.OptionTest_GetString,
                getStringRef    = libphx.OptionTest_GetStringRef,
            },
        }

        if onDef_OptionTest_t then onDef_OptionTest_t(t, mt) end
        OptionTest_t = ffi.metatype(t, mt)
    end

    return OptionTest
end

return Loader
