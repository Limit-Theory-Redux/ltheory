-- AUTO GENERATED. DO NOT MODIFY!
-- MyEnum3 ---------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef uint16 MyEnum3;
    ]]

    return 2, 'MyEnum3'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local MyEnum3

    do -- C Definitions
        ffi.cdef [[
            MyEnum3 MyEnum3_Var1;
            MyEnum3 MyEnum3_Var2;

            cstr    MyEnum3_ToString(MyEnum3);
        ]]
    end

    do -- Global Symbol Table
        MyEnum3 = {
            Var1     = libphx.MyEnum3_Var1,
            Var2     = libphx.MyEnum3_Var2,

            ToString = libphx.MyEnum3_ToString,
        }

        if onDef_MyEnum3 then onDef_MyEnum3(MyEnum3, mt) end
        MyEnum3 = setmetatable(MyEnum3, mt)
    end

    return MyEnum3
end

return Loader
