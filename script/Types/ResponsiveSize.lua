local ResponsiveSize = {}
ResponsiveSize.__index = ResponsiveSize

local meta = {
    __call = function(cls, ...)
        return cls:new(...)
    end
}

---@class ResponsiveSize
---@field x number
---@field y number

---returns a ResponsiveSize object
---@param baseX number
---@param baseY number
---@return ResponsiveSize|nil
function ResponsiveSize:new(baseX, baseY)
    if not baseX or not baseY then
        return
    end

    --todo: remove hardcode to 16:9 aspect ratio
    local newResponsiveSize = {}
    newResponsiveSize.baseX = baseX
    newResponsiveSize.baseY = baseY

    local mt = {
        __index = function(self, key)
            if key == "x" then
                return GameState.render.resX / 1600 * self.baseX
            elseif key == "y" then
                return GameState.render.resY / 900 * self.baseY
            end
        end
    }

    setmetatable(newResponsiveSize, mt)

    return newResponsiveSize
end

setmetatable(ResponsiveSize, meta)

return ResponsiveSize
