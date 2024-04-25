---@type UIPage
local MainMenu = UICore.Page {
    name = "Main_Menu"
}

local MainView = require("UI.HmGui.Views.MainMenu.Main")
MainMenu:addViewToPage(MainView)
local SettingsView = require("UI.HmGui.Views.MainMenu.Settings")
MainMenu:addViewToPage(SettingsView)
local BackgroundView = require("UI.HmGui.Views.MainMenu.Background")
MainMenu:addViewToPage(BackgroundView)

-- set initial view
MainMenu:setView("Main")

function MainMenu:onInput() end
function MainMenu:onUpdate(dt) end

return MainMenu
