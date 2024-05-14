local Window = {}
Window.__index = Window

local meta = {
    __call = function(cls, ...)
        return cls:new(...)
    end
}

---@class UIComponentWindow: UIComponent
---@field id integer
---@field visible boolean
---@field title string
---@field alignment AlignHorizontal|AlignVertical
---@field canClose boolean
---@field containers table<UIComponentContainer>
---@field setVisible fun(visible: boolean)
---@field render fun(self: UIComponentWindow)
---@field close boolean

---@class UIComponentWindowConstructor
---@field id integer
---@field visible boolean
---@field title string
---@field alignment AlignHorizontal|AlignVertical
---@field canClose boolean
---@field containers table<UIComponentContainer>

---returns a window object
---@param args UIComponentWindowConstructor
---@return UIComponentWindow|nil
function Window:new(args)
    if not args then
        Log.Error("nil ui window arguments")
        return
    end

    local newWindow = {
        id = nil, -- id assigned via view
        visible = true,
        title = args.title,
        stackAlignment = args.alignment,
        canClose = args.canClose,
        containers = {}
    }

    newWindow.close = false

    newWindow.containers = args.containers

    newWindow.setVisible = function(self, visible)
        if visible == nil then
            Log.Error("set visible bool nil")
        end

        self.close = false -- reset
        self.visible = visible
    end

    newWindow.render = function(self)
        Gui:beginStackContainer()                                      -- begin game window
        Gui:setAlignment(AlignHorizontal.Center, AlignVertical.Center) --! hardcoded to center right now, if we opt for this solution i will add configuration options
        Gui:beginWindow(self.title)
        Gui:setProperty(GuiProperties.TextFont, Cache.Font('Exo2', 26))
        Gui:textColored(self.title, Color(1, 1, 1, 0.25))

        -- temp until i figure out how to do groups properly
        if self.stackDirection == Enums.UI.StackDirection.Vertical or not self.stackDirection then
            Gui:beginVerticalContainer()
        elseif self.stackDirection == Enums.UI.StackDirection.Horizontal then
            Gui:beginHorizontalContainer()
        end

        for _, container in ipairs(self.containers) do
            local subGroup
            if container.stackDirection == Enums.UI.StackDirection.Vertical then
                Gui:beginVerticalContainer()
                subGroup = true
            elseif container.stackDirection == Enums.UI.StackDirection.Horizontal then
                Gui:beginHorizontalContainer()
                subGroup = true
            end

            -- apply padding
            if container.padding then
                Gui:setPadding(container.padding[1], container.padding[2])
            end

            if container.align then
                Gui:setAlignment(container.align[1], container.align[2])
            end

            -- render content
            for _, element in ipairs(container.contents) do
                -- temp until i figure out how to do groups properly
                if not element.stackDirection or element.stackDirection == Enums.UI.StackDirection.Vertical then
                    Gui:beginVerticalContainer()
                elseif element.stackDirection == Enums.UI.StackDirection.Horizontal then
                    Gui:beginHorizontalContainer()
                end
                Gui:setAlignment(AlignHorizontal.Center, AlignVertical.Center)

                element.render()
                Gui:endContainer()
            end

            if subGroup then
                Gui:endContainer()
            end
        end

        Gui:endContainer()

        if args.canClose then
            if Gui:button("Close") then self.close = true end
            Gui:setAlignment(AlignHorizontal.Center, AlignVertical.Center)
            Gui:setFixedWidth(120)
        end
        Gui:endWindow()
        Gui:endContainer() -- end game window
    end

    return newWindow
end
