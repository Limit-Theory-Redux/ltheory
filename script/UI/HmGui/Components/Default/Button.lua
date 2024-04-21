local Button = {}
Button.__index = Button

local meta = {
    __call = function(cls, ...)
        return cls:new(...)
    end
}

---@class UIComponentButton
---@field title string
---@field width number
---@field callback function
---@field render function renders the button

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
---@type UIComponentButtonConstructor
UIComponent.Button = Button

return Button