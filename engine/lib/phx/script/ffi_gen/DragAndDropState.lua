-- DragAndDropState ------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef struct DragAndDropState {} DragAndDropState;
    ]]

    return 1, 'DragAndDropState'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local DragAndDropState

    do -- C Definitions
        ffi.cdef [[
            void DragAndDropState_Free                   (DragAndDropState*);
            cstr DragAndDropState_GetDroppedFile         (DragAndDropState const*);
            cstr DragAndDropState_GetHoveredFile         (DragAndDropState const*);
            bool DragAndDropState_IfHoveredFileCancelled (DragAndDropState const*);
        ]]
    end

    do -- Global Symbol Table
        DragAndDropState = {}

        if onDef_DragAndDropState then onDef_DragAndDropState(DragAndDropState, mt) end
        DragAndDropState = setmetatable(DragAndDropState, mt)
    end

    do -- Metatype for class instances
        local t  = ffi.typeof('DragAndDropState')
        local mt = {
            __index = {
                ---@return cstr
                getDroppedFile         = libphx.DragAndDropState_GetDroppedFile,
                ---@return cstr
                getHoveredFile         = libphx.DragAndDropState_GetHoveredFile,
                ---@return bool
                ifHoveredFileCancelled = libphx.DragAndDropState_IfHoveredFileCancelled,
            },
        }

        if onDef_DragAndDropState_t then onDef_DragAndDropState_t(t, mt) end
        DragAndDropState_t = ffi.metatype(t, mt)
    end

    return DragAndDropState
end

return Loader
