---@class UIRouter
local UIRouter = class(function(self) end)

function UIRouter:__init()
    ---@type table<UIPage>
    self.pages = {}
    ---@type UIPage|nil
    self.currentPage = nil
    ---@type UIPage|nil
    self.lastPage = nil

    self:registerEvents()

    return self
end

function UIRouter:registerEvents()
    EventBus:subscribe(EventType.Input, self, self.input)
    EventBus:subscribe(EventType.Render, self, self.update)
end

-- routing the input loop through to the views
-- so it can be used there
---@param dt integer
function UIRouter:input()
    Profiler.Begin('UIRouter.Input')
    if self.currentPage then
        self.currentPage:input()
    end
    Profiler.End()
end

-- routing the update loop through to the views
-- so it can be used there
---@param dt integer
function UIRouter:update(dt)
    Gui:beginGui(GameState.render.resX, GameState.render.resY)
    Profiler.Begin('UIRouter.Update')
    if self.currentPage then
        self.currentPage:update(dt) --! should get delta time from the event, but event bus does not support payloads currently
    end
    Profiler.End()
    Gui:endGui()
end

-- sets current page
---@param name string
---@return UIPage|nil
function UIRouter:setCurrentPage(name)
    if not name or type(name) ~= "string" then
        Log.Error("nil page name or not a string")
        return nil
    elseif not self.pages[name] then
        Log.Error("page does not exist")
        return nil
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
---@return UIPage|nil
function UIRouter:getCurrentPage()
    return self.currentPage
end

-- gets current page name as string
---@return string|nil name
function UIRouter:getCurrentPageName()
    if not self.currentPage then
        return
    end

    return self.currentPage.name
end

-- gets last page name as string
---@return string|nil name
function UIRouter:getLastPageName()
    if not self.lastPage then
        return
    end

    return self.lastPage.name
end

-- gets current page
---@param name string
---@return UIPage|nil
function UIRouter:getPage(name)
    if not name then
        Log.Error("page name provided is nil")
        return nil
    elseif not self.pages[name] then
        Log.Error("page does not exist")
        return nil
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
    if not page or type(page) ~= "table" then
        Log.Error("nil ui page or wrong datatype")
        return
    elseif page.name and self.pages[page.name] then
        Log.Error("page with that name already exists")
        return
    end

    self.pages[page.name] = page
end

return UIRouter:__init()
