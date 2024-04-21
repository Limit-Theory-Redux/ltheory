local UIBuilder = class(function(self) end)

function UIBuilder:__init()
    self.pages = {}
    self.windows = {}
    self.currentPage = nil
    self.lastPage = nil

    return self
end

function UIBuilder:render()
    if self.currentPage then
        local page = self:getCurrentPage()

        if page and page.content then
            for id, window in pairs(page.content) do
                if window.close or not window.visible then
                    window.visible = false
                    window.close = false
                    goto skip
                end

                window.render()
                ::skip::
            end
        end
    end
end

-- sets current page
function UIBuilder:setCurrentPage(pageName)
    if not pageName or type(pageName) ~= "string" then
        Log.Error("nil page name or not a string")
    elseif not self.pages[pageName] then
        Log.Error("page does not exist")
    end

    self.lastPage = self.currentPage
    self.currentPage = pageName

    UIBuilder:setWindowsVisibility {
        page = pageName,
        visible = true
    }
end

-- gets current page
function UIBuilder:getCurrentPage()
    if not self.currentPage then
        Log.Error("current page is nil")
    elseif not self.pages[self.currentPage] then
        Log.Error("current page selected does not exist")
    end

    return self.pages[self.currentPage]
end

-- gets current page name as string
function UIBuilder:getCurrentPageName()
    if not self.currentPage then
        Log.Warn("current page is nil")
    elseif not self.pages[self.currentPage] then
        Log.Error("current page selected does not exist")
    end

    return self.currentPage
end

-- gets last page name as string
function UIBuilder:getLastPageName()
    if not self.lastPage then
        self.lastPage = self.currentPage
    elseif not self.pages[self.lastPage] then
        Log.Error("last page selected does not exist")
    end

    return self.lastPage
end

-- gets current page
function UIBuilder:getPage(page)
    if not page then
        Log.Error("get page is nil")
    elseif not self.pages[page] then
        Log.Error("page does not exist")
    end

    return self.pages[page]
end

-- gets all available pages as strings
function UIBuilder:getAvailablePages()
    local pageNames = {}
    for name, page in pairs(self.pages) do
        table.insert(pageNames, name)
    end
    return pageNames
end

-- build a page
function UIBuilder:buildPage(args)
    if not args then
        Log.Error("nil ui page arguments")
        return
    elseif not args.name or type(args.name) ~= "string" then
        Log.Error("nil or faulty ui page name argument")
    end

    self.pages[args.name] = { windowCount = 0, content = {} }
end

-- add a window to a page
function UIBuilder:addWindowToPage(args)
    if not args.page or type(args.page) ~= "string" then
        Log.Error("page identifier nil or not a string")
    elseif type(args.window) ~= "table" then
        Log.Error("argument not a table")
    end

    self.pages[args.page].windowCount = self.pages[args.page].windowCount + 1
    -- assing window count as id
    args.window.id = self.pages[args.page].windowCount
    self.pages[args.page].content[args.window.id] = args.window
end

-- set visibility of a singular window
function UIBuilder:setWindowVisibility(args)
    if not args.page or type(args.page) ~= "string" then
        Log.Error("page identifier nil or not a string")
    elseif type(args.window) ~= "table" then
        Log.Error("argument not a table")
    elseif type(args.visible) ~= "boolean" then
        Log.Error("argument not a boolean")
    end

    self.pages[args.page].content[args.window.id].visible = false
end

-- set visibility of all windows of a page
function UIBuilder:setWindowsVisibility(args)
    if not args.page or type(args.page) ~= "string" then
        Log.Error("page identifier nil or not a string")
    elseif type(args.visible) ~= "boolean" then
        Log.Error("argument not a boolean")
    end

    for id, window in ipairs(self.pages[args.page].content) do
        window.visible = true
    end
end

-- build a window
function UIBuilder:buildWindow(args)
    if not args then
        Log.Error("nil ui window arguments")
        return
    end

    local newWindow = {
        id = nil, -- id assigned via page
        visible = true,
        title = args.title,
        stackDirection = args.group,
        canClose = args.canClose,
        containers = {}
    }

    newWindow.close = false

    newWindow.containers = args.containers

    newWindow.render = function()
        Gui:beginStackContainer()                                      -- begin game window
        Gui:setAlignment(AlignHorizontal.Center, AlignVertical.Center) --! hardcoded to center right now, if we opt for this solution i will add configuration options
        Gui:beginWindow(newWindow.title, InputInstance)
        Gui:pushFont(Cache.Font("Exo2Bold", 12))
        Gui:textColored(newWindow.title, 1, 1, 1, 0.25)

        -- temp until i figure out how to do groups properly
        if newWindow.stackDirection == Enums.UI.StackDirection.X or not newWindow.stackDirection then
            Gui:beginVerticalContainer()
        elseif newWindow.stackDirection == Enums.UI.StackDirection.Y then
            Gui:beginHorizontalContainer()
        end

        for _, container in ipairs(newWindow.containers) do
            local subGroup
            if container.stackDirection == Enums.UI.StackDirection.X then
                Gui:beginVerticalContainer()
                subGroup = true
            elseif container.stackDirection == Enums.UI.StackDirection.Y then
                Gui:beginHorizontalContainer()
                subGroup = true
            end

            -- apply padding
            if container.padding then
                Gui:setPadding(container.padding[1], container.padding[2])
            end

            if container.align then
                Gui:setAlignment(container.align[1], container.align[2])
            end

            -- render content
            for _, element in ipairs(container.contents) do
                -- temp until i figure out how to do groups properly
                if not element.stackDirection or element.stackDirection == Enums.UI.StackDirection.X then
                    Gui:beginVerticalContainer()
                elseif element.stackDirection == Enums.UI.StackDirection.Y then
                    Gui:beginHorizontalContainer()
                end
                Gui:setAlignment(AlignHorizontal.Center, AlignVertical.Center)

                element.render()
                Gui:endContainer()
            end

            if subGroup then
                Gui:endContainer()
            end
        end

        Gui:endContainer()

        if args.canClose then
            if Gui:button("Close") then newWindow.close = true end
            Gui:setAlignment(AlignHorizontal.Center, AlignVertical.Center)
            Gui:setFixedWidth(120)
        end
        Gui:endWindow()
        Gui:endContainer() -- end game window
    end

    return newWindow
end

return UIBuilder:__init()
