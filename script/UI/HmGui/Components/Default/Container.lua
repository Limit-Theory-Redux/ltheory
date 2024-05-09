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
---@field visible boolean
---@field align table<AlignHorizontal, AlignVertical>
---@field childrenAlign table<AlignHorizontal, AlignVertical>
---@field padding table
---@field stackDirection number
---@field contents table
---@field render fun(self: UIComponentContainer)

---@class UIComponentContainerConstructor
---@field visible boolean|nil
---@field align table<AlignHorizontal, AlignVertical>
---@field childrenAlign table<AlignHorizontal, AlignVertical>
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
        visible = args.visible,
        align = args.align or { AlignHorizontal.Default, AlignVertical.Default },
        childrenAlign = args.childrenAlign or { AlignHorizontal.Default, AlignVertical.Default },
        padding = args.padding or { 0, 0 },
        margin = args.margin or { 0, 0 },
        width = args.width,
        height = args.height,
        widthInLayout = args.widthInLayout,
        heightInLayout = args.heightInLayout,
        stackDirection = args.stackDirection or Enums.UI.StackDirection.Horizontal,
        contents = args.contents,
        color = {
            background = args.color and args.color.background
        },
        showContainer = args.showContainer or function() return GameState.debug.metricsEnabled end,
        showContainerColor = Color((math.random() + math.random(50, 99)) / 100, (math.random() + math.random(50, 99)) / 100, (math.random() + math.random(50, 99)) / 100, .4)
    }

    newContainer.render = function(self)
        if not self.state.visible() then
            return
        end

        -- color
        if self.state.color().background then
            Gui:setPropertyColor(GuiProperties.BackgroundColor, self.state.color().background)
        end

        if self.state.showContainer() then
            Gui:setPropertyColor(GuiProperties.BorderColor, self.state.showContainerColor())
            Gui:beginStackContainer()
            Gui:setAlignment(AlignHorizontal.Stretch, AlignVertical.Stretch)
            Gui:setBorderWidth(1)
        else
            Gui:beginStackContainer()
            Gui:setAlignment(AlignHorizontal.Stretch, AlignVertical.Stretch)
        end

        Gui:clearStyle()

        if self.state.stackDirection() == Enums.UI.StackDirection.Horizontal then
            Gui:beginHorizontalContainer()
        elseif self.state.stackDirection() == Enums.UI.StackDirection.Vertical then
            Gui:beginVerticalContainer()
        end

        Gui:setAlignment(self.state.align()[1], self.state.align()[2])
        Gui:setChildrenAlignment(self.state.childrenAlign()[1], self.state.childrenAlign()[2])
        Gui:setPadding(self.state.padding()[1], self.state.padding()[2])
        Gui:setMargin(self.state.margin()[1], self.state.margin()[2])

        if self.state.width then
            Gui:setPercentWidth(self.state.width() * 100)
        end

        if self.state.height then
            Gui:setPercentHeight(self.state.height() * 100)
        end

        if #self.state.contents() > 1 then
            for _, component in ipairs(self.state.contents()) do
                component:render()
            end
        elseif #self.state.contents() == 1 then
            self.state.contents()[1]:render()
        end -- this allows containers without any content

        Gui:endContainer()

        Gui:endContainer()
        Gui:setPercentSize(100, 100)

        Gui:clearStyle() -- clear style so it doesn´t affect other components
    end

    return newContainer
end

setmetatable(Container, meta)

-- Add to global UIComponent table
---@type UIComponentContainerConstructor
UIComponent.Container = Container

return Container
