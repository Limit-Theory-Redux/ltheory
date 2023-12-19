local UIBuilder = class(function(self) end)

function UIBuilder:Init()
    self.windows = {}
end

function UIBuilder:buildWindow(args)
    if not args then
        error("nil ui window arguments")
        return
    end

    local newWindow = {
        guid = guidToKey(GUID.Create()),
        title = args.title,
        group = args.group,
        canClose = args.canClose,
        containers = {},
        page = {}
    }

    newWindow.close = false

    newWindow.containers = args.containers

    newWindow.render = function()
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
                Gui:setAlignment(0.5, 0.5)

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
        end
        Gui:endWindow()
    end

    return newWindow
end

return UIBuilder