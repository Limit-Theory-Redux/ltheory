---@type UIPage
local MainMenu = UICore.Page {
    name = "Main_Menu"
}

---@type UIRouter
local UIRouter = require("UI.HmGui.UICore.UIRouter")

local TitleView = require("UI.HmGui.Views.MainMenu.Title")
MainMenu:addViewToPage(TitleView)
local MainView = require("UI.HmGui.Views.MainMenu.Main")
MainMenu:addViewToPage(MainView)
local NewgameView = require("UI.HmGui.Views.MainMenu.Newgame")
MainMenu:addViewToPage(NewgameView)
local LoadgameView = require("UI.HmGui.Views.MainMenu.Loadgame")
MainMenu:addViewToPage(LoadgameView)
local SettingsView = require("UI.HmGui.Views.MainMenu.Settings")
MainMenu:addViewToPage(SettingsView)
local BackgroundView = require("UI.HmGui.Views.MainMenu.Background")
MainMenu:addViewToPage(BackgroundView)
local CreditsView = require("UI.HmGui.Views.MainMenu.Credits")
MainMenu:addViewToPage(CreditsView)

function MainMenu:onInput()
    if InputInstance:isPressed(Button.MouseRight) then
        ---@type UIView|nil
        local lastView = MainMenu:getLastView()
        local currentView = MainMenu:getCurrentView()

        if lastView and lastView ~= currentView and currentView.name ~= "Main" then --todo: maybe introduce view hierarchies?
            MainMenu:setView(lastView.name)
        elseif lastView and lastView ~= currentView and currentView.name == "Main" then
            MainMenu:setView("Background")
        end
    end
end

function MainMenu:onPageOpen() end

function MainMenu:onPageClose() end

function MainMenu:onUpdate(dt) end

return MainMenu
