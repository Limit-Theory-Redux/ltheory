local UIBuilder = class(function(self) end)

local windowId = 0

function UIBuilder:__init()
    self.pages = {}
    self.windows = {}
    self.currentPage = nil
    self.lastPage = nil

    return self
end

function UIBuilder:update()
    if self.currentPage then
        local page = self:getCurrentPage()

        if page then
            for id, window in pairs(page) do
                if window.close then
                    page[id] = nil
                    goto skip
                end

                window.render()
                ::skip::
            end
        end
    end
end

function UIBuilder:setCurrentPage(pageName)
    if not pageName or type(pageName) ~= "string" then
        Log.Error("nil page name or not a string")
    elseif not self.pages[pageName] then
        Log.Error("page does not exist")
    end

    self.lastPage = self.currentPage
    self.currentPage = pageName
end

function UIBuilder:getCurrentPage()
    if not self.currentPage then
        Log.Error("current page is nil")
    elseif not self.pages[self.currentPage] then
        Log.Error("current page selected does not exist")
    end

    return self.pages[self.currentPage]
end

function UIBuilder:getCurrentPageName()
    if not self.currentPage then
        Log.Warn("current page is nil")
    elseif not self.pages[self.currentPage] then
        Log.Error("current page selected does not exist")
    end

    return self.currentPage
end

function UIBuilder:getLastPageName()
    if not self.lastPage then
        self.lastPage = self.currentPage
    elseif not self.pages[self.lastPage] then
        Log.Error("last page selected does not exist")
    end

    return self.lastPage
end

function UIBuilder:getAvailablePages()
    local pageNames = {}
    for name, page in pairs(self.pages) do
        table.insert(pageNames, name)
    end
    return pageNames
end

function UIBuilder:buildPage(args)
    if not args then
        Log.Error("nil ui page arguments")
        return
    elseif not args.name or type(args.name) ~= "string" then
        Log.Error("nil or faulty ui page name argument")
    end

    self.pages[args.name] = {}
end

function UIBuilder:addWindowToPage(args)
    if not args.page or type(args.page) ~= "string" then
        Log.Error("page identifier nil or not a string")
    elseif type(args.window) ~= "table" then
        Log.Error("argument not a table")
    end

    self.pages[args.page][args.window.id] = args.window
end

function UIBuilder:buildWindow(args)
    if not args then
        Log.Error("nil ui window arguments")
        return
    end

    windowId = windowId + 1

    local newWindow = {
        id = windowId,
        title = args.title,
        group = args.group,
        canClose = args.canClose,
        containers = {},
        page = {}
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
        if newWindow.group == "Y" or not newWindow.group then
            Gui:beginVerticalContainer()
        elseif newWindow.group == "X" then
            Gui:beginHorizontalContainer()
        end

        for _, container in ipairs(newWindow.containers) do
            local subGroup
            if container.group == "X" then
                Gui:beginVerticalContainer()
                subGroup = true
            elseif container.group == "Y" then
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
                if not element.group or element.group == "X" then
                    Gui:beginVerticalContainer()
                elseif element.group == "Y" then
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
