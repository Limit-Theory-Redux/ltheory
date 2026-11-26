local Component = require("Core.ECS.Component")

---@class ProductionLine
---@field recipe RecipeDefinition
---@field running boolean Whether a cycle has taken its inputs and is under way
---@field progress number Simulated seconds of the running cycle
---@field cycles integer Cycles finished

---The production lines of a station: each runs one recipe, a cycle at a time, from the inputs in the
---entity's inventory into it. `ProductionSystem` runs them.
---@class ProductionComponent: Component
---@field lines ProductionLine[]
---@overload fun(self: ProductionComponent, recipes: RecipeDefinition[]|nil): ProductionComponent subclass internal
---@overload fun(recipes: RecipeDefinition[]|nil): ProductionComponent subclass external
local ProductionComponent = Subclass("ProductionComponent", Component, function(self, recipes)
    self:setComponentName("EconomyProduction")

    self.lines = {}
    for _, recipe in ipairs(recipes or {}) do self:addRecipe(recipe) end
end)

---@param recipe RecipeDefinition
function ProductionComponent:addRecipe(recipe)
    self.lines[#self.lines + 1] = { recipe = recipe, running = false, progress = 0, cycles = 0 }
end

---@return ProductionLine[]
function ProductionComponent:getLines()
    return self.lines
end

---Every unit of each item the lines consume and make in one cycle each, together.
---@return table<integer, integer> inputs, table<integer, integer> outputs Item id -> count
function ProductionComponent:getFlows()
    local inputs, outputs = {}, {}
    for _, line in ipairs(self.lines) do
        for _, stack in ipairs(line.recipe.inputs) do
            inputs[stack.item.id] = (inputs[stack.item.id] or 0) + stack.count
        end
        for _, stack in ipairs(line.recipe.outputs) do
            outputs[stack.item.id] = (outputs[stack.item.id] or 0) + stack.count
        end
    end
    return inputs, outputs
end

return ProductionComponent
