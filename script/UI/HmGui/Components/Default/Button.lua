local Button = {}
Button.__index = Button

local meta = {
    __call = function(cls, ...)
        return cls:new(...)
    end
}

---@class UIComponentButton: UIComponent
---@field title string
---@field width number
---@field callback function
---@field render fun(self: UIComponentButton) renders the button

---@class UIComponentButtonConstructor
---@field title string
---@field width number
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
        title = args.title,
        width = args.width,
        callback = args.callback
    }

    newButton.render = function(self)
        if Gui:button(self.state.title()) then self.state.callback() end
        if self.state.width then Gui:setFixedWidth(self.state.width()) end
    end

    return newButton
end

setmetatable(Button, meta)

-- Add to global UIComponent table
---@type UIComponentButtonConstructor
UIComponent.Button = Button

return Button
