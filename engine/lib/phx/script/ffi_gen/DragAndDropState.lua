-- DragAndDropState ------------------------------------------------------------
local ffi = require('ffi')
local libphx = require('libphx').lib
local DragAndDropState

function declareType()
    ffi.cdef [[
        typedef struct DragAndDropState {} DragAndDropState;
    ]]

    return 1, 'DragAndDropState'
end

do -- C Definitions
    ffi.cdef [[
        cstr DragAndDropState_GetDroppedFile         (DragAndDropState const*);
        cstr DragAndDropState_GetHoveredFile         (DragAndDropState const*);
        bool DragAndDropState_IfHoveredFileCancelled (DragAndDropState const*);
    ]]
end

do -- Global Symbol Table
    DragAndDropState = {
        GetDroppedFile         = libphx.DragAndDropState_GetDroppedFile,
        GetHoveredFile         = libphx.DragAndDropState_GetHoveredFile,
        IfHoveredFileCancelled = libphx.DragAndDropState_IfHoveredFileCancelled,
    }

    if onDef_DragAndDropState then onDef_DragAndDropState(DragAndDropState, mt) end
    DragAndDropState = setmetatable(DragAndDropState, mt)
end

do -- Metatype for class instances
    local t  = ffi.typeof('DragAndDropState')
    local mt = {
        __index = {
            getDroppedFile         = libphx.DragAndDropState_GetDroppedFile,
            getHoveredFile         = libphx.DragAndDropState_GetHoveredFile,
            ifHoveredFileCancelled = libphx.DragAndDropState_IfHoveredFileCancelled,
        },
    }

    if onDef_DragAndDropState_t then onDef_DragAndDropState_t(t, mt) end
    DragAndDropState_t = ffi.metatype(t, mt)
end

return DragAndDropState
