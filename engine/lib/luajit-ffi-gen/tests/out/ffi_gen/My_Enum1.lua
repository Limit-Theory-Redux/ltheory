-- AUTO GENERATED. DO NOT MODIFY!
-- My_Enum1 --------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef uint8 My_Enum1;
    ]]

    return 2, 'My_Enum1'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local My_Enum1

    do -- C Definitions
        ffi.cdef [[
            My_Enum1 My_Enum1_Var1;
            My_Enum1 My_Enum1_Var2;

            cstr     My_Enum1_ToString(My_Enum1);
        ]]
    end

    do -- Global Symbol Table
        My_Enum1 = {
            Var1     = libphx.My_Enum1_Var1,
            Var2     = libphx.My_Enum1_Var2,

            ToString = libphx.My_Enum1_ToString,
        }

        if onDef_My_Enum1 then onDef_My_Enum1(My_Enum1, mt) end
        My_Enum1 = setmetatable(My_Enum1, mt)
    end

    return My_Enum1
end

return Loader
