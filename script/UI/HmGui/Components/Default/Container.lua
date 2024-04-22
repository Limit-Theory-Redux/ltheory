local Container = {}
Container.__index = Container

local meta = {
    __call = function(cls, ...)
        return cls:new(...)
    end
}

---@class UIComponent
---@field render function|nil

---@class UIComponentContainer: UIComponent
---@field align table
---@field padding table
---@field stackDirection number
---@field contents table

---@class UIComponentContainerConstructor
---@field align table
---@field padding table
---@field stackDirection number
---@field contents table

---returns a button object
---@param args UIComponentContainerConstructor
---@return UIComponentContainer|nil
function Container:new(args)
    if not args then
        return
    end

    local newContainer = {}
    newContainer.align = args.align
    newContainer.padding = args.padding
    newContainer.stackDirection = args.stackDirection
    newContainer.contents = args.contents

    return newContainer
end

setmetatable(Container, meta)

-- Add to global UIComponent table
---@type UIComponentContainerConstructor
UIComponent.Container = Container

return Container
