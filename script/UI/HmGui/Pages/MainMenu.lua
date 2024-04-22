---@type UIPage
local MainMenu = UICore.Page {
    name = "Main_Menu",
    views = {}
}

local TitleScreen = require("UI.HmGui.Views.MainMenu.TitleScreen")
MainMenu:addViewToPage(TitleScreen)
local MainView = require("UI.HmGui.Views.MainMenu.Main")
MainMenu:addViewToPage(MainView)

return MainMenu
