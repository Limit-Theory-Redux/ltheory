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
---@field input fun(self: UIView)
---@field update fun(self: UIView, dt: integer)
---@field close fun(self: UIView)
---@field onInput fun(self: UIView)
---@field onUpdate fun(self: UIView, dt: integer)
---@field onViewClose fun(self: UIView, isPageClose: boolean)
---@field onViewOpen fun(self: UIView, isPageOpen: boolean)

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
    newView.contents = args.contents or {}
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

    newView.close = function(self, isPageClose)
        if self.onViewClose then
            self:onViewClose(isPageClose)
        end
    end

    newView.open = function(self, isPageOpen)
        if self.onViewOpen then
            self:onViewOpen(isPageOpen)
        end
    end

    newView.input = function(self)
        if self.onInput then
            self:onInput()
        end
    end

    newView.update = function(self, dt)
        if self.onUpdate then
            -- call onUpdate function before rendering the components
            self:onUpdate(dt)
        end

        if #self.contents > 1 then
            for _, component in ipairs(self.contents) do
                if type(component) == "function" then
                    component = component() -- dynamic components
                end

                -- if component is set to not visible
                if component.state.visible and not component.state.visible() then
                    goto skip
                end

                -- if component is a window
                if component.state.close and not component.state.close() then
                    ---@cast component UIComponentWindow
                    component.close = false
                    component.visible = false
                    goto skip
                end
                component:render()
                ::skip::
            end
        else
            if not self.contents[1] then
                return
            end
            -- if component is set to not visible
            if self.contents[1].state.visible and not self.contents[1].state.visible() then
                return
            end
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
