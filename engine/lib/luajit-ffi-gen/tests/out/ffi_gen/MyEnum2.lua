-- MyEnum2 ---------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef uint32 MyEnum2;
    ]]

    return 2, 'MyEnum2'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local MyEnum2

    do -- C Definitions
        ffi.cdef [[
            MyEnum2 MyEnum2_Var1;
            MyEnum2 MyEnum2_Var2;

            cstr    MyEnum2_ToString(MyEnum2);
        ]]
    end

    do -- Global Symbol Table
        MyEnum2 = {
            Var1     = libphx.MyEnum2_Var1,
            Var2     = libphx.MyEnum2_Var2,

            ToString = libphx.MyEnum2_ToString,
        }

        if onDef_MyEnum2 then onDef_MyEnum2(MyEnum2, mt) end
        MyEnum2 = setmetatable(MyEnum2, mt)
    end

    return MyEnum2
end

return Loader
