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
---@param args {title: string, width: number, callback: function }
---@return Button|nil
function Button:new(args)
    if not args then
        return
    end

    local newButton = {}
    newButton.title = args.title
    newButton.width = args.width
    newButton.callback = args.callback

    newButton.render = function()
        if Gui:button(newButton.title) then newButton.callback() end
        if newButton.width then Gui:setFixedWidth(newButton.width) end
    end

    return newButton
end

setmetatable(Button, meta)

-- Add to global UIComponent table
UIComponent.Button = Button

return Button