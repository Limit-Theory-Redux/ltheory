local Button = {}
Button.__index = Button

local meta = {
    __call = function(cls, ...)
        return cls:new(...)
    end
}

---@class Button
---@field render function renders the button

---returns a button object
---@param args {title: string, callback: function }
---@return Button|nil
function Button:new(args)
    if not args then
        return
    end

    local newButton = {}
    newButton.title = args.title
    newButton.callback = args.callback

    newButton.render = function()
        if Gui:button(newButton.title) then newButton.callback() end
    end

    return newButton
end

setmetatable(Button, meta)

-- Add to global UIComponent table
UIComponent.Button = Button

return Button