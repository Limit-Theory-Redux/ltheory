-- AUTO GENERATED. DO NOT MODIFY!
-- MyEnum4 ---------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef uint16 MyEnum4;
    ]]

    return 2, 'MyEnum4'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local MyEnum4

    do -- C Definitions
        ffi.cdef [[
            MyEnum4 MyEnum4_Var1;
            MyEnum4 MyEnum4_Var2;
            MyEnum4 MyEnum4_Var3;

            cstr    MyEnum4_ToString(MyEnum4);
        ]]
    end

    do -- Global Symbol Table
        MyEnum4 = {
            Var1     = libphx.MyEnum4_Var1,
            Var2     = libphx.MyEnum4_Var2,
            Var3     = 11,

            ToString = libphx.MyEnum4_ToString,
        }

        if onDef_MyEnum4 then onDef_MyEnum4(MyEnum4, mt) end
        MyEnum4 = setmetatable(MyEnum4, mt)
    end

    return MyEnum4
end

return Loader
