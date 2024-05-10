---@type UIPage
local Gameplay = UICore.Page {
    name = "Gameplay"
}

---@type UIRouter
local UIRouter = require("UI.HmGui.UICore.UIRouter")

local ShipCreationView = require("UI.HmGui.Views.Gameplay.ShipCreation")
Gameplay:addViewToPage(ShipCreationView)
local InGameView = require("UI.HmGui.Views.Gameplay.InGame")
Gameplay:addViewToPage(InGameView)
local PausedView = require("UI.HmGui.Views.Gameplay.Paused")
Gameplay:addViewToPage(PausedView)
local SystemMapView = require("UI.HmGui.Views.Gameplay.SystemMap")
Gameplay:addViewToPage(SystemMapView)

function Gameplay:onInput() end
function Gameplay:onUpdate(dt) end
function Gameplay:onPageOpen() end
function Gameplay:onPageClose() end

local buildNumber = UIComponent.Container {
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

Gameplay:addContent(buildNumber)

return Gameplay
