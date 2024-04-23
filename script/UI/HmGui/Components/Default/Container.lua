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
---@field visible boolean|nil
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
    newContainer.state = UICore.ComponentState {
        visible = args.visible or true,
        align = args.align or { AlignHorizontal.Default, AlignVertical.Default },
        padding = args.padding or { 0, 0 },
        stackDirection = args.stackDirection or Enums.UI.StackDirection.Horizontal,
        contents = args.contents
    }

    newContainer.render = function(self)
        if self.state.stackDirection() == Enums.UI.StackDirection.Horizontal then
            Gui:beginHorizontalContainer()
        elseif self.state.stackDirection() == Enums.UI.StackDirection.Vertical then
            Gui:beginVerticalContainer()
        end

        Gui:setAlignment(self.state.align()[1], self.state.align()[2])
        Gui:setPadding(self.state.padding()[1], self.state.padding()[2])

        if #self.state.contents() > 1 then
            for _, component in ipairs(self.state.contents()) do
                component:render()
            end
        else
            self.state.contents()[1]:render()
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
