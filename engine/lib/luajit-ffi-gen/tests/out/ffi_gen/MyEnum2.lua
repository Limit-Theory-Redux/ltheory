-- AUTO GENERATED. DO NOT MODIFY!
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
            cstr    MyEnum2_ToString(MyEnum2);
        ]]
    end

    do -- Global Symbol Table
        MyEnum2 = {
            Var1     = 1,
            Var2     = 3,

            ToString = libphx.MyEnum2_ToString,
        }

        if onDef_MyEnum2 then onDef_MyEnum2(MyEnum2, mt) end
        MyEnum2 = setmetatable(MyEnum2, mt)
    end

    return MyEnum2
end

return Loader
