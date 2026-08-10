-- AUTO GENERATED. DO NOT MODIFY!
-- MemoryReport ----------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    return 0, 'MemoryReport'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local MemoryReport

    do -- C Definitions
        ffi.cdef [[
            void MemoryReport_BeginFrame ();
            void MemoryReport_Add        (cstr category, uint64 count, uint64 bytes);
        ]]
    end

    do -- Global Symbol Table
        MemoryReport = {
            BeginFrame = libphx.MemoryReport_BeginFrame,
            Add        = libphx.MemoryReport_Add,
        }

        if onDef_MemoryReport then onDef_MemoryReport(MemoryReport, mt) end
        MemoryReport = setmetatable(MemoryReport, mt)
    end

    return MemoryReport
end

return Loader
