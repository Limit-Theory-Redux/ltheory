-- AUTO GENERATED. DO NOT MODIFY!
-- PlainTest -------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef struct PlainTest {} PlainTest;
    ]]

    return 1, 'PlainTest'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local PlainTest

    do -- C Definitions
        ffi.cdef [[
            void                PlainTest_Free                   (PlainTest*);
            void                PlainTest_Func1                  (PlainTest const*);
            void                PlainTest_Func2                  (PlainTest*);
            void                PlainTest_PrivateFunc1           (PlainTest const*);
            void                PlainTest_FUNC3                  ();
            void                PlainTest_SetU32                 (PlainTest*, uint32 val);
            uint32              PlainTest_GetU32                 (PlainTest const*);
            void                PlainTest_SetF32Ref              (PlainTest*, float const* val);
            float               PlainTest_GetF32                 (PlainTest const*);
            uint32 const*       PlainTest_GetU32Ref              (PlainTest const*);
            uint32*             PlainTest_GetU32Mut              (PlainTest*);
            void                PlainTest_SetManaged             (PlainTest*, ManagedData* val);
            void                PlainTest_SetManagedRef          (PlainTest*, ManagedData const* val);
            void                PlainTest_SetManagedMut          (PlainTest*, ManagedData* val);
            ManagedData*        PlainTest_GetManaged             (PlainTest const*);
            void                PlainTest_GetManagedViaOutParam  (PlainTest const*, ManagedData* out);
            ManagedData const*  PlainTest_GetManagedRef          (PlainTest const*);
            ManagedData*        PlainTest_GetManagedMut          (PlainTest*);
            void                PlainTest_SetCopyable            (PlainTest*, CopyableData val);
            void                PlainTest_SetCopyableRef         (PlainTest*, CopyableData const* val);
            void                PlainTest_SetCopyableMut         (PlainTest*, CopyableData* val);
            CopyableData        PlainTest_GetCopyable            (PlainTest const*);
            void                PlainTest_GetCopyableViaOutParam (PlainTest const*, CopyableData* out);
            CopyableData const* PlainTest_GetCopyableRef         (PlainTest const*);
            CopyableData*       PlainTest_GetCopyableMut         (PlainTest*);
            void                PlainTest_SetStr                 (PlainTest*, cstr val);
            void                PlainTest_SetString              (PlainTest*, cstr val);
            void                PlainTest_SetStringRef           (PlainTest*, cstr val);
            cstr                PlainTest_GetStr                 (PlainTest const*);
            cstr                PlainTest_GetString              (PlainTest const*);
            cstr                PlainTest_GetStringRef           (PlainTest const*);
        ]]
    end

    do -- Global Symbol Table
        PlainTest = {
            FUNC3                  = libphx.PlainTest_FUNC3,
        }

        if onDef_PlainTest then onDef_PlainTest(PlainTest, mt) end
        PlainTest = setmetatable(PlainTest, mt)
    end

    do -- Metatype for class instances
        local t  = ffi.typeof('PlainTest')
        local mt = {
            __index = {
                func1                  = libphx.PlainTest_Func1,
                func2                  = libphx.PlainTest_Func2,
                privateFunc1           = libphx.PlainTest_PrivateFunc1,
                setU32                 = libphx.PlainTest_SetU32,
                getU32                 = libphx.PlainTest_GetU32,
                setF32Ref              = libphx.PlainTest_SetF32Ref,
                getF32                 = libphx.PlainTest_GetF32,
                getU32Ref              = libphx.PlainTest_GetU32Ref,
                getU32Mut              = libphx.PlainTest_GetU32Mut,
                setManaged             = function(self, val)
                    ffi.gc(val, nil)
                    libphx.PlainTest_SetManaged(self, val)
                end,
                setManagedRef          = libphx.PlainTest_SetManagedRef,
                setManagedMut          = libphx.PlainTest_SetManagedMut,
                getManaged             = function(self)
                    local _instance = libphx.PlainTest_GetManaged(self)
                    return Core.ManagedObject(_instance, libphx.ManagedData_Free)
                end,
                getManagedViaOutParam  = libphx.PlainTest_GetManagedViaOutParam,
                getManagedRef          = libphx.PlainTest_GetManagedRef,
                getManagedMut          = libphx.PlainTest_GetManagedMut,
                setCopyable            = libphx.PlainTest_SetCopyable,
                setCopyableRef         = libphx.PlainTest_SetCopyableRef,
                setCopyableMut         = libphx.PlainTest_SetCopyableMut,
                getCopyable            = libphx.PlainTest_GetCopyable,
                getCopyableViaOutParam = libphx.PlainTest_GetCopyableViaOutParam,
                getCopyableRef         = libphx.PlainTest_GetCopyableRef,
                getCopyableMut         = libphx.PlainTest_GetCopyableMut,
                setStr                 = libphx.PlainTest_SetStr,
                setString              = libphx.PlainTest_SetString,
                setStringRef           = libphx.PlainTest_SetStringRef,
                getStr                 = libphx.PlainTest_GetStr,
                getString              = libphx.PlainTest_GetString,
                getStringRef           = libphx.PlainTest_GetStringRef,
            },
        }

        if onDef_PlainTest_t then onDef_PlainTest_t(t, mt) end
        PlainTest_t = ffi.metatype(t, mt)
    end

    return PlainTest
end

return Loader
