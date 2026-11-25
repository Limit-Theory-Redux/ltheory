---@type UIPage
local Gameplay = UICore.Page {
    name = "Gameplay"
}

---@type UIRouter
local UIRouter = require("UI.HmGui.UICore.UIRouter")
---@type ResponsiveSize
local ResponsiveSize = require("Types.ResponsiveSize")
local CoreComponents = require("Modules.Core.Components")
local CursorManager = require("Input.CursorManager")

local ShipCreationView = require("UI.HmGui.Views.Gameplay.ShipCreation")
Gameplay:addViewToPage(ShipCreationView)
local InGameView = require("UI.HmGui.Views.Gameplay.InGame")
Gameplay:addViewToPage(InGameView)
local PausedView = require("UI.HmGui.Views.Gameplay.Paused")
Gameplay:addViewToPage(PausedView)
local ControlsView = require("UI.HmGui.Views.Gameplay.Controls")
Gameplay:addViewToPage(ControlsView)
local SystemMapView = require("UI.HmGui.Views.Gameplay.SystemMap")
Gameplay:addViewToPage(SystemMapView)

local deathTextVisible = false

---@return boolean destroyed the player's ship exists and has no health left
local function isPlayerShipDestroyed()
    local ship = LimitTheoryRedux.playerShip
    if not ship or not ship:isValid() then
        return false
    end
    local health = ship:get(CoreComponents.Health)
    return health ~= nil and health:isDestroyed()
end

function Gameplay:onInput() end

-- Checked every frame rather than latched: the player's ship can be replaced
-- by a new one while the page stays open.
function Gameplay:onUpdate(dt)
    local destroyed = isPlayerShipDestroyed()
    if destroyed and not deathTextVisible then
        -- Visible and not grabbed: a locked cursor can't reach the button
        CursorManager:free()
    end
    deathTextVisible = destroyed
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
            callback = function() LimitTheoryRedux:returnToMainMenu() end
        },
    }
}

Gameplay:addContent(buildNumber)
Gameplay:addContent(deathText)

return Gameplay
