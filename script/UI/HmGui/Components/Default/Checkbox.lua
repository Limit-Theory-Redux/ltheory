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
---@field size ResponsiveSize -- if size is defined it will overwrite width, height
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
---@field size ResponsiveSize -- if size is defined it will overwrite width, height
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
        size = args.size,
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
            highlight = args.color and args.color.highlight or Color(0.5, 0.5, 0.5, 1.0),
            clickArea = {
                border = args.color and args.color.clickArea.border or Color(1, 1, 1, 1),
                checked = args.color and args.color.clickArea.checked or Color(0.1, 0.5, 1, 1),
                notChecked = args.color and args.color.clickArea.notChecked or Color(0, 0, 0, 0),
            }
        },
        sound = args.sound,
        callback = args.callback or function(checked)
            Log.Warn("undefined checkbox callback function: " ..
                tostring(args.title))
        end
    }

    newCheckbox.render = function(self)
        if not self.state.visible() then
            return
        end

        Gui:beginHorizontalContainer()
        Gui:setAlignment(self.state.align()[1], self.state.align()[2])
        Gui:setOpacity(1.0)

        local isMouseOver = Gui:isMouseOver(FocusType.Mouse)
        if isMouseOver then
            Gui:setBackgroundColor(self.state.color().highlight)
        else
            Gui:setBackgroundColor(self.state.color().background)
        end

        local triggered = isMouseOver and Input:mouse():isPressed(MouseControl.Left)
        if triggered then
            self.state.checked = not self.state.checked
        end

        -- no need for an if check, since we always have a default defined
        Gui:text(self.state.title(), Cache.Font(self.state.font().name, self.state.font().size), self.state.color().text)
        Gui:setAlignment(self.state.textAlign()[1], self.state.textAlign()[2])

        Gui:spacer()

        Gui:rect()
        Gui:setFixedSize(self.state.clickArea().size[1], self.state.clickArea().size[2])
        Gui:setBorderWidth(self.state.clickArea().borderWidth)
        Gui:setVerticalAlignment(AlignVertical.Center)
        Gui:setBorderColor(self.state.color().clickArea.border)

        if self.state.checked then
            Gui:setBackgroundColor(self.state.color().clickArea.checked)
        else
            Gui:setBackgroundColor(self.state.color().clickArea.notChecked)
        end

        Gui:endContainer()

        if self.state.size then
            local size = self.state.size()
            Gui:setFixedSize(size.x, size.y)
        else
            if self.state.width then Gui:setFixedWidth(self.state.width()) end
            if self.state.height then Gui:setFixedHeight(self.state.height()) end
        end

        if self.state.padding then Gui:setPadding(self.state.padding()[1], self.state.padding()[2]) end
        if self.state.margin then Gui:setMargin(self.state.margin()[1], self.state.margin()[2]) end

        if triggered then
            if self.state.sound then
                self.state.sound():play(1.0)
            end

            self.state.callback(self.state.checked)
        end
    end

    return newCheckbox
end

setmetatable(Checkbox, meta)

-- Add to global UIComponent table
---@type UIComponentCheckboxConstructor
UIComponent.Checkbox = Checkbox

return Checkbox
