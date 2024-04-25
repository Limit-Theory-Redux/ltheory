---@type UIPage
local MainMenu = UICore.Page {
    name = "Main_Menu"
}

local MainView = require("UI.HmGui.Views.MainMenu.Main")
MainMenu:addViewToPage(MainView)
local PlayView = require("UI.HmGui.Views.MainMenu.Play")
MainMenu:addViewToPage(PlayView)
local SettingsView = require("UI.HmGui.Views.MainMenu.Settings")
MainMenu:addViewToPage(SettingsView)
local BackgroundView = require("UI.HmGui.Views.MainMenu.Background")
MainMenu:addViewToPage(BackgroundView)

-- set initial view
MainMenu:setView("Main")

function MainMenu:onInput()
    if InputInstance:isPressed(Button.MouseRight) then
        ---@type UIView|nil
        local lastView = MainMenu:getLastView()
        local currentView = MainMenu:getCurrentView()

        if lastView and lastView ~= currentView and currentView.name ~= "Main" then --todo: maybe introduce view hierarchies?
            MainMenu:setView(lastView.name)
        end
    end
end

function MainMenu:onUpdate(dt) end

return MainMenu
