-- ResultTest ------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    return 0, 'ResultTest'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local ResultTest

    do -- C Definitions
        ffi.cdef [[
            uint8        ResultTest_ResultPrimitive ();
            uint8        ResultTest_ResultErr       ();
            uint8 const* ResultTest_ResultOption    ();
            cstr         ResultTest_ResultString    ();
        ]]
    end

    do -- Global Symbol Table
        ResultTest = {
            ResultPrimitive = libphx.ResultTest_ResultPrimitive,
            ResultErr       = libphx.ResultTest_ResultErr,
            ResultOption    = libphx.ResultTest_ResultOption,
            ResultString    = libphx.ResultTest_ResultString,
        }

        if onDef_ResultTest then onDef_ResultTest(ResultTest, mt) end
        ResultTest = setmetatable(ResultTest, mt)
    end

    return ResultTest
end

return Loader
