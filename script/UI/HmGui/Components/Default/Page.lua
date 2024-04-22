local Page = {}
Page.__index = Page

local meta = {
    __call = function(cls, ...)
        return cls:new(...)
    end
}

---@class UIComponent
---@field render function|nil

---@class UIComponentPage: UIComponent
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

    --todo: add set view function

    return newPage
end

setmetatable(Page, meta)

-- Add to global UIComponent table
---@type UIComponentPageConstructor
UIComponent.Page = Page

return Page
