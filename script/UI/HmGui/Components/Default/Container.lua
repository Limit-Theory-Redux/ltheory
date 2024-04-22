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
---@field render fun(self: UIComponentContainer)

---@class UIComponentContainerConstructor
---@field align table
---@field padding table
---@field stackDirection number
---@field contents table

---returns a container object
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

    newContainer.render = function(self)
        if self.stackDirection == Enums.UI.StackDirection.Horizontal then
            Gui:beginHorizontalContainer()
        elseif self.stackDirection == Enums.UI.StackDirection.Vertical then
            Gui:beginVerticalContainer()
        end

        Gui:setAlignment(self.align[1], self.align[2])
        Gui:setPadding(self.padding[1], self.padding[2])

        if #self.contents > 1 then
            for _, component in ipairs(self.contents) do
                component:render()
            end
        else
            self.contents[1]:render()
        end

        Gui:endContainer()
    end

    return newContainer
end

setmetatable(Container, meta)

-- Add to global UIComponent table
---@type UIComponentContainerConstructor
UIComponent.Container = Container

return Container
