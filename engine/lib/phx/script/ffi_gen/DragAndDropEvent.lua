-- AUTO GENERATED. DO NOT MODIFY!
-- DragAndDropEvent ------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef uint8 DragAndDropEvent;
    ]]

    return 2, 'DragAndDropEvent'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local DragAndDropEvent

    do -- C Definitions
        ffi.cdef [[
            cstr             DragAndDropEvent_ToString(DragAndDropEvent);
        ]]
    end

    do -- Global Symbol Table
        DragAndDropEvent = {
            DroppedFile          = 0,
            HoveredFile          = 1,
            HoveredFileCancelled = 2,

            ToString             = libphx.DragAndDropEvent_ToString,
        }

        if onDef_DragAndDropEvent then onDef_DragAndDropEvent(DragAndDropEvent, mt) end
        DragAndDropEvent = setmetatable(DragAndDropEvent, mt)
    end

    return DragAndDropEvent
end

return Loader
