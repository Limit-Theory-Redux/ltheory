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
        HmGui.BeginWindow(newWindow.title)
        HmGui.PushFont(Cache.Font("Exo2Bold", 12))
        HmGui.TextColored(newWindow.title, 1, 1, 1, 0.25)

        -- temp until i figure out how to do groups properly
        if newWindow.group == "Y" or not newWindow.group then
            HmGui.BeginGroupY()
        elseif newWindow.group == "X" then
            HmGui.BeginGroupX()
        end

        for _, container in ipairs(newWindow.containers) do
            local subGroup
            if container.group == "X" then
                HmGui.BeginGroupX()
                subGroup = true
            elseif container.group == "Y" then
                HmGui.BeginGroupY()
                subGroup = true
            end

            -- apply padding
            if container.padding then
                HmGui.SetPadding(container.padding[1], container.padding[2])
            end

            if container.align then
                HmGui.SetAlign(container.align[1], container.align[2])
            end

            -- render content
            for _, content in ipairs(container) do
                -- temp until i figure out how to do groups properly
                if not content.group or content.group == "X" then
                    HmGui.BeginGroupX()
                elseif content.group == "Y" then
                    HmGui.BeginGroupY()
                end
                HmGui.SetAlign(0.5, 0.5)

                content.render()
                HmGui.EndGroup()
            end

            if subGroup then
                HmGui.EndGroup()
            end
        end

        HmGui.EndGroup()

        if args.canClose then
            if HmGui.Button("Close") then newWindow.close = true end
        end
        HmGui.EndWindow()
    end

    return newWindow
end

return UIBuilder