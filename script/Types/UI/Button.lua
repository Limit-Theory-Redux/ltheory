local Button = {}
Button.__index = Button

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
        if HmGui.Button(newButton.title) then newButton.callback() end
    end

    return newButton
end

return Button