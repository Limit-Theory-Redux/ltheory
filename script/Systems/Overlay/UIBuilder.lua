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
        title = args.title,
        group = args.group,
        containers = {},
        page = {}
    }

    newWindow.containers = args.containers

    newWindow.render = function()
        HmGui.BeginWindow(newWindow.title)

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
                HmGui.BeginGroupY()
                HmGui.SetAlign(0.5, 0.5)
                content.render()
                HmGui.EndGroup()
            end

            if subGroup then
                HmGui.EndGroup()
            end
        end

        HmGui.EndGroup()
        HmGui.EndWindow()
    end

    return newWindow
end

return UIBuilder