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
---@field layoutType GuiLayoutType
---@field contents table<UIComponent>
---@field widthInLayout number
---@field heightInLayout number
---@field showGrid boolean

---@class UILayoutGrid: UILayout

---@class UILayoutGridConstructor: UILayout
---@field visible boolean
---@field align table<AlignHorizontal, AlignVertical>
---@field padding table<number, number>
---@field margin table<number, number>
---@field layoutType GuiLayoutType
---@field contents table<UIComponent>
---@field widthInLayout number
---@field heightInLayout number
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
        layoutType = args.layoutType or GuiLayoutType.Horizontal,
        contents = args.contents,
        widthInLayout = args.widthInLayout,
        heightInLayout = args.heightInLayout,
        showGrid = args.showGrid or function() return GameState.debug.metricsEnabled end,
        showGridColor = Color((math.random() + math.random(50, 99)) / 100, (math.random() + math.random(50, 99)) / 100, (math.random() + math.random(50, 99)) / 100, .4)
    }

    newGridLayout.render = function(self)
        Gui:beginContainer(self.state.layoutType())
        Gui:setAlignment(self.state.align()[1], self.state.align()[2])
        Gui:setPadding(self.state.padding()[1], self.state.padding()[2])
        Gui:setMargin(self.state.margin()[1], self.state.margin()[2])

        local contentCount = #self.state.contents()

        if contentCount > 1 then
            for index, container in ipairs(self.state.contents()) do
                if self.state.layoutType() == GuiLayoutType.Horizontal then
                    if self.state.showGrid() then
                        Gui:beginStackContainer()
                        Gui:setAlignment(AlignHorizontal.Left, AlignVertical.Top)
                        Gui:setPercentSize(100, 100)
                        Gui:setBorderWidth(2)
                        Gui:setBorderColor(self.state.showGridColor())

                        Gui:text(tostring(index), Cache.Font("Exo2", 14), self.state.showGridColor())

                        if type(container) == "function" then
                            container = container() -- dynamic components
                        end

                        if container then
                            container:render()
                        end

                        Gui:endContainer()
                    else
                        Gui:beginStackContainer()
                        Gui:setPercentSize(100, 100)

                        if type(container) == "function" then
                            container = container() -- dynamic components
                        end

                        if container then
                            container:render()
                        end

                        Gui:endContainer()
                    end

                    if container then
                        if container.state.widthInLayout then
                            Gui:setPercentSize(container.state.widthInLayout() * 100, 100)
                        else
                            Gui:setPercentSize(100 / contentCount, 100) -- even distribution
                        end
                    end
                elseif self.state.layoutType() == GuiLayoutType.Vertical then
                    if self.state.showGrid() then
                        Gui:beginStackContainer()
                        Gui:setPercentSize(100, 100)
                        Gui:setAlignment(AlignHorizontal.Right, AlignVertical.Top)
                        Gui:setBorderWidth(2)
                        Gui:setBorderColor(self.state.showGridColor())

                        Gui:text(tostring(index), Cache.Font("Exo2", 14), self.state.showGridColor())

                        if type(container) == "function" then
                            container = container() -- dynamic components
                        end

                        if container then
                            container:render()
                        end

                        Gui:endContainer()
                    else
                        Gui:beginStackContainer()
                        Gui:setPercentSize(100, 100)

                        if type(container) == "function" then
                            container = container() -- dynamic components
                        end

                        if container then
                            container:render()
                        end

                        Gui:endContainer()
                    end

                    if container then
                        if container.state.heightInLayout then
                            Gui:setPercentSize(100, container.state.heightInLayout() * 100)
                        else
                            Gui:setPercentSize(100, 100 / contentCount) -- even distribution
                        end
                    end
                end
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
