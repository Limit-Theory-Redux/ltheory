-- ImplTest --------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef struct ImplTest {} ImplTest;
    ]]

    return 1, 'ImplTest'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local ImplTest

    do -- C Definitions
        ffi.cdef [[
            void                ImplTest_Free                   (ImplTest*);
            void                ImplTest_Func1                  (ImplTest const*);
            void                ImplTest_Func2                  (ImplTest*);
            void                ImplTest_PrivateFunc1           (ImplTest const*);
            void                ImplTest_FUNC3                  ();
            void                ImplTest_SetU32                 (ImplTest*, uint32 val);
            uint32              ImplTest_GetU32                 (ImplTest const*);
            void                ImplTest_SetF32                 (ImplTest*, float val);
            float               ImplTest_GetF32                 (ImplTest const*);
            void                ImplTest_SetStr                 (ImplTest*, cstr val);
            cstr                ImplTest_GetStr                 (ImplTest const*);
            void                ImplTest_SetData                (ImplTest*, Data const* val);
            void                ImplTest_TakeData               (ImplTest*, Data* val);
            void                ImplTest_TakeBoxedData          (ImplTest*, Data* val);
            Data*               ImplTest_GetData                (ImplTest const*);
            void                ImplTest_GetDataViaOutParam     (ImplTest const*, Data* out);
            Data const*         ImplTest_GetDataRef             (ImplTest const*);
            Data*               ImplTest_GetBoxedData           (ImplTest const*);
            Data*               ImplTest_GetDataMut             (ImplTest*);
            void                ImplTest_SetOpt                 (ImplTest*, uint32 const* val);
            uint32 const*       ImplTest_GetOptU32              (ImplTest const*);
            Data const*         ImplTest_GetOptData             (ImplTest const*);
            void                ImplTest_SetOptRef              (ImplTest*, uint32 const* val);
            void                ImplTest_SetOptMut              (ImplTest*, uint32* val);
            uint8               ImplTest_RetResVal              ();
            uint8               ImplTest_RetResErr              ();
            uint8 const*        ImplTest_RetResOptVal           ();
            void                ImplTest_SetCopyable            (ImplTest*, CopyableData c);
            void                ImplTest_SetCopyableByRef       (ImplTest*, CopyableData const* c);
            void                ImplTest_SetCopyableByMutRef    (ImplTest*, CopyableData* c);
            CopyableData        ImplTest_GetCopyable            (ImplTest const*);
            void                ImplTest_GetCopyableViaOutParam (ImplTest const*, CopyableData* out);
            CopyableData        ImplTest_GetBoxedCopyable       (ImplTest const*);
            CopyableData const* ImplTest_GetOptCopyable         (ImplTest const*);
            void                ImplTest_SetOptStr              (ImplTest*, cstr val);
            void                ImplTest_SetOptString           (ImplTest*, cstr val);
            void                ImplTest_SetOptStringRef        (ImplTest*, cstr val);
            cstr                ImplTest_GetOptStr              (ImplTest const*);
            cstr                ImplTest_GetOptString           (ImplTest const*);
            cstr                ImplTest_GetOptStringRef        (ImplTest const*);
        ]]
    end

    do -- Global Symbol Table
        ImplTest = {
            FUNC3                  = libphx.ImplTest_FUNC3,
            RetResVal              = libphx.ImplTest_RetResVal,
            RetResErr              = libphx.ImplTest_RetResErr,
            RetResOptVal           = libphx.ImplTest_RetResOptVal,
        }

        if onDef_ImplTest then onDef_ImplTest(ImplTest, mt) end
        ImplTest = setmetatable(ImplTest, mt)
    end

    do -- Metatype for class instances
        local t  = ffi.typeof('ImplTest')
        local mt = {
            __index = {
                func1                  = libphx.ImplTest_Func1,
                func2                  = libphx.ImplTest_Func2,
                privateFunc1           = libphx.ImplTest_PrivateFunc1,
                setU32                 = libphx.ImplTest_SetU32,
                getU32                 = libphx.ImplTest_GetU32,
                setF32                 = libphx.ImplTest_SetF32,
                getF32                 = libphx.ImplTest_GetF32,
                setStr                 = libphx.ImplTest_SetStr,
                getStr                 = libphx.ImplTest_GetStr,
                setData                = libphx.ImplTest_SetData,
                takeData               = libphx.ImplTest_TakeData,
                takeBoxedData          = libphx.ImplTest_TakeBoxedData,
                getData                = function(...)
                    local instance = libphx.ImplTest_GetData(...)
                    return Core.ManagedObject(instance, libphx.Data_Free)
                end,
                getDataViaOutParam     = libphx.ImplTest_GetDataViaOutParam,
                getDataRef             = libphx.ImplTest_GetDataRef,
                getBoxedData           = function(...)
                    local instance = libphx.ImplTest_GetBoxedData(...)
                    return Core.ManagedObject(instance, libphx.Data_Free)
                end,
                getDataMut             = libphx.ImplTest_GetDataMut,
                setOpt                 = libphx.ImplTest_SetOpt,
                getOptU32              = libphx.ImplTest_GetOptU32,
                getOptData             = function(...)
                    local instance = libphx.ImplTest_GetOptData(...)
                    return Core.ManagedObject(instance, libphx.Data_Free)
                end,
                setOptRef              = libphx.ImplTest_SetOptRef,
                setOptMut              = libphx.ImplTest_SetOptMut,
                setCopyable            = libphx.ImplTest_SetCopyable,
                setCopyableByRef       = libphx.ImplTest_SetCopyableByRef,
                setCopyableByMutRef    = libphx.ImplTest_SetCopyableByMutRef,
                getCopyable            = libphx.ImplTest_GetCopyable,
                getCopyableViaOutParam = libphx.ImplTest_GetCopyableViaOutParam,
                getBoxedCopyable       = libphx.ImplTest_GetBoxedCopyable,
                getOptCopyable         = libphx.ImplTest_GetOptCopyable,
                setOptStr              = libphx.ImplTest_SetOptStr,
                setOptString           = libphx.ImplTest_SetOptString,
                setOptStringRef        = libphx.ImplTest_SetOptStringRef,
                getOptStr              = libphx.ImplTest_GetOptStr,
                getOptString           = libphx.ImplTest_GetOptString,
                getOptStringRef        = libphx.ImplTest_GetOptStringRef,
            },
        }

        if onDef_ImplTest_t then onDef_ImplTest_t(t, mt) end
        ImplTest_t = ffi.metatype(t, mt)
    end

    return ImplTest
end

return Loader
