---@type UIPage
local MainMenu = UICore.Page {
    name = "Main_Menu",
    views = {}
}

local MainView = require("UI.HmGui.Views.MainMenu.Main")
MainMenu:addViewToPage(MainView)

return MainMenu
