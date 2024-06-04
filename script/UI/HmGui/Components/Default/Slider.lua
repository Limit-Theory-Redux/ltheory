local Slider = {}
Slider.__index = Slider

local meta = {
    __call = function(cls, ...)
        return cls:new(...)
    end
}

---@class UIComponentSlider: UIComponent
---@field visible boolean
---@field title string
---@field width number
---@field height number
---@field sound SFXObject|nil
---@field color UIComponentSliderColors
---@field callback function
---@field render fun(self: UIComponentSlider) renders the button

---@class UIComponentSliderConstructor
---@field visible boolean
---@field title string
---@field width number
---@field height number
---@field padding { paddingX: number, paddingY: number }|nil
---@field margin { marginX: number, marginY: number }|nil
---@field align { h: AlignHorizontal, v: AlignVertical }|{ h: AlignHorizontal.Default, v: AlignVertical.Default}
---@field color UIComponentSliderColors
---@field font UIComponentFont
---@field sound SFXObject|nil
---@field callback function

---@class UIComponentSliderColors
---@field text Color|nil
---@field background Color|nil
---@field highlight Color|nil

---@class UIComponentFont
---@field name string
---@field size number

---returns a button object
---@param args UIComponentSliderConstructor
---@return UIComponentSlider|nil
function Slider:new(args)
    if not args then
        return
    end

    local newSlider = {}
    newSlider.state = UICore.ComponentState {
        visible = args.visible,
        title = args.title,
        width = args.width,
        height = args.height,
        padding = args.padding or { 10, 10 },
        margin = args.margin,
        align = args.align or { AlignHorizontal.Default, AlignVertical.Default },
        color = {
            text = args.color and args.color.text or Color(1.0, 1.0, 1.0, 1.0),
            background = args.color and args.color.background or Color(0.85, 0.85, 0.85, 1),
            highlight = args.color and args.color.highlight or Color(0.95, 0.95, 0.95, 1)
        },
        font = args.font or { name = "Exo2Bold", size = 12 },
        minValue = args.minValue or 0,
        maxValue = args.maxValue or 100,
        currentValue = args.currentValue or 50,
        increment = args.increment or 0,
        showValueAsPercentage = args.showValueAsPercentage or false, -- if value is representing 0-100 by 0-1
        sound = args.sound,
        callback = args.callback or function() Log.Warn("undefined slider callback function: " .. tostring(args.title)) end
    }

    newSlider.render = function(self)
        if not self.state.visible() then
            return
        end

        if self.state.title and self.state.title() then
            Gui:text(self.state.title(), Cache.Font(self.state.font().name, self.state.font().size),
                self.state.color().text)
        end

        Gui:beginHorizontalContainer()

        if self.state.width then Gui:setFixedWidth(self.state.width()) end
        if self.state.height then Gui:setFixedHeight(self.state.height()) end

        if self.state.padding then Gui:setPadding(self.state.padding()[1], self.state.padding()[2]) end
        if self.state.margin then Gui:setMargin(self.state.margin()[1], self.state.margin()[2]) end

        local containerPos = Gui:containerPos()
        local containerSize = Gui:containerSize()

        -- check mouse interaction
        local isMouseOverSlider = Gui:isMouseOver(FocusType.Mouse)
        local sliderHeld = isMouseOverSlider and InputInstance:mouse():isDown(MouseControl.Left)

        -- slider stack
        Gui:beginStackContainer()
        Gui:setAlignment(AlignHorizontal.Stretch, AlignVertical.Stretch)
        Gui:setBackgroundColor(Color(1, 1, 1, 0.2))

        -- slider body
        Gui:setAlignment(AlignHorizontal.Stretch, AlignVertical.Stretch)
        Gui:rect()

        local minValue = self.state.minValue() or 0
        local maxValue = self.state.maxValue() or 100
        local outerValue = self.state.currentValue()

        -- create internal representation
        local internalValue = (outerValue - minValue) / (maxValue - minValue) * 100
        local sliderValuePercent = internalValue

        if sliderHeld then
            -- calculate relative mouse position
            local relativeMousePositionX = InputInstance:mouse():position().x -
                (containerPos.x + self.state.padding()[1])

            -- clamp the relative position to be within the container bounds
            if relativeMousePositionX < 0 then
                relativeMousePositionX = 0
            elseif relativeMousePositionX > (containerSize.x - 2 * self.state.padding()[1]) then
                relativeMousePositionX = (containerSize.x - 2 * self.state.padding()[1])
            end

            internalValue = (relativeMousePositionX / (containerSize.x - 2 * self.state.padding()[1])) * 100

            local increment = self.state.increment()

            if increment > 0 then
                -- scale increment to match the internal range (0-100)
                local scaledIncrement = increment / (maxValue - minValue) * 100
                internalValue = math.floor(internalValue / scaledIncrement + 0.5) * scaledIncrement
                internalValue = math.max(0, math.min(internalValue, 100)) -- ensure internalValue is within the bounds
            end

            sliderValuePercent = internalValue

            local newOuterValue = internalValue / 100 * (maxValue - minValue) + minValue

            if not self.state.lastValue then
                self.state.lastValue = outerValue
            end

            if newOuterValue ~= self.state.lastValue then
                self.state.lastValue = self.state.currentValue()
                self.state.currentValue = function() return newOuterValue end

                if self.state.sound then
                    self.state.sound():Play(1.0)
                end
            end

            -- set the background color to indicate interaction
            Gui:setBackgroundColor(self.state.color().highlight)
        else
            -- set default background color
            Gui:setBackgroundColor(self.state.color().background)
        end

        Gui:setPercentWidth(sliderValuePercent)

        -- slider thumb
        Gui:beginHorizontalContainer()
        Gui:setAlignment(AlignHorizontal.Stretch, AlignVertical.Stretch)

        Gui:rect()
        Gui:setPercentWidth(sliderValuePercent - 0.5)
        Gui:setBackgroundColor(Color(1, 1, 1, 0))

        Gui:rect()
        Gui:setAlignment(AlignHorizontal.Right, AlignVertical.Default)
        Gui:setPercentWidth(0.5)
        Gui:setFixedHeight(self.state.height())
        Gui:setBackgroundColor(Color(0, 0, 0, 1))
        Gui:setBorderColor(Color(0.5, 1, 0.5, 0.5))
        Gui:setBorderWidth(5)
        Gui:endContainer()

        -- slider value text
        local sliderValueText

        if not self.state.showValueAsPercentage() then
            sliderValueText = string.format("%.2f", outerValue)       -- round to 2 decimals
        else
            sliderValueText = string.format("%.0f", outerValue * 100) -- display as percentage
        end
        Gui:text(sliderValueText,
            Cache.Font(self.state.font().name, self.state.font().size),
            Color(0.7, 0.7, 0.7, 1))
        Gui:setAlignment(AlignHorizontal.Center, AlignVertical.Center)

        Gui:endContainer()
        Gui:endContainer()

        -- callback
        if self.state.currentValue() ~= self.state.lastValue then
            self.state.callback(self.state.currentValue())
        end
    end

    return newSlider
end

setmetatable(Slider, meta)

-- Add to global UIComponent table
---@type UIComponentSliderConstructor
UIComponent.Slider = Slider

return Slider