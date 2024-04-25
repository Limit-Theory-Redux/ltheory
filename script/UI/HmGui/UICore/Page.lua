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
---@field getAvailableViews fun(self: UIPage)
---@field addContent fun(self, component: UIComponent)
---@field input fun(self: UIPage)
---@field update fun(self: UIPage, dt: integer)
---@field onInput fun(self: UIPage)
---@field onUpdate fun(self: UIPage, dt: integer)

---@class UIPageConstructor
---@field name string
---@field views string<UIView>

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

        self.currentView = self.views[viewName]
    end

    newPage.getAvailableViews = function(self)
        local viewNames = {}
        for name, page in pairs(self.views) do
            table.insert(viewNames, name)
        end
        return viewNames
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
            -- if component is set to not visible
            if not self.contents[1] or self.contents[1].state.visible and not self.contents[1].state.visible() then
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
