--[[----------------------------------------------------------------------------
  Convenience functions to eliminate tedious OOP idioms in Lua.

  Constructors are optional. Subclassing provides inheritance, virtualism, and
  chained constructors. Subclass is implemented with chained metatables.

  Example:

    local Ship = Class(function (self, name, hp)
      self.name = name
      self.health = hp
      self.healthMax = hp
    end)

    function Ship:addHangar (unit) assert(false, "No hangar!") end
    function Ship:getHangar () return {} end

    local Carrier = Subclass(Ship, function (self)
      self.hanger = {}
    end)

    function Carrier:addHangar (unit) insert(self.hangar, unit) end
    function Carrier:getHangar () return self.hangar end
----------------------------------------------------------------------------]]
--

function Class(ctor)
    local cls = {}
    cls.__index = cls
    setmetatable(cls, {
        __call = function(T, ...)
            local self = {}
            setmetatable(self, cls)
            if ctor then ctor(T, ...) end
            return self
        end
    })
    return cls
end

function Subclass(base, ctor)
    local cls = {}
    cls.__index = cls
    setmetatable(cls, {
        __call = function(T, ...)
            local self = base()
            setmetatable(self, cls)
            if ctor then ctor(T, ...) end
            return self
        end,
        __index = base,
    })
    return cls
end
