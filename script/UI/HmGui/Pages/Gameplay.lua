---@type UIPage
local Gameplay = UICore.Page {
    name = "Gameplay"
}

---@type UIRouter
local UIRouter = require("UI.HmGui.UICore.UIRouter")
---@type ResponsiveSize
local ResponsiveSize = require("Types.ResponsiveSize")

local ShipCreationView = require("UI.HmGui.Views.Gameplay.ShipCreation")
Gameplay:addViewToPage(ShipCreationView)
local InGameView = require("UI.HmGui.Views.Gameplay.InGame")
Gameplay:addViewToPage(InGameView)
local PausedView = require("UI.HmGui.Views.Gameplay.Paused")
Gameplay:addViewToPage(PausedView)
local SystemMapView = require("UI.HmGui.Views.Gameplay.SystemMap")
Gameplay:addViewToPage(SystemMapView)

local deathTextVisible = false

function Gameplay:onInput() end

function Gameplay:onUpdate(dt)
    if not deathTextVisible and GameState.player.currentShip and GameState.player.currentShip:isDestroyed() then
        deathTextVisible = true
        Input:setCursorVisible(true)
    end
end

function Gameplay:onPageOpen()
    deathTextVisible = false
end

function Gameplay:onPageClose() end

local buildNumber = UIComponent.Container {
    visible = function() return not GameState.debug.metricsEnabled end,
    align = { AlignHorizontal.Stretch, AlignVertical.Stretch },
    childrenAlign = { AlignHorizontal.Left, AlignVertical.Bottom },
    padding = { 5, 5 },
    contents = {
        UIComponent.Text {
            text = "Build " .. Config.gameVersion,
            font = "Unageo-Regular",
            size = 12,
            color = Color(0.75, 0.75, 0.75, 0.75)
        }
    }
}

local deathText = UIComponent.Container {
    visible = function() return deathTextVisible end,
    align = { AlignHorizontal.Stretch, AlignVertical.Stretch },
    childrenAlign = { AlignHorizontal.Center, AlignVertical.Center },
    layoutType = GuiLayoutType.Vertical,
    contents = {
        UIComponent.Text {
            text = "[GAME OVER]",
            font = "Unageo-Semibold",
            size = 32,
            color = Color(1, 1, 1, 1)
        },
        UIComponent.Button_MainMenu {
            title = "Back to Main Menu",
            size = ResponsiveSize(300, 60),
            margin = { 0, 40 },
            font = { name = "Unageo-Medium", size = 24 },
            align = { AlignHorizontal.Center, AlignVertical.Center },
            callback = function() LimitTheoryRedux:initMainMenu() end
        },
    }
}

Gameplay:addContent(buildNumber)
Gameplay:addContent(deathText)

return Gameplay
