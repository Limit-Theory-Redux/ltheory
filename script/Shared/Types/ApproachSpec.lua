---Constructor and validation for approach specs.
---An approach spec says where a ship stops at a target: `StandOff`, at a distance from the target's
---centre (target radii, or a gap in metres between the hulls), or `Point`, at a point fixed in the
---target's own axes. Registers with ApproachSpecRegistry when it has an id;
---Shared/Definitions/ApproachSpecDefs.lua is the data source. A spec with no id is a plain value,
---for an order that carries its own.

local ApproachSpecRegistry = require("Shared.Registries.ApproachSpecRegistry")
local ScaleConfig          = require("Config.Gen.UniverseScaleConfig")

local Kind = Enums.ApproachKind

---@class ApproachOffset
---@field right number
---@field up number
---@field forward number

---@class ApproachSpecConstructor
---@field id ApproachSpecId|nil Registers the spec when given
---@field name string|nil Defaults to "inline"
---@field kind ApproachKindId
---@field radii number|nil StandOff: the distance from the target's centre, in target radii, above zero
---@field gapM number|nil StandOff: the distance between the hulls, in metres, above zero; instead of `radii`
---@field offsetRadii ApproachOffset|nil Point: where the point is, in target radii along the target's own axes

---@class ApproachSpec
---@field id ApproachSpecId|nil
---@field name string
---@field kind ApproachKindId
---@field private _radii number|nil
---@field private _gapM number|nil
---@field private _offsetRadii ApproachOffset|nil
---@overload fun(args: ApproachSpecConstructor): ApproachSpec
local ApproachSpec = Class("ApproachSpec")

---@param value any
---@param what string
local function assertPositive(value, what)
    assert(type(value) == "number" and value > 0 and value < math.huge,
        string.format("ApproachSpec: %s must be a finite number above zero", what))
end

---@param offset any
---@return ApproachOffset
local function checkedOffset(offset)
    assert(rawtype(offset) == "table", "ApproachSpec: a Point needs offsetRadii { right, up, forward }")
    for _, axis in ipairs({ "right", "up", "forward" }) do
        local value = offset[axis]
        assert(type(value) == "number" and value == value and math.abs(value) < math.huge,
            string.format("ApproachSpec: offsetRadii.%s must be a finite number", axis))
    end
    return { right = offset.right, up = offset.up, forward = offset.forward }
end

function ApproachSpec.new(args)
    assert(rawtype(args) == "table", "ApproachSpec: expects a table of fields")
    assert(args.id == nil or (type(args.id) == "number" and args.id > 0), "ApproachSpec: the id is a positive number")
    local name = args.name or "inline"
    assert(type(name) == "string" and #name > 0, "ApproachSpec: the name is a non-empty string")
    if args.id and ApproachSpecRegistry:has(args.id) then
        Log.Warn("Attempting to recreate ApproachSpec: " .. tostring(args.id))
        return ApproachSpecRegistry:get(args.id)
    end

    local definition = setmetatable({ id = args.id, name = name, kind = args.kind }, ApproachSpec)
    if args.kind == Kind.StandOff then
        assert((args.radii == nil) ~= (args.gapM == nil),
            "ApproachSpec: a StandOff has either radii or gapM")
        assert(args.offsetRadii == nil, "ApproachSpec: a StandOff has no offsetRadii")
        if args.radii then assertPositive(args.radii, "radii") end
        if args.gapM then assertPositive(args.gapM, "gapM") end
        definition._radii = args.radii
        definition._gapM = args.gapM
    elseif args.kind == Kind.Point then
        assert(args.radii == nil and args.gapM == nil, "ApproachSpec: a Point has only offsetRadii")
        definition._offsetRadii = checkedOffset(args.offsetRadii)
    else
        error("ApproachSpec: the kind is Enums.ApproachKind.StandOff or Point, got " .. tostring(args.kind))
    end
    if args.id then ApproachSpecRegistry:new(args.id, definition) end
    return definition
end

---Turns what an order names into a spec: a registered id, a table of fields (an inline spec), or a spec.
---@param value ApproachSpecId|ApproachSpecConstructor|ApproachSpec
---@return ApproachSpec
function ApproachSpec.Of(value)
    if type(value) == "number" then return ApproachSpecRegistry:get(value) end
    assert(rawtype(value) == "table", "ApproachSpec.Of: an Enums.ApproachSpec id, a table of fields or a spec")
    if getmetatable(value) == ApproachSpec then return value end
    assert(value.id == nil, "ApproachSpec.Of: an inline spec has no id (register it in ApproachSpecDefs)")
    return ApproachSpec.new(value)
end

---@return boolean
function ApproachSpec:isPoint()
    return self.kind == Kind.Point
end

---@return number|nil The StandOff's distance from the target's centre in target radii; nil for a gap or a Point
function ApproachSpec:getRadii() return self._radii end

---@return gameunit|nil The StandOff's distance between the hulls; nil for radii or a Point
function ApproachSpec:getGap()
    return self._gapM and ScaleConfig:toGameUnits(self._gapM, "starSystem") or nil
end

---The Point's place in the world: the offset in target radii, along the target's own axes.
---@param targetPos Position
---@param targetRadius gameunit
---@param rotation Quat The target's orientation
---@return Position
function ApproachSpec:pointOf(targetPos, targetRadius, rotation)
    assert(self:isPoint(), "ApproachSpec:pointOf: not a Point")
    local offset = self._offsetRadii
    local right, up, forward = rotation:getRight(), rotation:getUp(), rotation:getForward()
    local r, u, f = offset.right * targetRadius, offset.up * targetRadius, offset.forward * targetRadius
    return Position(
        targetPos.x + right.x * r + up.x * u + forward.x * f,
        targetPos.y + right.y * r + up.y * u + forward.y * f,
        targetPos.z + right.z * r + up.z * u + forward.z * f)
end

return ApproachSpec
