---@type UIPage
local MainMenu = UICore.Page {
    name = "Main_Menu"
}

---@type UIRouter
local UIRouter = require("UI.HmGui.UICore.UIRouter")

local MainView = require("UI.HmGui.Views.MainMenu.Main")
MainMenu:addViewToPage(MainView)
local PlayView = require("UI.HmGui.Views.MainMenu.Play")
MainMenu:addViewToPage(PlayView)
local SettingsView = require("UI.HmGui.Views.MainMenu.Settings")
MainMenu:addViewToPage(SettingsView)
local BackgroundView = require("UI.HmGui.Views.MainMenu.Background")
MainMenu:addViewToPage(BackgroundView)

function MainMenu:onInput()
    if InputInstance:isPressed(Button.MouseRight) then
        ---@type UIView|nil
        local lastView = MainMenu:getLastView()
        local currentView = MainMenu:getCurrentView()

        -- for testing from example
        --if currentView.name == "Main" then
        --    UIRouter:getPage("Example"):setView("Main")
        --    UIRouter:setCurrentPage("Example")
        --end

        if lastView and lastView ~= currentView and currentView.name ~= "Main" then --todo: maybe introduce view hierarchies?
            MainMenu:setView(lastView.name)
        end
    end
end

function MainMenu:onPageOpen()
    if self:getCurrentView() then
        self:getCurrentView():open(true)
    end
end

function MainMenu:onPageClose()
    if self:getCurrentView() then
        self:getCurrentView():close(true)
    end
end

function MainMenu:onUpdate(dt) end

return MainMenu
