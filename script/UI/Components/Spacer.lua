local Spacer = {}
Spacer.__index = Spacer

local meta = {
    __call = function(cls, ...)
        return cls:new(...)
    end
}

---@class Spacer
---@field render function renders the spacer

---returns a spacer object
---@param args {size: number}
---@return Spacer|nil
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
UIComponent.Spacer = Spacer

return Spacer