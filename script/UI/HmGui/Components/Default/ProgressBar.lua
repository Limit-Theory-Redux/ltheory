local ProgressBar = {}
ProgressBar.__index = ProgressBar

local meta = {
    __call = function(cls, ...)
        return cls:new(...)
    end
}

---@class UIComponentProgressBar: UIComponent
---@field visible boolean
---@field title string
---@field width number
---@field height number
---@field size ResponsiveSize -- if size is defined it will overwrite width, height
---@field color UIComponentProgressBarColors
---@field callback function
---@field render fun(self: UIComponentProgressBar) renders the button

---@class UIComponentProgressBarConstructor
---@field visible boolean
---@field title string
---@field width number
---@field height number
---@field size ResponsiveSize -- if size is defined it will overwrite width, height
---@field padding { paddingX: number, paddingY: number }|nil
---@field margin { marginX: number, marginY: number }|nil
---@field align { h: AlignHorizontal, v: AlignVertical }|{ h: AlignHorizontal.Default, v: AlignVertical.Default}
---@field textAlign { h: AlignHorizontal, v: AlignVertical }|{ h: AlignHorizontal.Center, v: AlignVertical.Center}
---@field color UIComponentProgressBarColors
---@field font UIComponentFont
---@field toolTip UIComponentToolTip
---@field minValue number
---@field maxValue number
---@field currentValue number
---@field showValueAsPercentage boolean
---@field callback function

---@class UIComponentProgressBarColors
---@field text Color|nil
---@field background Color|nil
---@field borderWidth number
---@field borderColor Color
---@field highlight Color|nil
---@field thumb Color|nil

---returns a progress bar object
---@param args UIComponentProgressBarConstructor
---@return UIComponentProgressBar|nil
function ProgressBar:new(args)
    if not args then
        return
    end

    local newProgressBar = {}
    newProgressBar.state = UICore.ComponentState {
        visible = args.visible,
        title = args.title,
        width = args.width,
        height = args.height,
        size = args.size,
        padding = args.padding or { 0, 0 }, -- if non-zero, will remove the top and bottom of the inner progress bar
        margin = args.margin,
        borderWidth = args.borderWidth or 2,
        borderColor = args.borderColor or Color(0.0, 0.0, 0.0, 1.0),
        align = args.align or { AlignHorizontal.Default, AlignVertical.Default },
        textAlign = args.textAlign or { AlignHorizontal.Center, AlignVertical.Center },
        color = {
            text = args.color and args.color.text or Color(0.7, 0.7, 0.7, 1.0),
            background = args.color and args.color.background or Color(0.85, 0.85, 0.85, 1.0),
            highlight = args.color and args.color.highlight or Color(0.95, 0.95, 0.95, 1.0),
        },
        font = args.font or { name = "Unageo-Medium", size = 12 },
        toolTip = UIComponent.ToolTip { text = args.toolTip },
        minValue = args.minValue or 0,
        maxValue = args.maxValue or 100,
        currentValue = args.currentValue or 50,
    }

    newProgressBar.render = function(self)
        if not self.state.visible() then
            return
        end

        Gui:beginHorizontalContainer()
        Gui:setAlignment(self.state.textAlign()[1], self.state.textAlign()[2])

        if self.state.title and self.state.title() then
            Gui:text(self.state.title(), Cache.Font(self.state.font().name, self.state.font().size),
                self.state.color().text)
            Gui:setAlignment(self.state.textAlign()[1], self.state.textAlign()[2])
        end

        Gui:beginHorizontalContainer()

        if self.state.size then
            local size = self.state.size()
            Gui:setFixedSize(size.x, size.y)
        else
            if self.state.width then Gui:setFixedWidth(self.state.width()) end
            if self.state.height then Gui:setFixedHeight(self.state.height()) end
        end

        if self.state.padding then Gui:setPadding(self.state.padding()[1], self.state.padding()[2]) end
        if self.state.margin then Gui:setMargin(self.state.margin()[1], self.state.margin()[2]) end

        local containerPos = Gui:containerPos()
        local containerSize = Gui:containerSize()

        Gui:setBorderWidth(self.state.borderWidth())
        Gui:setBorderColor(self.state.borderColor())

        -- check mouse interaction
        local isMouseOverProgressBar = Gui:isMouseOver(FocusType.Mouse)

        -- progress bar stack
        Gui:beginStackContainer()
        Gui:setAlignment(AlignHorizontal.Stretch, AlignVertical.Stretch)

        -- progress bar body
        Gui:setBackgroundColor(Color(0.3, 0.3, 0.3, 0.7))
        Gui:rect()
        if self.state.size then
            local size = self.state.size()
            Gui:setFixedSize(size.x, size.y)
        else
            if self.state.width then Gui:setFixedWidth(self.state.width()) end
            if self.state.height then Gui:setFixedHeight(self.state.height()) end
        end

        local minValue = self.state.minValue() or 0
        local maxValue = self.state.maxValue() or 100
        local outerValue = self.state.currentValue()

        -- create internal representation
        local internalValue = (outerValue - minValue) / (maxValue - minValue) * 100
        local progressbarValuePercent = internalValue

        -- set default background color
        Gui:setBackgroundColor(self.state.color().background)

        Gui:setPercentWidth(progressbarValuePercent)

        -- progress bar value text
        local progressbarValueText = string.format("%.0f%%", outerValue) -- display as percentage

        Gui:text(progressbarValueText,
            Cache.Font(self.state.font().name, floor((self.state.size().y or self.state.height()) / 1.1)), -- calculate interior text size from progress bar height
            self.state.color().text)
        Gui:setAlignment(AlignHorizontal.Center, AlignVertical.Center)

        Gui:endContainer()
        Gui:endContainer()

        self.state.toolTip():render()

        Gui:endContainer()
    end

    return newProgressBar
end

---sets the current value of the progress bar (use 0 to 100)
---@param percentVal number
function ProgressBar:setPercent(percentVal)
    if percentVal >= 0 and percentVal <= 100 then
        self.currentValue = percentVal
    end
end

setmetatable(ProgressBar, meta)

-- Add to global UIComponent table
---@type UIComponentProgressBarConstructor
UIComponent.ProgressBar = ProgressBar

return ProgressBar
