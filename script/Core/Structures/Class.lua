--[[----------------------------------------------------------------------------
  Convenience functions to eliminate tedious OOP idioms in Lua.

  Constructors are optional. Subclassing provides inheritance, virtualism, and
  chained constructors. Subclass is implemented with chained metatables.

  Example:

    local Ship = Class("Ship", function (self, name, hp)
      self.name = name
      self.health = hp
      self.healthMax = hp
    end)

    function Ship:addHangar (unit) assert(false, "No hangar!") end
    function Ship:getHangar () return {} end

    local Carrier = Subclass("Carrier", Ship, function (self)
      self.hanger = {}
    end)

    function Carrier:addHangar (unit) insert(self.hangar, unit) end
    function Carrier:getHangar () return self.hangar end
----------------------------------------------------------------------------]]
--

-- This function takes the class instance `self` and generates a string representation of it in
-- the form: ClassName{field1: 0, field2: 0}.
local function defaulttostring(self)
    local result = {}
    for key, value in pairs(self) do
        local keyStr = tostring(key)
        local valueStr = tostring(value)
        table.insert(result, keyStr .. ": " .. valueStr)
    end
    return tostring(type(self)) .. "{" .. table.concat(result, ", ") .. "}"
end

function Class(name, ctor)
    -- Define the class.
    local cls = {}
    cls.__index = cls
    cls.__type = cls
    cls.__tostring = defaulttostring

    -- Define the default constructor
    -- This just invokes ctor if it is not nil.
    function cls.new(...)
        local self = {}
        setmetatable(self, cls)
        if ctor then ctor(self, ...) end
        return self
    end

    -- Set up the class metatable.
    setmetatable(cls, {
        __call = function(T, ...)
            return cls.new(...)
        end,
        __tostring = function()
            return name
        end
    })

    return cls
end

function Subclass(name, base, ctor)
    -- Define the class.
    local cls = {}
    cls.__index = cls
    cls.__type = cls
    cls.__tostring = base.__tostring or defaulttostring

    -- Define the default constructor.
    -- This just invokes ctor on an instance of base() if it is not nil.
    function cls.new(...)
        local self = base()
        setmetatable(self, cls)
        if ctor then ctor(self, ...) end
        return self
    end

    -- Set up the class metatable.
    setmetatable(cls, {
        __call = function(T, ...)
            return cls.new(...)
        end,
        __tostring = function()
            return name
        end,
        __index = base,
    })
    return cls
end
