local Grid = {}
Grid.__index = Grid

local meta = {
    __call = function(cls, ...)
        return cls:new(...)
    end
}

---@class UILayout
---@field visible boolean
---@field align table<AlignHorizontal, AlignVertical>
---@field padding table<number, number>
---@field margin table<number, number>
---@field stackDirection UIStackDirection
---@field contents table<UIComponent>
---@field showGrid boolean

---@class UILayoutGrid: UILayout

---@class UILayoutGridConstructor: UILayout
---@field visible boolean
---@field align table<AlignHorizontal, AlignVertical>
---@field padding table<number, number>
---@field margin table<number, number>
---@field stackDirection UIStackDirection
---@field contents table<UIComponent>
---@field showGrid boolean|nil


---returns a grid layout object
---@param args UILayoutGridConstructor
---@return UILayoutGrid|nil
function Grid:new(args)
    if not args then
        return
    end

    local newGridLayout = {}
    newGridLayout.state = UICore.ComponentState {
        visible = args.visible,
        align = args.align or { AlignHorizontal.Default, AlignVertical.Default },
        padding = args.padding or { 0, 0 },
        margin = args.margin or { 0, 0 },
        stackDirection = args.stackDirection or Enums.UI.StackDirection.Horizontal,
        contents = args.contents,
        widthInLayout = args.widthInLayout,
        heightInLayout = args.heightInLayout,
        showGrid = args.showGrid or function() return GameState.debug.metricsEnabled end,
        showGridColor = Color((math.random() + math.random(50, 99)) / 100, (math.random() + math.random(50, 99)) / 100, (math.random() + math.random(50, 99)) / 100, .4)
    }

    newGridLayout.render = function(self)
        if self.state.stackDirection() == Enums.UI.StackDirection.Horizontal then
            Gui:beginHorizontalContainer()
        elseif self.state.stackDirection() == Enums.UI.StackDirection.Vertical then
            Gui:beginVerticalContainer()
        end

        Gui:setAlignment(self.state.align()[1], self.state.align()[2])
        Gui:setPadding(self.state.padding()[1], self.state.padding()[2])
        Gui:setMargin(self.state.margin()[1], self.state.margin()[2])

        local contentCount = #self.state.contents()

        if contentCount > 1 then
            for index, container in ipairs(self.state.contents()) do
                if self.state.stackDirection() == Enums.UI.StackDirection.Horizontal then
                    if self.state.showGrid() then
                        Gui:beginStackContainer()
                        Gui:setBorderWidth(2)
                        Gui:setProperty(GuiProperties.BorderColor, self.state.showGridColor())
                        Gui:setPercentSize(100, 100)
                        Gui:setProperty(GuiProperties.TextFont, Cache.Font("Exo2", 14))
                        Gui:setProperty(GuiProperties.TextColor, self.state.showGridColor())
                        Gui:setAlignment(AlignHorizontal.Left, AlignVertical.Top)
                        Gui:text(tostring(index))
                        Gui:clearStyle()
                        container:render()
                        Gui:endContainer()
                    else
                        Gui:beginStackContainer()
                        Gui:setPercentSize(100, 100)
                        container:render()
                        Gui:endContainer()
                    end

                    if container.state.widthInLayout then
                        Gui:setPercentSize(container.state.widthInLayout() * 100, 100)
                    else
                        Gui:setPercentSize(100 / contentCount, 100) -- even distribution
                    end
                elseif self.state.stackDirection() == Enums.UI.StackDirection.Vertical then
                    if self.state.showGrid() then
                        Gui:beginStackContainer()
                        Gui:setBorderWidth(2)
                        Gui:setProperty(GuiProperties.BorderColor, self.state.showGridColor())
                        Gui:setPercentSize(100, 100)
                        Gui:setProperty(GuiProperties.TextFont, Cache.Font("Exo2", 14))
                        Gui:setProperty(GuiProperties.TextColor, self.state.showGridColor())
                        Gui:setAlignment(AlignHorizontal.Right, AlignVertical.Top)
                        Gui:text(tostring(index))
                        Gui:clearStyle()
                        container:render()
                        Gui:endContainer()
                    else
                        Gui:beginStackContainer()
                        Gui:setPercentSize(100, 100)
                        container:render()
                        Gui:endContainer()
                    end

                    if container.state.heightInLayout then
                        Gui:setPercentSize(100, container.state.heightInLayout() * 100)
                    else
                        Gui:setPercentSize(100, 100 / contentCount) -- even distribution
                    end
                end
                Gui:clearStyle()
            end
        end
        Gui:endContainer()
    end

    return newGridLayout
end

setmetatable(Grid, meta)

-- Add to global UIComponent table
---@type UILayoutGridConstructor
UILayout.Grid = Grid

return Grid
