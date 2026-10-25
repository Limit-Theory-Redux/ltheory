-- AUTO GENERATED. DO NOT MODIFY!
-- SceneList -------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef struct SceneList {} SceneList;
    ]]

    return 1, 'SceneList'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local SceneList

    do -- C Definitions
        ffi.cdef [[
            void       SceneList_Free              (SceneList*);
            SceneList* SceneList_Create            ();
            void       SceneList_Reset             (SceneList*);
            uint32     SceneList_AddItem           (SceneList*, Renderer* r, uint32 transform, Mesh* mesh, Material* material);
            uint32     SceneList_Prepare           (SceneList*, Renderer* r, BlendMode blend, bool cull);
            void       SceneList_Emit              (SceneList*, Renderer* r);
            uint32     SceneList_GetItemCount      (SceneList const*);
            uint32     SceneList_GetTransformCount (SceneList const*);
            uint32     SceneList_GetSubmitted      (SceneList const*);
            uint32     SceneList_GetVisible        (SceneList const*);
            uint32     SceneList_GetCulled         (SceneList const*);
        ]]
    end

    do -- Global Symbol Table
        SceneList = {
            Create            = function()
                local _instance = libphx.SceneList_Create()
                return Core.ManagedObject(_instance, libphx.SceneList_Free)
            end,
        }

        if onDef_SceneList then onDef_SceneList(SceneList, mt) end
        SceneList = setmetatable(SceneList, mt)
    end

    do -- Metatype for class instances
        local t  = ffi.typeof('SceneList')
        local mt = {
            __index = {
                reset             = libphx.SceneList_Reset,
                addItem           = libphx.SceneList_AddItem,
                prepare           = libphx.SceneList_Prepare,
                emit              = libphx.SceneList_Emit,
                getItemCount      = libphx.SceneList_GetItemCount,
                getTransformCount = libphx.SceneList_GetTransformCount,
                getSubmitted      = libphx.SceneList_GetSubmitted,
                getVisible        = libphx.SceneList_GetVisible,
                getCulled         = libphx.SceneList_GetCulled,
            },
        }

        if onDef_SceneList_t then onDef_SceneList_t(t, mt) end
        SceneList_t = ffi.metatype(t, mt)
    end

    return SceneList
end

return Loader
