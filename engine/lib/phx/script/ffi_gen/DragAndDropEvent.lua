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
            DragAndDropEvent DragAndDropEvent_DroppedFile;
            DragAndDropEvent DragAndDropEvent_HoveredFile;
            DragAndDropEvent DragAndDropEvent_HoveredFileCancelled;

            cstr             DragAndDropEvent_ToString(DragAndDropEvent);
        ]]
    end

    do -- Global Symbol Table
        DragAndDropEvent = {
            DroppedFile          = libphx.DragAndDropEvent_DroppedFile,
            HoveredFile          = libphx.DragAndDropEvent_HoveredFile,
            HoveredFileCancelled = libphx.DragAndDropEvent_HoveredFileCancelled,

            ToString             = libphx.DragAndDropEvent_ToString,
        }

        if onDef_DragAndDropEvent then onDef_DragAndDropEvent(DragAndDropEvent, mt) end
        DragAndDropEvent = setmetatable(DragAndDropEvent, mt)
    end

    return DragAndDropEvent
end

return Loader
