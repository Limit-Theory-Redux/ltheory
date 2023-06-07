local CArray     = require('Core.CFFI.CArray')
local CPointer   = require('Core.CFFI.CPointer')
local CReference = require('Core.CFFI.CReference')
local CStruct    = require('Core.CFFI.CStruct')
local Type       = require('Core.CFFI.Type')

local CType      = {}

function CType.Array(T)
    return CArray(T)
end

function CType.Pointer(T)
    return CPointer(T)
end

function CType.Reference(T)
    return CReference(T)
end

function CType.Struct(name)
    local self = CStruct(name)
    CType[name] = self
    return self
end

function CType.Subclass(name, parent)
    local self = CStruct(name)
    self.parent = parent
    for i = 1, #parent.fields do
        self:add(parent.fields[i].T, parent.fields[i].name)
    end
    CType[name] = self
    for i = 1, #parent.onSubclass do
        parent.onSubclass[i](parent, self)
    end
    return self
end

CType.MultiArray   = require('Core.CFFI.CMultiArray')
CType.MultiPointer = require('Core.CFFI.CMultiPointer')

return CType
