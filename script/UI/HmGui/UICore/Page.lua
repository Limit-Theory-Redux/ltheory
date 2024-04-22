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
---@field render fun(self: UIPage)

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
    newPage.views = args.views
    newPage.currentView = nil

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

    newPage.render = function(self)
        if self.currentView then
            self.currentView:render()
        end
    end

    return newPage
end

setmetatable(Page, meta)

-- Add to global UICore table
---@type UIPageConstructor
UICore.Page = Page

return Page
