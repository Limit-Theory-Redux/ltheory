local View = {}
View.__index = View

local meta = {
    __call = function(cls, ...)
        return cls:new(...)
    end
}

---@class UIView
---@field name string
---@field content table
---@field windowCount integer
---@field addWindowToView fun(self: UIView, window: UIComponentWindow)

---@class UIViewConstructor
---@field name string
---@field content table

---returns a view object
---@param args UIViewConstructor
---@return UIView|nil
function View:new(args)
    if not args then
        return
    end

    local newView = {}
    newView.name = args.name
    newView.content = args.content
    newView.windowCount = 0 -- 0 on init

    newView.addWindowToView = function(self, window)
        if not window then
            Log.Error("window nil")
        end

        self.windowCount = self.windowCount + 1
        -- assing window count as id
        window.id = self.windowCount
        self.content[window.id] = window
    end

    return newView
end

setmetatable(View, meta)

-- Add to global UICore table
---@type UIViewConstructor
UICore.View = View

return View
