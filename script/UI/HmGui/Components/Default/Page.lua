local Page = {}
Page.__index = Page

local meta = {
    __call = function(cls, ...)
        return cls:new(...)
    end
}

---@class UIComponentPage
---@field name string
---@field views string<UIComponentView>
---@field currentView integer

---@class UIComponentPageConstructor
---@field name string
---@field views string<UIComponentView>

---returns a page object
---@param args UIComponentPageConstructor
---@return UIComponentPage|nil
function Page:new(args)
    if not args then
        return
    end

    local newPage = {}
    newPage.name = args.name
    newPage.views = args.views
    newPage.currentView = 1 -- start with first view

    return newPage
end

setmetatable(Page, meta)

-- Add to global UIComponent table
---@type UIComponentPageConstructor
UIComponent.Page = Page

return Page
