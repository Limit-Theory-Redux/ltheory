local Spacer = {}
Spacer.__index = Spacer

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
        HmGui.SetSpacing(newSpacer.size)
    end

    return newSpacer
end

return Spacer