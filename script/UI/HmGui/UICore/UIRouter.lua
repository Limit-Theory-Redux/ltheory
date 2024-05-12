---@class UIRouter
local UIRouter = class(function(self) end)

--[[
* Development Notes:
* - styles should be defined in styles.yaml or in code
* - pages, views and component structure inspired by Vue.js
* - router?
]]

function UIRouter:__init()
    ---@type table<UIPage>
    self.pages = {}
    ---@type UIPage|nil
    self.currentPage = nil
    ---@type UIPage|nil
    self.lastPage = nil

    return self
end

-- routing the input loop through to the views
-- so it can be used there
---@param dt integer
function UIRouter:input()
    if self.currentPage then
        self.currentPage:input()
    end
end

-- routing the update loop through to the views
-- so it can be used there
---@param dt integer
function UIRouter:update(dt)
    if self.currentPage then
        self.currentPage:update(dt)
    end
end

-- sets current page
---@param name string
---@return UIPage
function UIRouter:setCurrentPage(name)
    if not name or type(name) ~= "string" then
        Log.Error("nil page name or not a string")
    elseif not self.pages[name] then
        Log.Error("page does not exist")
    elseif name == self.currentPage then
        Log.Warn("Already rendering this page: " .. name)
        return self.currentPage
    end

    self.lastPage = self.currentPage

    if self.lastPage then
        self.lastPage:close()
    end

    self.currentPage = self.pages[name]
    self.currentPage:open()
    return self.currentPage
end

-- resets current page to nil
function UIRouter:clearCurrentPage()
    self.lastPage = self.currentPage

    if self.currentPage then
        self.currentPage:close()
    end
    self.currentPage = nil
end

-- gets current page
---@return UIPage
function UIRouter:getCurrentPage()
    if not self.currentPage then
        Log.Error("current page is nil")
    elseif self.currentPage and not self.pages[self.currentPage.name] then
        Log.Error("current page selected does not exist")
    end

    return self.currentPage
end

-- gets current page name as string
---@return string name
function UIRouter:getCurrentPageName()
    if not self.currentPage then
        Log.Error("current page is nil")
    elseif self.currentPage and not self.pages[self.currentPage.name] then
        Log.Error("current page selected does not exist")
    end

    return self.currentPage.name
end

-- gets last page name as string
---@return string name
function UIRouter:getLastPageName()
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
function UIRouter:getPage(name)
    if not name then
        Log.Error("page name provided is nil")
    elseif not self.pages[name] then
        Log.Error("page does not exist")
    end

    return self.pages[name]
end

-- gets all available pages as strings
---@return table<string>
function UIRouter:getAvailablePages()
    local pageNames = {}
    for name, page in pairs(self.pages) do
        table.insert(pageNames, name)
    end
    return pageNames
end

-- add a page
---@param page UIPage
function UIRouter:addPage(page)
    if not page then
        Log.Error("nil ui page")
        return
    elseif page.name and self.pages[page.name] then
        Log.Error("page with that name already exists")
    end

    self.pages[page.name] = page
end

return UIRouter:__init()
