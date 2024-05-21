local ScrollArea = {}
ScrollArea.__index = ScrollArea

local meta = {
    __call = function(cls, ...)
        return cls:new(...)
    end
}

---@class UIComponent
---@field render function|nil

---@class UIComponentScrollArea: UIComponent
---@field visible boolean
---@field scrollDirection Enums.ScrollDirection|nil
---@field showHScroolbar boolean|true
---@field showVScroolbar boolean|true
---@field scroolbarFading boolean|true
---@field scrollScale number|20
---@field scrollbarVisibilityStableTimeMs number|400
---@field scrollbarVisibilityFadeTimeMs number|200
---@field scrollbarSize number|4
---@field scrollbarBackgroundColor Color|Color(0.3, 0.3, 0.3, 0.3)
---@field scrollbarKnobColor Color|Color(0.1, 0.1, 0.1, 0.5)
---@field align table<AlignHorizontal, AlignVertical>
---@field childrenAlign table<AlignHorizontal, AlignVertical>
---@field width number
---@field height number
---@field padding { paddingX: number, paddingY: number }|{ paddingX: 0, paddingY: 0 }
---@field margin { marginX: number, marginY: number }|{ marginX: 0, marginY: 0 }
---@field layoutType GuiLayoutType
---@field color UIComponentButtonColors
---@field render fun(self: UIComponentScrollArea)
---@field contents table
---@field showContainer boolean

---@class UIComponentScrollAreaConstructor
---@field visible boolean|nil
---@field scrollDirection Enums.ScrollDirection|nil
---@field showHScrollbar boolean
---@field showVScrollbar boolean
---@field scrollbarFading boolean
---@field scrollScale number
---@field scrollbarVisibilityStableTimeMs number
---@field scrollbarVisibilityFadeTimeMs number
---@field scrollbarSize number
---@field scrollbarBackgroundColor Color|nil
---@field scrollbarKnobColor Color|nil
---@field align table<AlignHorizontal, AlignVertical>
---@field childrenAlign table<AlignHorizontal, AlignVertical>
---@field width number
---@field height number
---@field padding { paddingX: number, paddingY: number }|nil
---@field margin { marginX: number, marginY: number }|nil
---@field layoutType GuiLayoutType
---@field color UIComponentButtonColors
---@field contents table
---@field showContainer boolean

---@class UIComponentScrollAreaColors
---@field background Color|nil

local function lerp(a, b, t) return a * (1 - t) + b * t end

---returns a scroll area object
---@param args UIComponentScrollAreaConstructor
---@return UIComponentScrollArea|nil
function ScrollArea:new(args)
    if not args then
        return
    end

    local newScrollArea = {
        scrollbarActivationTime = nil,
    }

    newScrollArea.state = UICore.ComponentState {
        visible = args.visible,
        scrollDirection = args.scrollDirection,
        showHScrollbar = args.showHScrollbar == nil or args.showHScrollbar, -- TODO: doesn't work. Idea is to treat nil as true. Same 2 below.
        showVScrollbar = args.showVScrollbar == nil or args.showVScrollbar,
        scrollbarFading = args.scrollbarFading == nil or args.scrollbarFading,
        scrollScale = args.scrollScale or 20,
        scrollbarVisibilityStableTimeMs = args.scrollbarVisibilityStableTimeMs or 400,
        scrollbarVisibilityFadeTimeMs = args.scrollbarVisibilityFadeTimeMs or 200,
        scrollbarSize = args.scrollbarSize or 4,
        scrollbarBackgroundColor = args.scrollbarBackgroundColor or Color(0.3, 0.3, 0.3, 1),
        scrollbarKnobColor = args.scrollbarKnobColor or Color(0.1, 0.1, 0.5, 1),
        align = args.align or { AlignHorizontal.Default, AlignVertical.Default },
        childrenAlign = args.childrenAlign or { AlignHorizontal.Default, AlignVertical.Default },
        padding = args.padding,
        margin = args.margin,
        width = args.width,
        height = args.height,
        layoutType = args.layoutType or GuiLayoutType.Vertical,
        contents = args.contents,
        color = {
            background = args.color and args.color.background
        },
        showContainer = args.showContainer or function() return GameState.debug.metricsEnabled end,
        showContainerColor = Color((math.random() + math.random(50, 99)) / 100, (math.random() + math.random(50, 99)) / 100, (math.random() + math.random(50, 99)) / 100, .4)
    }

    newScrollArea.render = function(self)
        if not self.state.visible() then
            return
        end

        -- color
        if self.state.color().background then
            Gui:setProperty(GuiProperties.BackgroundColor, self.state.color().background)
        end

        if self.state.showContainer() then
            Gui:setProperty(GuiProperties.BorderColor, self.state.showContainerColor())
        end

        -- external container
        Gui:beginStackContainer()
        Gui:clearStyle() -- clear properties

        if self.state.showContainer() then
            Gui:setBorderWidth(1)
        end

        Gui:setAlignment(self.state.align()[1], self.state.align()[2])
        Gui:setChildrenAlignment(self.state.childrenAlign()[1], self.state.childrenAlign()[2])

        if self.state.padding then Gui:setPadding(self.state.padding()[1], self.state.padding()[2]) end
        if self.state.margin then Gui:setMargin(self.state.margin()[1], self.state.margin()[2]) end

        if self.state.width then Gui:setFixedWidth(self.state.width()) end
        if self.state.height then Gui:setFixedHeight(self.state.height()) end

        -- internal 'scrollable' container
        Gui:beginContainer(self.state.layoutType())
        Gui:setAlignment(AlignHorizontal.Stretch, AlignVertical.Stretch)

        if #self.state.contents() > 1 then
            for _, component in ipairs(self.state.contents()) do
                component:render()
            end
        elseif #self.state.contents() == 1 then
            self.state.contents()[1]:render()
        end -- this allows scroll areas without any content

        -- recalculate container offset
        local innerSize = Gui:containerSize()
        local innerMinSize = Gui:containerMinSize()

        local maxScroll = Vec2f(math.max(0, innerMinSize.x - innerSize.x), math.max(0, innerMinSize.y - innerSize.y))

        -- store offset of the inner container
        local innerOffset = Gui:updateContainerOffset(maxScroll)

        Gui:endContainer()

        local hScroll = self.state.showHScrollbar and (self.state.scrollDirection ~= Enums.ScrollDirection.Vertical)
        local vScroll = self.state.showVScrollbar and (self.state.scrollDirection ~= Enums.ScrollDirection.Horizontal)

        if hScroll or vScroll then
            local isMouseOver = Gui:isMouseOver(FocusType.Scroll)
            local scroll = InputInstance:mouse():scroll()

            -- swap scroll values for horizontal scrolling
            if InputInstance:keyboard():isDown(KeyboardButton.ShiftLeft) then
                scroll = Vec2f(scroll.y, scroll.x)
            end

            local fadeScale = 1
            if self.state.scrollbarFading() then
                local mouseDelata = InputInstance:mouse():delta()
                if isMouseOver and
                    (math.abs(scroll.x) > 0.3 or math.abs(scroll.y) > 0.3 or
                        math.abs(mouseDelata.x) > 0.5 or math.abs(mouseDelata.y) > 0.5)
                then
                    self.scrollbarActivationTime = LimitTheoryRedux.lastUpdate
                elseif self.scrollbarActivationTime then
                    local elapsedTime = self.scrollbarActivationTime:getDifference(LimitTheoryRedux.lastUpdate) * 1000

                    if elapsedTime <= self.state.scrollbarVisibilityStableTimeMs() then
                        fadeScale = 1
                    elseif elapsedTime <= self.state.scrollbarVisibilityStableTimeMs() + self.state.scrollbarVisibilityFadeTimeMs() then
                        fadeScale = 1 - (elapsedTime - self.state.scrollbarVisibilityStableTimeMs())
                            / self.state.scrollbarVisibilityFadeTimeMs()
                    else
                        fadeScale = 0
                    end
                else
                    fadeScale = 0
                end
            end

            if isMouseOver then
                Gui:updateElementOffset(scroll:scale(self.state.scrollScale()))
            end

            if fadeScale > 0 then
                local sbSize = self.state.scrollbarSize()
                local sbBackgroundColor = Color(self.state.scrollbarBackgroundColor())
                sbBackgroundColor.a = sbBackgroundColor.a * fadeScale
                local sbKnobColor = Color(self.state.scrollbarKnobColor())
                sbKnobColor.a = sbKnobColor.a * fadeScale

                if hScroll and maxScroll.x > 0 then
                    Gui:beginHorizontalContainer()
                    Gui:setAlignment(AlignHorizontal.Stretch, AlignVertical.Bottom)

                    local knobSize = innerSize.x * (innerSize.x / innerMinSize.x);
                    local knobPos = lerp(0, (innerSize.x - knobSize), (innerOffset.x / maxScroll.x));

                    Gui:setProperty(GuiProperties.BackgroundColor, sbBackgroundColor);
                    Gui:rect();
                    Gui:setFixedSize(knobPos, sbSize);

                    Gui:setProperty(GuiProperties.BackgroundColor, sbKnobColor);
                    Gui:rect();
                    Gui:setFixedSize(knobSize, sbSize);

                    Gui:setProperty(GuiProperties.BackgroundColor, sbBackgroundColor);
                    Gui:rect();
                    Gui:setFixedHeight(sbSize);
                    Gui:setHorizontalAlignment(AlignHorizontal.Stretch);

                    Gui:endContainer()
                end

                if vScroll and maxScroll.y > 0 then
                    Gui:beginVerticalContainer()
                    Gui:setAlignment(AlignHorizontal.Right, AlignVertical.Stretch)

                    local knobSize = innerSize.y * (innerSize.y / innerMinSize.y);
                    local knobPos = lerp(0, (innerSize.y - knobSize), (innerOffset.y / maxScroll.y));

                    Gui:setProperty(GuiProperties.BackgroundColor, sbBackgroundColor);
                    Gui:rect();
                    Gui:setFixedSize(sbSize, knobPos);

                    Gui:setProperty(GuiProperties.BackgroundColor, sbKnobColor);
                    Gui:rect();
                    Gui:setFixedSize(sbSize, knobSize);

                    Gui:setProperty(GuiProperties.BackgroundColor, sbBackgroundColor);
                    Gui:rect();
                    Gui:setFixedWidth(sbSize);
                    Gui:setVerticalAlignment(AlignVertical.Stretch);

                    Gui:endContainer()
                end
            end
        end

        Gui:endContainer()

        Gui:clearStyle() -- clear style so it doesn´t affect other components
    end

    return newScrollArea
end

setmetatable(ScrollArea, meta)

-- Add to global UIComponent table
---@type UIComponentScrollAreaConstructor
UIComponent.ScrollArea = ScrollArea

return ScrollArea
