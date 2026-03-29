local PhysicsComponents   = require("Modules.Physics.Components")
local ConstructComponents = require("Modules.Constructs.Components")
local CameraManager       = require("Modules.Cameras.Managers.CameraManager")
local RenderCoreSystem    = require("Modules.Rendering.Systems.RenderCoreSystem")
local UniverseScaleConfig = require("Config.Gen.UniverseScaleConfig")
local ShipFlightSystem    = require("Modules.Constructs.Systems.ShipFlightSystem")
local GravityWellSystem   = require("Modules.Physics.Systems.GravityWellSystem")
local AutoPilotSystem     = require("Modules.Constructs.Systems.AutoPilotSystem")
local DrawEx              = require("UI.DrawEx")

--- GameplayHUDSystem — renders flight info, drive status, zone, and controls overlay.
---@class GameplayHUDSystem
local GameplayHUDSystem = {}

--- Draw the gameplay HUD for a player ship
---@param shipEntity Entity
---@param modeName string Camera mode name
---@param piloting boolean Whether player is piloting
function GameplayHUDSystem:draw(shipEntity, modeName, piloting)
    if not shipEntity then return end

    local y = 40
    local lineHeight = 22

    local speed = ShipFlightSystem:getSpeed(shipEntity)

    local speedStr = UniverseScaleConfig:formatSpeed(speed)

    -- Travel drive status
    local drive = shipEntity:get(ConstructComponents.TravelDrive)
    local driveStr = ""
    if drive then
        local driveState = drive:getState()
        if driveState == "charging" then
            driveStr = string.format(" | Drive: CHARGING %.0f%%", drive:getChargeProgress() * 100)
        elseif driveState == "active" then
            driveStr = string.format(" | Drive: ACTIVE x%.0f", drive:getCurrentMult())
        elseif driveState == "decelerating" then
            driveStr = " | Drive: DECELERATING"
        end
    end

    local zoneName = GravityWellSystem:getZoneName()
    local maxDriveSpeed = GravityWellSystem:getMaxDriveSpeed()

    local apActive = AutoPilotSystem:isActive(shipEntity)
    local apName   = AutoPilotSystem:getTargetName(shipEntity)

    local infoLines = {
        string.format("FPS: %d", RenderCoreSystem:getSmoothFPS()),
        string.format("Frametime: %.2f ms", RenderCoreSystem:getSmoothFrameTime(true)),
        string.format("Speed: %s%s", speedStr, driveStr),
        string.format("Zone: %s | Max Drive: x%d", zoneName, maxDriveSpeed),
        apActive
            and string.format("AUTOPILOT: %s [N to cancel]", apName)
            or  string.format("Camera: %s %s", modeName, piloting and "(piloting)" or "(observer)"),
    }

    for _, line in ipairs(infoLines) do
        DrawEx.TextAdditive('Unageo-Medium', line, 11, 40, y, 400, 20, 0.9, 0.9, 0.9, 0.9, 0.0, 0.5)
        y = y + lineHeight
    end

    y = y + 10
    local controls = {
        "WASD - Thrust | Q/E - Roll | Shift - Boost | T - Travel Drive",
        "M - Map | N - Navigate (map) / Cancel | C - Camera",
    }
    for _, line in ipairs(controls) do
        DrawEx.TextAdditive('Unageo-Medium', line, 9, 40, y, 400, 16, 0.6, 0.6, 0.6, 0.7, 0.0, 0.5)
        y = y + 18
    end
end

return GameplayHUDSystem
