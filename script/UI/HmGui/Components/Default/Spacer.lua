local Spacer = {}
Spacer.__index = Spacer

local meta = {
    __call = function(cls, ...)
        return cls:new(...)
    end
}

---@class UIComponentSpacer: UIComponent
---@field size number
---@field render function renders the spacer

---@class UIComponentSpacerConstructor
---@field size number

---returns a spacer object
---@param args UIComponentSpacerConstructor
---@return UIComponentSpacer|nil
function Spacer:new(args)
    if not args then
        return
    end

    local newSpacer = {}
    newSpacer.size = args.size

    newSpacer.render = function()
        Gui:setSpacing(newSpacer.size)
    end

    return newSpacer
end

setmetatable(Spacer, meta)

-- Add to global UIComponent table
---@type UIComponentSpacerConstructor
UIComponent.Spacer = Spacer

return Spacer
