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
---@field ignoreUIScaling boolean -- if true, will not scale with GameState.ui.scaleFactor

---returns a ResponsiveSize object
---@param baseX number
---@param baseY number
---@return ResponsiveSize|nil
function ResponsiveSize:new(baseX, baseY, ignoreUIScaling)
    if not baseX or not baseY then
        return
    end

    --todo: remove hardcode to 16:9 aspect ratio
    local newResponsiveSize = {}
    newResponsiveSize.baseX = baseX
    newResponsiveSize.baseY = baseY
    newResponsiveSize.useUIScale = not ignoreUIScaling or false

    local mt = {
        __index = function(self, key)
            if key == "x" then
                if self.useUIScale then
                    return GameState.render.resX / 1600 * self.baseX * GameState.ui.scaleFactor
                else
                    return GameState.render.resX / 1600 * self.baseX
                end
            elseif key == "y" then
                if self.useUIScale then
                    return GameState.render.resY / 900 * self.baseY * GameState.ui.scaleFactor
                else
                    return GameState.render.resY / 900 * self.baseY
                end
            end
        end
    }

    setmetatable(newResponsiveSize, mt)

    return newResponsiveSize
end

setmetatable(ResponsiveSize, meta)

return ResponsiveSize
