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
local PlayView = require("UI.HmGui.Views.MainMenu.Play")
MainMenu:addViewToPage(PlayView)
local SettingsView = require("UI.HmGui.Views.MainMenu.Settings")
MainMenu:addViewToPage(SettingsView)
local BackgroundView = require("UI.HmGui.Views.MainMenu.Background")
MainMenu:addViewToPage(BackgroundView)
local CreditsView = require("UI.HmGui.Views.MainMenu.Credits")
MainMenu:addViewToPage(CreditsView)

---! debug only
local bgTesting = Tex2D.Load("./res/images/Background-for-testing2.png")

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
        elseif lastView and lastView ~= currentView and currentView.name == "Main" then
            MainMenu:setView("Background")
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

--[[! debug only
local backgroundContainer =
    UIComponent.Container {
        align = { AlignHorizontal.Stretch, AlignVertical.Stretch },
        childrenAlign = { AlignHorizontal.Stretch, AlignVertical.Stretch },
        padding = { 0, 0 },
        margin = { 0, 0 },
        stackDirection = Enums.UI.StackDirection.Vertical,
        contents = {
            UIComponent.RawInput { fn = function()
                Gui:setBorder(0.0001, Color(1.0, 1.0, 1.0, 0.5))
                Gui:image(bgTesting)
                Gui:setPercentSize(100, 100)
            end
            }
        }
    }

MainMenu:addContent(backgroundContainer)]]

return MainMenu
