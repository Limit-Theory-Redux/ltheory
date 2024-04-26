---@type UIPage
local Example = UICore.Page {
    name = "Example"
}

---@type UIRouter
local UIRouter = require("UI.HmGui.UICore.UIRouter")

local OtherView = require("UI.HmGui.Views.Example.OtherView")
Example:addViewToPage(OtherView)
local MainView = require("UI.HmGui.Views.Example.Main")
Example:addViewToPage(MainView)

function Example:onInput() end
function Example:onUpdate(dt) end
function Example:onPageOpen() end
function Example:onPageClose() end

local function switchToMainView()
    Example:setView("Main")
end

local function switchToOtherView()
    Example:setView("Other_View")
end

local function switchToMainMenu()
    UIRouter:getPage("Main_Menu"):setView("Title")
    UIRouter:setCurrentPage("Main_Menu")
end

local viewSelection = UIComponent.Container {
    padding = { 10, 50 },
    align = { AlignHorizontal.Center, AlignVertical.Bottom },
    stackDirection = Enums.UI.StackDirection.Horizontal,
    contents = {
        UIComponent.Button { title = "Main View", width = 100, callback = switchToMainView, sound = Config.audio.sounds.click },
        UIComponent.Spacer { size = 24 },
        UIComponent.Button { title = "Other View", width = 100, callback = switchToOtherView, sound = Config.audio.sounds.click },
        UIComponent.Spacer { size = 24 },
        UIComponent.Button { title = "Hidden View", width = 100, sound = Config.audio.sounds.click, visible = false },
        UIComponent.Spacer { size = 24 },
        UIComponent.Button { title = "Main Menu", width = 100, callback = switchToMainMenu, sound = Config.audio.sounds.click, color = {
            text = Color(1, 1, .4, 1)
        } }
    }
}

Example:addContent(viewSelection)

return Example
