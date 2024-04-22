local UIBuilder = class(function(self) end)

--[[
* Development Notes:
* - styles should be defined in styles.yaml or in code
* - pages, views and component structure inspired by Vue.js
* - router?
]]

function UIBuilder:__init()
    ---@type table<UIPage>
    self.pages = {}
    ---@type UIPage|nil
    self.currentPage = nil
    ---@type UIPage|nil
    self.lastPage = nil

    return self
end

function UIBuilder:render()
    if self.currentPage then
        if self.currentPage.currentView then
            ---@param id integer
            ---@param view UIComponentView
            for id, view in ipairs(self.currentPage.views) do
                ---@param component UIComponent
                for _, component in ipairs(view.content) do
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
            end
        end
    end
end

-- sets current page
---@param name string
function UIBuilder:setCurrentPage(name)
    if not name or type(name) ~= "string" then
        Log.Error("nil page name or not a string")
    elseif not self.pages[name] then
        Log.Error("page does not exist")
    elseif name == self.currentPage then
        Log.Warn("Already rendering this page: " .. name)
        return
    end

    self.lastPage = self.currentPage
    self.currentPage = self.pages[name]
end

-- gets current page
---@return UIPage
function UIBuilder:getCurrentPage()
    if not self.currentPage then
        Log.Error("current page is nil")
    elseif self.currentPage and not self.pages[self.currentPage.name] then
        Log.Error("current page selected does not exist")
    end

    return self.currentPage
end

-- gets current page name as string
---@return string name
function UIBuilder:getCurrentPageName()
    if not self.currentPage then
        Log.Error("current page is nil")
    elseif self.currentPage and not self.pages[self.currentPage.name] then
        Log.Error("current page selected does not exist")
    end

    return self.currentPage.name
end

-- gets last page name as string
---@return string name
function UIBuilder:getLastPageName()
    if not self.lastPage then
        self.lastPage = self.currentPage
    elseif self.lastPage and not self.pages[self.lastPage.name] then
        Log.Error("current page selected does not exist")
    end

    return self.lastPage.name
end

-- gets current page
---@param name string
---@return UIPage
function UIBuilder:getPage(name)
    if not name then
        Log.Error("page name provided is nil")
    elseif not self.pages[name] then
        Log.Error("page does not exist")
    end

    return self.pages[name]
end

-- gets all available pages as strings
---@return table<string>
function UIBuilder:getAvailablePages()
    local pageNames = {}
    for name, page in pairs(self.pages) do
        table.insert(pageNames, name)
    end
    return pageNames
end

-- add a page
---@param page UIPage
function UIBuilder:addPage(page)
    if not page then
        Log.Error("nil ui page")
        return
    elseif page.name and self.pages[page.name] then
        Log.Error("page with that name already exists")
    end

    self.pages[page.name] = page
end

return UIBuilder:__init()
