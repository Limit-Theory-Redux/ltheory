---@class ItemDefinition
---@field id integer
---@field group string
---@field name string
---@field mass number
---@field energy number
---@overload fun(args: {name: string, mass: number, energyDensity: number}): ItemDefinition
local ItemDefinition = Class("ItemDefinition")

function ItemDefinition.new(args)
    if not args.name then
        Log.Warn("No name Set for ItemDefinition")
        return nil
    end

    local self = setmetatable({}, ItemDefinition)
    self.name = args.name
    self.mass = args.mass
    self.energy = Math.Round(math.max(0, (args.energyDensity or 1) * args.mass))
    return self
end

return ItemDefinition