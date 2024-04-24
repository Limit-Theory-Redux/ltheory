---@type UIPage
local Example = UICore.Page {
    name = "Example"
}

local OtherView = require("UI.HmGui.Views.Example.OtherView")
Example:addViewToPage(OtherView)
local MainView = require("UI.HmGui.Views.Example.Main")
Example:addViewToPage(MainView)

function Example:onInput() end
function Example:onUpdate(dt) end

local function switchToMainView()
    Example:setView("Main")
end

local function switchToOtherView()
    Example:setView("Other_View")
end

local viewSelection = UIComponent.Container {
    padding = { 10, 10 },
    align = { AlignHorizontal.Center, AlignVertical.Bottom },
    stackDirection = Enums.UI.StackDirection.Horizontal,
    contents = {
        UIComponent.Button { title = "Main View", callback = switchToMainView, sound = Config.audio.sounds.click },
        UIComponent.Button { title = "Other View", callback = switchToOtherView, sound = Config.audio.sounds.click }
    }
}

Example:addContent(viewSelection)

return Example
