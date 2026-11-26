---Constructor and validation for cost profiles.
---A cost profile says how much a ship minds each kind of zone: a weight per zone tag that
---multiplies the zone's path cost. Registers with CostProfileRegistry when it has an id;
---Shared/Definitions/CostProfileDefs.lua is the data source. A profile with no id is a plain
---value, for a ship that carries its own weights.
-- Design: doc/script/nav/system-planner.md

local CostProfileRegistry = require("Shared.Registries.CostProfileRegistry")
local Tags                = require("Shared.Registries.Tags")

---@class CostProfileConstructor
---@field id CostProfileId|nil Registers the profile when given
---@field name string
---@field weights table<string, number> Weight of each zone tag by tag name, finite and at least zero

---@class CostProfile
---@field id CostProfileId|nil
---@field name string
---@field weights table<string, number>
---@field private _byTagId table<integer, number>
---@overload fun(args: CostProfileConstructor): CostProfile
local CostProfile = Class("CostProfile")

-- The weight of a tag that the profile doesn't name, and of a zone with no tags: the zone's own
-- cost applies unchanged.
CostProfile.UNNAMED_WEIGHT = 1

function CostProfile.new(args)
    if args.id ~= nil and (type(args.id) ~= "number" or args.id <= 0) then
        Log.Warn("No valid ID set for CostProfile")
        return nil
    elseif not args.name or #args.name == 0 then
        Log.Warn("No name set for CostProfile: " .. tostring(args.id))
        return nil
    elseif args.id and CostProfileRegistry:has(args.id) then
        Log.Warn("Attempting to recreate CostProfile: " .. tostring(args.id))
        return CostProfileRegistry:get(args.id)
    end

    local byTagId = {}
    for tag, weight in pairs(args.weights or {}) do
        local tagId = Tags:getId(tag)
        assert(tagId, string.format("CostProfile %s: unknown zone tag %s", args.name, tostring(tag)))
        assert(type(weight) == "number" and weight >= 0 and weight < math.huge,
            string.format("CostProfile %s: the weight of %s must be a finite number of at least zero",
                args.name, tag))
        byTagId[tagId] = weight
    end

    local definition = setmetatable({ id = args.id, name = args.name, weights = args.weights or {} }, CostProfile)
    definition._byTagId = byTagId
    if args.id then CostProfileRegistry:new(args.id, definition) end
    return definition
end

---The factor a zone's path cost is multiplied by: each of the zone's tags weighs what the profile
---names for it, or `UNNAMED_WEIGHT`, and the zone takes the largest. A zone with no tags weighs
---`UNNAMED_WEIGHT`.
---@param tagComponent TagComponent|nil The zone's tags
---@return number
function CostProfile:weightOf(tagComponent)
    local heaviest = nil
    if tagComponent then
        for tagId = 1, Tags._count do
            if tagComponent:hasTag(tagId) then
                local weight = self._byTagId[tagId] or CostProfile.UNNAMED_WEIGHT
                if heaviest == nil or weight > heaviest then heaviest = weight end
            end
        end
    end
    return heaviest or CostProfile.UNNAMED_WEIGHT
end

return CostProfile
