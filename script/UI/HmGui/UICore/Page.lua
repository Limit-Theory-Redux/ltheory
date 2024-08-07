local Page = {}
Page.__index = Page

local meta = {
    __call = function(cls, ...)
        return cls:new(...)
    end
}

---@class UIPage
---@field name string
---@field views string<UIView>
---@field currentView integer
---@field addViewToPage fun(self: UIPage, view: UIView)
---@field setView fun(self: UIPage, viewName: string)
---@field getLastView fun(self: UIPage)
---@field getCurrentView fun(self: UIPage)
---@field getAvailableViews fun(self: UIPage)
---@field addContent fun(self, component: UIComponent)
---@field input fun(self: UIPage)
---@field update fun(self: UIPage, dt: integer)
---@field onInput fun(self: UIPage)
---@field onUpdate fun(self: UIPage, dt: integer)
---@field onPageOpen fun(self: UIPage)
---@field onPageClose fun(self: UIPage)

---@class UIPageConstructor
---@field name string
---@field views string<UIView>
---@field contents table<UIComponent|UILayout>

---returns a page object
---@param args UIPageConstructor
---@return UIPage|nil
function Page:new(args)
    if not args then
        return
    end

    local newPage = {}
    newPage.name = args.name
    newPage.views = args.views or {}
    newPage.lastView = nil
    newPage.currentView = nil
    newPage.contents = args.contents or {}

    newPage.addContent = function(self, component)
        if not component then
            Log.Error("content nil")
        end

        table.insert(self.contents, component)
    end

    newPage.addViewToPage = function(self, view)
        if not view then
            Log.Error("view nil")
        end

        self.views[view.name] = view
    end

    newPage.setView = function(self, viewName)
        if not viewName then
            Log.Error("viewName nil")
        elseif not self.views[viewName] then
            Log.Error("view doesnt exist")
        end

        if self.lastView ~= self.currentView then
            self.lastView = self.currentView
            self.lastView:close() -- calls onCloseView()
        end
        self.currentView = self.views[viewName]
        self.currentView:open() -- calls onOpenView()
    end

    newPage.getLastView = function(self)
        return self.lastView
    end

    newPage.getCurrentView = function(self)
        return self.currentView
    end

    newPage.getAvailableViews = function(self)
        local viewNames = {}
        for name, page in pairs(self.views) do
            table.insert(viewNames, name)
        end
        return viewNames
    end

    newPage.close = function(self)
        if self.onPageClose then
            self:onPageClose()
        end

        if self:getCurrentView() then
            self:getCurrentView():close(true)
        end
    end

    newPage.open = function(self)
        if self.onPageOpen then
            self:onPageOpen()
        end

        if self:getCurrentView() then
            self:getCurrentView():open(true)
        end
    end

    newPage.input = function(self)
        if self.onInput then
            -- call onInput function before rendering the components
            self:onInput()
        end

        if self.currentView then
            self.currentView:input()
        end
    end

    newPage.update = function(self, dt)
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
            local component = self.contents[1]
            -- if component is set to not visible
            if not component then
                goto skip
            end

            if type(component) == "function" then
                component = component() -- dynamic components
            end

            if component.state.visible and not component.state.visible() then
                goto skip
            end
            self.contents[1]:render()
        end

        ::skip::

        if self.currentView then
            self.currentView:update(dt)
        end
    end

    return newPage
end

setmetatable(Page, meta)

-- Add to global UICore table
---@type UIPageConstructor
UICore.Page = Page

return Page
