local Checkbox = {}
Checkbox.__index = Checkbox

local meta = {
    __call = function(cls, ...)
        return cls:new(...)
    end
}

---@class UIComponentCheckbox: UIComponent
---@field visible boolean
---@field title string
---@field width number
---@field height number
---@field sound SFXObject|nil
---@field color UIComponentCheckboxColors
---@field font UIComponentFont
---@field callback function
---@field render fun(self: UIComponentCheckbox) renders the checkbox

---@class UIComponentCheckboxConstructor
---@field checked boolean
---@field visible boolean
---@field title string
---@field width number
---@field height number
---@field padding { paddingX: number, paddingY: number }|nil
---@field margin { marginX: number, marginY: number }|nil
---@field align { h: AlignHorizontal, v: AlignVertical }|{ h: AlignHorizontal.Default, v: AlignVertical.Default}
---@field textAlign { h: AlignHorizontal, v: AlignVertical }|{ h: AlignHorizontal.Left, v: AlignVertical.Center}
---@field font UIComponentFont
---@field color UIComponentCheckboxColors
---@field sound SFXObject|nil
---@field callback function

---@class UIComponentCheckboxColors
---@field text Color|nil
---@field background Color|nil
---@field highlight Color|nil
---@field clickArea { border: Color, checked: Color, notChecked: Color}|nil

---@class UIComponentFont
---@field name string
---@field size number

---returns a checkbox object
---@param args UIComponentCheckboxConstructor
---@return UIComponentCheckbox|nil
function Checkbox:new(args)
    if not args then
        return
    end

    local newCheckbox = {}
    newCheckbox.state = UICore.ComponentState {
        checked = args.checked or false,
        visible = args.visible,
        title = args.title,
        width = args.width,
        height = args.height,
        padding = args.padding,
        margin = args.margin,
        align = args.align or { AlignHorizontal.Default, AlignVertical.Default },
        textAlign = args.textAlign or { AlignHorizontal.Left, AlignVertical.Center },
        font = args.font or { name = "Exo2", size = 12 },
        clickArea = {
            size = { 10, 10 },
            borderWidth = 3,
        },
        color = {
            text = args.color and args.color.text or Color(1.0, 1.0, 1.0, 1.0),
            background = args.color and args.color.background or Color(0.0, 0.0, 0.0, 1.0),
            highlight = args.color and args.color.highlight or Color(0.1, 0.1, 0.1, 1.0),
            clickArea = {
                border = args.color and args.color.clickArea.border or Color(1, 1, 1, 1),
                checked = args.color and args.color.clickArea.checked or Color(0.1, 0.5, 1, 1),
                notChecked = args.color and args.color.clickArea.notChecked or Color(0, 0, 0, 0),
            }
        },
        sound = args.sound,
        callback = args.callback or function(checked) Log.Warn("undefined checkbox callback function: " .. args.title) end
    }

    newCheckbox.render = function(self)
        if not self.state.visible() then
            return
        end

        Gui:setProperty(GuiProperties.Opacity, 1.0)
        Gui:setProperty(GuiProperties.BackgroundColor, self.state.color().background)
        Gui:setProperty(GuiProperties.HighlightColor, self.state.color().highlight)
        Gui:beginHorizontalContainer()
        Gui:setAlignment(self.state.align()[1], self.state.align()[2])

        local triggered = Gui:isMouseOver(FocusType.Mouse) and InputInstance:mouse():isPressed(MouseControl.Left)
        if triggered then
            self.checked = not self.checked
        end

        -- no need for an if check, since we always have a default defined
        Gui:setProperty(GuiProperties.TextFont, Cache.Font(self.state.font().name, self.state.font().size))
        Gui:setProperty(GuiProperties.TextColor, self.state.color().text)

        Gui:text(self.state.title())
        Gui:setAlignment(self.state.textAlign()[1], self.state.textAlign()[2])

        Gui:spacer()

        if self.checked then
            Gui:setProperty(GuiProperties.BackgroundColor, self.state.color().clickArea.checked)
        else
            Gui:setProperty(GuiProperties.BackgroundColor, self.state.color().clickArea.notChecked)
        end

        Gui:setProperty(GuiProperties.BorderColor, self.state.color().clickArea.border)
        Gui:rect()
        Gui:setFixedSize(self.state.clickArea().size[1], self.state.clickArea().size[2])
        Gui:setBorderWidth(self.state.clickArea().borderWidth)
        Gui:setVerticalAlignment(AlignVertical.Center)

        Gui:endContainer()

        if self.state.width then Gui:setFixedWidth(self.state.width()) end
        if self.state.height then Gui:setFixedHeight(self.state.height()) end

        if self.state.padding then Gui:setPadding(self.state.padding()[1], self.state.padding()[2]) end
        if self.state.margin then Gui:setMargin(self.state.margin()[1], self.state.margin()[2]) end

        Gui:clearStyle() -- clear style so it doesn´t affect other components

        if triggered then
            if self.state.sound then
                self.state.sound():Play(1.0)
            end

            -- print("-> " .. tostring(self.checked))
            self.state.callback(self.checked)
        end
    end

    return newCheckbox
end

setmetatable(Checkbox, meta)

-- Add to global UIComponent table
---@type UIComponentCheckboxConstructor
UIComponent.Checkbox = Checkbox

return Checkbox
