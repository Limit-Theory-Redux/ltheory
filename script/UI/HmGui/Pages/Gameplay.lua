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
local SystemMapView = require("UI.HmGui.Views.Gameplay.SystemMap")
Gameplay:addViewToPage(SystemMapView)

function Gameplay:onInput() end
function Gameplay:onUpdate(dt) end
function Gameplay:onPageOpen() end
function Gameplay:onPageClose() end

return Gameplay
