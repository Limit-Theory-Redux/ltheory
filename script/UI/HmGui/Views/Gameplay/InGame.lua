---@type UIView
local InGame = UICore.View {
    name = "In_Game",
    noDarken = true -- in-game overlay: never dim the game view
}

local ShipFlightSystem    = require("Modules.Constructs.Systems.ShipFlightSystem")
local AutoPilotSystem     = require("Modules.Constructs.Systems.AutoPilotSystem")
local GravityWellSystem   = require("Modules.Physics.Systems.GravityWellSystem")
local ConstructComponents = require("Modules.Constructs.Components")
local UniverseScaleConfig = require("Config.Gen.UniverseScaleConfig")

-- NOTE: in-game input (pause, map, camera, autopilot) is owned by
-- LimitTheoryRedux:onInput and the PlayerController; the In_Game view only
-- draws.

-- Set by LimitTheoryRedux: the player's controller (ship and camera mode), and
-- whether the HUD is shown (hidden by the HUD toggle and in the 3D map).
---@type PlayerController|nil
InGame.player = nil
InGame.hudShown = false
-- Set by LimitTheoryRedux: the running scenario's text (Scenario.overlayLines).
---@type (fun(): string[]|nil)|nil
InGame.overlayLines = nil
-- Set by LimitTheoryRedux: the running scenario's own HUD components (Scenario.hud).
---@type UIComponent[]|nil
InGame.scenarioHud = nil

local NO_LINES = {}

--- What the HUD shows this frame; filled in by onUpdate, read by the components.
local hud = {
    driveText = "", driveAlpha = 0,
    autopilotText = "",
    zoneText = "",
    overlayLines = NO_LINES,
    cameraText = "", cameraAlpha = 0,
}

local function hudVisible()
    return InGame.hudShown and InGame.player ~= nil
end

------------------------------------------------------------------------------
-- Speed tape, curved around the reticle on its left (Components/Hud/SpeedTape).

-- ShipFlightSystem multiplies the thrust by 1 + 2 x boost; with linear drag the
-- top speed scales with the thrust, so boosting triples it.
local BOOST_SPEED_FACTOR = 3

-- Sized like the rest of the HUD: to the window height, times the UI Scale setting.
local speedTape = UIComponent.SpeedTape {
    visible = hudVisible,
    scale = function() return Window:height() / 1080 * GameState.ui.hudScale end,
}

local function updateSpeedTape(ship, dt)
    local drive = ship:get(ConstructComponents.TravelDrive)
    local state = drive and drive:getState()
    local maxSpeed = Config.game.shipFlight.maxSpeedGame
    speedTape:update({
        speed = ShipFlightSystem:getSpeed(ship),
        maxSpeed = maxSpeed,
        boostSpeed = maxSpeed * BOOST_SPEED_FACTOR,
        travelSpeed = maxSpeed * GravityWellSystem:getMaxDriveSpeed(),
        travel = state == "active" or state == "decelerating",
    }, dt)
end

------------------------------------------------------------------------------
-- Travel drive text: blinks while charging (faster as it nears 100%), steady
-- while active, fades out when the drive stops.

local BLINK_HZ_START, BLINK_HZ_FULL = 1.0, 4.0
local DRIVE_FADE_TIME = 0.5 -- seconds

local blinkPhase, driveFadeLeft = 0, 0

local function updateDrive(ship, dt)
    local drive = ship:get(ConstructComponents.TravelDrive)
    local state = drive and drive:getState() or "idle"
    if state == "charging" then
        local progress = math.min(1, drive:getChargeProgress())
        blinkPhase = (blinkPhase + dt * (BLINK_HZ_START + (BLINK_HZ_FULL - BLINK_HZ_START) * progress)) % 1
        hud.driveText = string.format("TRAVEL DRIVE CHARGING  %d%%", math.floor(progress * 100))
        hud.driveAlpha = 0.35 + 0.65 * (0.5 + 0.5 * math.cos(blinkPhase * 2 * math.pi))
        driveFadeLeft = 0
    elseif state == "active" then
        hud.driveText = "TRAVEL DRIVE ACTIVE"
        hud.driveAlpha = 1
        driveFadeLeft = DRIVE_FADE_TIME
    elseif driveFadeLeft > 0 then -- just left travel mode
        driveFadeLeft = math.max(0, driveFadeLeft - dt)
        hud.driveText = "TRAVEL DRIVE ACTIVE"
        hud.driveAlpha = driveFadeLeft / DRIVE_FADE_TIME
    else
        hud.driveText = ""
        hud.driveAlpha = 0
    end
end

------------------------------------------------------------------------------
-- Autopilot status, above the travel drive text.

--- "PlanetEntity(41)" -> "Planet", "SpaceStationEntity(3)" -> "Space station".
local function targetName(name)
    if name == "Position" then return "Selected point" end
    local base = name:match("^(%a-)Entity")
    if not base then return name end
    local words = base:gsub("(%l)(%u)", "%1 %2")
    return (words:sub(1, 1) .. words:sub(2):lower())
end

local function etaText(seconds)
    if seconds > 3600 then
        return string.format("  ·  ETA %dh %02dm", math.floor(seconds / 3600), math.floor(seconds % 3600 / 60))
    elseif seconds > 60 then
        return string.format("  ·  ETA %dm %02ds", math.floor(seconds / 60), math.floor(seconds % 60))
    elseif seconds > 0 then
        return string.format("  ·  ETA %ds", math.floor(seconds))
    end
    return ""
end

local function updateAutopilot(ship)
    if not AutoPilotSystem:isActive(ship) then
        hud.autopilotText = ""
        return
    end
    local cmp = ship:get(ConstructComponents.AutoPilot)
    hud.autopilotText = string.format("AUTOPILOT  ›  %s  ·  %s%s  ·  N to cancel",
        targetName(AutoPilotSystem:getTargetName(ship)),
        UniverseScaleConfig:formatDistance(cmp and cmp.distance or 0),
        etaText(cmp and cmp.eta or 0))
end

------------------------------------------------------------------------------
-- Zone and travel-drive cap, top left (below where System / Location will go
-- once systems and bodies have names).

local ZONE_NAMES = { planet = "Planet", moon = "Moon", star = "Star", openSpace = "Open space" }

--- 2005 -> "2,005"
local function thousands(n)
    local s = tostring(math.floor(n + 0.5))
    local grouped = s:reverse():gsub("(%d%d%d)", "%1,"):reverse()
    return (grouped:gsub("^,", ""))
end

local function updateZone()
    local wells = GravityWellSystem
    hud.zoneText = string.format("Zone: %s  ·  Travel cap ×%s",
        ZONE_NAMES[wells.currentZoneType] or wells.currentZoneType, thousands(wells:getMaxDriveSpeed()))
end

------------------------------------------------------------------------------
-- Camera mode: a label in the cameras that don't fly the ship (orbit, free),
-- and a short message on every camera switch.

local CAMERA_NAMES = { FirstPerson = "First person", Chase = "Chase", Orbit = "Orbit", Free = "Free", RTS = "RTS" }
local SWITCH_MESSAGE_TIME, SWITCH_FADE_TIME = 2.0, 0.5 -- seconds

local lastCameraMode, switchMessageLeft = nil, 0

local function updateCamera(player, dt)
    local mode = player:getMode()
    if lastCameraMode ~= nil and mode ~= lastCameraMode then
        switchMessageLeft = SWITCH_MESSAGE_TIME
    end
    lastCameraMode = mode

    local name = CAMERA_NAMES[player:getModeName()] or player:getModeName()
    if not player:isPiloting() then
        hud.cameraText = string.format("%s CAMERA  ·  ship not controlled  ·  C to switch", name:upper())
        hud.cameraAlpha = 1
    elseif switchMessageLeft > 0 then
        switchMessageLeft = math.max(0, switchMessageLeft - dt)
        hud.cameraText = name .. " camera"
        hud.cameraAlpha = math.min(1, switchMessageLeft / SWITCH_FADE_TIME)
    else
        hud.cameraText = ""
        hud.cameraAlpha = 0
    end
end

------------------------------------------------------------------------------

---@param player PlayerController|nil
function InGame:setPlayer(player)
    self.player = player
    lastCameraMode, switchMessageLeft = nil, 0 -- no "camera switched" message for a new game
end

---@param shown boolean
function InGame:setHudShown(shown)
    self.hudShown = shown
end

---@param overlayLines (fun(): string[]|nil)|nil the scenario's text, a list of lines; nil = none
function InGame:setOverlayLines(overlayLines)
    self.overlayLines = overlayLines
    hud.overlayLines = NO_LINES
end

---@param components UIComponent[]|nil the scenario's own HUD components; nil = none
function InGame:setScenarioHud(components)
    self.scenarioHud = components
end

function InGame:onInput() end

-- Views get no frame time from the UI router, so the HUD keeps its own clock.
local clock, lastTime

function InGame:onUpdate()
    clock = clock or TimeStamp.Now()
    local now = clock:getElapsed()
    local dt = lastTime and math.min(0.1, now - lastTime) or 0
    lastTime = now

    local player = self.player
    local ship = player and player:getShip()
    if not ship then return end

    updateSpeedTape(ship, dt)
    updateDrive(ship, dt)
    updateAutopilot(ship)
    updateZone()
    hud.overlayLines = self.overlayLines and self.overlayLines() or NO_LINES
    updateCamera(player, dt)
end

function InGame:onViewOpen(isPageOpen)
    GameState:SetState(Enums.GameStates.InGame)
end

------------------------------------------------------------------------------
-- Components. Each is a screen-filling container (a UIComponent.Container
-- fills its parent by default) that places its text with alignment and
-- padding. Every text here is outlined (a halo, plus a 1 px outline from 24 px),
-- so it stays readable in front of a sun or a bright nebula. (The speed tape
-- draws its own text, its tag on a dark ground.)

--- The HUD's UI scale (each container's `scale`): its sizes are for a 1080p
--- window and grow and shrink with the window height, within 0.7 and 1.3,
--- times the player's UI Scale setting (Settings > Interface, 100 % by default).
local function hudScale()
    return math.max(0.7, math.min(1.3, Window:height() / 1080)) * GameState.ui.hudScale
end

local AUTOPILOT_COLOR = Color(0.51, 0.92, 1.00, 1.0)
local ZONE_COLOR = Color(0.35, 0.70, 1.00, 1.0)

-- Centre, slightly above: autopilot status over the travel drive text. The
-- autopilot line keeps a fixed height, so the drive text never moves.
local travelBlock = UIComponent.Container {
    visible = hudVisible,
    align = { AlignHorizontal.Stretch, AlignVertical.Stretch },
    layoutType = GuiLayoutType.Vertical,
    childrenAlign = { AlignHorizontal.Center, AlignVertical.Top },
    scale = hudScale,
    -- 30 % of the window height down to the drive text (padding is in HUD pixels)
    padding = function() return { 0, Window:height() * 0.30 / hudScale() - 34 - 28 } end,
    contents = {
        UIComponent.Text {
            text = function() return hud.autopilotText end,
            font = "Unageo-Medium",
            size = 24,
            height = 34,
            color = AUTOPILOT_COLOR,
            outline = true,
        },
        UIComponent.Text {
            visible = function() return hud.driveText ~= "" end,
            text = function() return hud.driveText end,
            font = "Unageo-Semibold",
            size = 46,
            color = function() return Color(1.0, 1.0, 1.0, hud.driveAlpha) end,
            outline = true,
        },
    },
}

local zoneLine = UIComponent.Container {
    visible = hudVisible,
    align = { AlignHorizontal.Stretch, AlignVertical.Stretch },
    layoutType = GuiLayoutType.Vertical,
    childrenAlign = { AlignHorizontal.Left, AlignVertical.Top },
    scale = hudScale,
    padding = { 40, 30 },
    contents = {
        UIComponent.Text {
            text = function() return hud.zoneText end,
            font = "UbuntuBold",
            size = 18,
            color = ZONE_COLOR,
            outline = true,
        },
    },
}

-- The scenario's text (what it shows, its keys), under the zone line. A text
-- per line: Gui:text doesn't break lines.
local OVERLAY_COLOR = Color(0.85, 0.90, 1.00, 0.9)
local MAX_OVERLAY_LINES = 12 -- the Physics Test shows up to 10

local overlayTexts = {}
for i = 1, MAX_OVERLAY_LINES do
    overlayTexts[i] = UIComponent.Text {
        visible = function() return hud.overlayLines[i] ~= nil end,
        text = function() return hud.overlayLines[i] or "" end,
        font = "Unageo-Medium",
        size = 15,
        height = 24,
        color = OVERLAY_COLOR,
        outline = true,
    }
end

local scenarioText = UIComponent.Container {
    visible = function() return hudVisible() and #hud.overlayLines > 0 end,
    align = { AlignHorizontal.Stretch, AlignVertical.Stretch },
    layoutType = GuiLayoutType.Vertical,
    childrenAlign = { AlignHorizontal.Left, AlignVertical.Top },
    scale = hudScale,
    padding = { 40, 80 },
    contents = overlayTexts,
}

local cameraLabel = UIComponent.Container {
    visible = function() return hudVisible() and hud.cameraText ~= "" end,
    align = { AlignHorizontal.Stretch, AlignVertical.Stretch },
    childrenAlign = { AlignHorizontal.Center, AlignVertical.Top },
    scale = hudScale,
    padding = { 0, 30 },
    contents = {
        UIComponent.Text {
            text = function() return hud.cameraText end,
            font = "Unageo-Medium",
            size = 20,
            color = function() return Color(0.80, 0.88, 0.97, hud.cameraAlpha) end,
            outline = true,
        },
    },
}

-- Centre of the screen while the game is paused with P. (The pause menu
-- replaces this view, so it never shows there.)
local pausedText = UIComponent.Container {
    visible = function() return hudVisible() and GameState.paused end,
    scale = hudScale,
    align = { AlignHorizontal.Stretch, AlignVertical.Stretch },
    childrenAlign = { AlignHorizontal.Center, AlignVertical.Center },
    contents = {
        UIComponent.Text {
            text = "PAUSED",
            font = "Unageo-Semibold",
            size = 46,
            color = Color(1.0, 1.0, 1.0, 1.0),
            outline = true,
        },
    },
}

-- The running scenario's own HUD (Scenario.hud), drawn after the speed tape:
-- each of its components draws while it is visible.
local scenarioHud = {
    state = UICore.ComponentState {
        visible = function() return hudVisible() and InGame.scenarioHud ~= nil end,
    },
}
function scenarioHud:render()
    for _, component in ipairs(InGame.scenarioHud) do
        if not (component.state.visible and not component.state.visible()) then
            component:render()
        end
    end
end

InGame:addContent(speedTape)
InGame:addContent(scenarioHud)
InGame:addContent(travelBlock)
InGame:addContent(zoneLine)
InGame:addContent(scenarioText)
InGame:addContent(cameraLabel)
InGame:addContent(pausedText)

return InGame
