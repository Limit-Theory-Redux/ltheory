local Button = {}
Button.__index = Button

local meta = {
    __call = function(cls, ...)
        return cls:new(...)
    end
}

---@class UIComponentButton: UIComponent
---@field visible boolean
---@field title string
---@field width number
---@field sound SFXObject|nil
---@field callback function
---@field render fun(self: UIComponentButton) renders the button

---@class UIComponentButtonConstructor
---@field visible boolean
---@field title string
---@field width number
---@field sound SFXObject|nil
---@field callback function

---returns a button object
---@param args UIComponentButtonConstructor
---@return UIComponentButton|nil
function Button:new(args)
    if not args then
        return
    end

    local newButton = {}
    newButton.state = UICore.ComponentState {
        visible = args.visible or true,
        title = args.title,
        width = args.width,
        sound = args.sound,
        callback = args.callback
    }

    newButton.render = function(self)
        if Gui:button(self.state.title()) then
            if self.state.sound then
                self.state.sound():Play(1.0)
            end

            self.state.callback()
        end
        if self.state.width then Gui:setFixedWidth(self.state.width()) end

        Gui:clearStyle() -- clear style so it doesn´t affect other components
    end

    return newButton
end

setmetatable(Button, meta)

-- Add to global UIComponent table
---@type UIComponentButtonConstructor
UIComponent.Button = Button

return Button
