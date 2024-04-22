local View = {}
View.__index = View

local meta = {
    __call = function(cls, ...)
        return cls:new(...)
    end
}

---@class UIView
---@field name string
---@field contents table
---@field windowCount integer
---@field addWindowToView fun(self: UIView, window: UIComponentWindow)
---@field addContent fun(self, component: UIComponent)
---@field render fun(self: UIView)

---@class UIViewConstructor
---@field name string
---@field contents table

---returns a view object
---@param args UIViewConstructor
---@return UIView|nil
function View:new(args)
    if not args then
        return
    end

    local newView = {}
    newView.name = args.name
    newView.contents = args.contents
    newView.windowCount = 0 -- 0 on init

    newView.addWindowToView = function(self, window)
        if not window then
            Log.Error("window nil")
        end

        self.windowCount = self.windowCount + 1
        -- assing window count as id
        window.id = self.windowCount
        self.contents[window.id] = window
    end

    newView.addContent = function(self, component)
        if not component then
            Log.Error("content nil")
        end

        table.insert(self.contents, component)
    end

    newView.render = function(self)
        if #self.contents > 1 then
            for _, component in ipairs(self.contents) do
                -- if content is window
                if component.close or not component.visible then
                    ---@cast component UIComponentWindow
                    component.close = false
                    component.visible = false
                    goto skip
                end
                component:render()
                ::skip::
            end
        else
            self.contents[1]:render()
        end
    end

    return newView
end

setmetatable(View, meta)

-- Add to global UICore table
---@type UIViewConstructor
UICore.View = View

return View
