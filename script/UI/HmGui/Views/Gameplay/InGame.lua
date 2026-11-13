---@type UIView
local InGame = UICore.View {
    name = "In_Game",
    noDarken = true -- in-game overlay: never dim the game view
}

-- NOTE: in-game input (pause, map, camera, autopilot) is owned by
-- LimitTheoryRedux:onInput and the PlayerController; the In_Game view only
-- draws.

-- Set by LimitTheoryRedux: the player's controller (ship and camera mode), and
-- whether the HUD is shown (hidden by the HUD toggle and in the 3D map).
---@type PlayerController|nil
InGame.player = nil
InGame.hudShown = false

-- Systems the HUD reads, required on first use: views load at startup, before
-- the game's modules.
local deps
local function systems()
    if not deps then
        deps = {
            flight     = require("Modules.Constructs.Systems.ShipFlightSystem"),
            autoPilot  = require("Modules.Constructs.Systems.AutoPilotSystem"),
            wells      = require("Modules.Physics.Systems.GravityWellSystem"),
            constructs = require("Modules.Constructs.Components"),
            scale      = require("Config.Gen.UniverseScaleConfig"),
        }
    end
    return deps
end

--- What the HUD shows this frame; filled in by onUpdate, read by the components.
local hud = {
    speedText = "", speedColor = Color(1, 1, 1, 1),
    driveText = "", driveAlpha = 0,
    autopilotText = "",
    zoneText = "",
    cameraText = "", cameraAlpha = 0,
}

------------------------------------------------------------------------------
-- Speed readout: the number the speed tape will be built around. Its colour
-- follows the speed state and blends between states.

local STOPPED = { 0.69, 0.71, 0.75 } -- dim grey-white
local SLOW    = { 0.12, 0.41, 0.84 } -- deep blue at low speed ...
local AT_MAX  = { 0.43, 0.80, 1.00 } -- ... to light blue at max normal speed
local BOOST   = { 0.16, 0.94, 1.00 } -- full cyan
local TRAVEL  = { 0.96, 0.98, 1.00 } -- bright white
local SPEED_TEXT_TINT = 0.35         -- text = state colour mixed this much towards white, so it stays readable
local COLOR_BLEND_TIME = 0.1         -- seconds; time constant of the colour blend (~0.3 s to settle)

local speedColor = { 1, 1, 1 }

local function mix(a, b, t)
    return { a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t, a[3] + (b[3] - a[3]) * t }
end

---@return number[]
local function speedStateColor(ship, speed)
    local drive = ship:get(systems().constructs.TravelDrive)
    local state = drive and drive:getState()
    if state == "active" or state == "decelerating" then return TRAVEL end
    local max = Config.game.shipFlight.maxSpeedGame
    if speed > max * 1.02 then return BOOST end
    if speed / systems().scale.globalScale < 1 then return STOPPED end -- below 1 m/s
    return mix(SLOW, AT_MAX, math.min(1, speed / max))
end

local function updateSpeed(ship, dt)
    local speed = systems().flight:getSpeed(ship)
    speedColor = mix(speedColor, speedStateColor(ship, speed), 1 - math.exp(-dt / COLOR_BLEND_TIME))
    local c = mix(speedColor, { 1, 1, 1 }, SPEED_TEXT_TINT)
    hud.speedColor = Color(c[1], c[2], c[3], 1)
    hud.speedText = systems().scale:formatSpeed(speed)
end

------------------------------------------------------------------------------
-- Travel drive text: blinks while charging (faster as it nears 100%), steady
-- while active, fades out when the drive stops.

local BLINK_HZ_START, BLINK_HZ_FULL = 1.0, 4.0
local DRIVE_FADE_TIME = 0.5 -- seconds

local blinkPhase, driveFadeLeft = 0, 0

local function updateDrive(ship, dt)
    local drive = ship:get(systems().constructs.TravelDrive)
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
    local autoPilot = systems().autoPilot
    if not autoPilot:isActive(ship) then
        hud.autopilotText = ""
        return
    end
    local cmp = ship:get(systems().constructs.AutoPilot)
    hud.autopilotText = string.format("AUTOPILOT  ›  %s  ·  %s%s  ·  N to cancel",
        targetName(autoPilot:getTargetName(ship)),
        systems().scale:formatDistance(cmp and cmp.distance or 0),
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
    local wells = systems().wells
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

    updateSpeed(ship, dt)
    updateDrive(ship, dt)
    updateAutopilot(ship)
    updateZone()
    updateCamera(player, dt)
end

function InGame:onViewOpen(isPageOpen)
    GameState:SetState(Enums.GameStates.InGame)
end

------------------------------------------------------------------------------
-- Components. Each is a screen-filling container (UIComponent.Container always
-- fills its parent) that places its text with alignment and padding.

--- Sizes scale with the window height (1080 p = 1).
local function px(n)
    return math.floor(n * math.max(0.7, math.min(1.3, Window:height() / 1080)))
end

local function hudVisible()
    return InGame.hudShown and InGame.player ~= nil
end

-- HmGui measures text only above the baseline (Font::get_size2), so the
-- container clips descenders (p, g, y). Vertical padding makes room for them.
local function descenderPadding()
    return { 0, px(4) }
end

local AUTOPILOT_COLOR = Color(0.51, 0.92, 1.00, 1.0)
local ZONE_COLOR = Color(0.35, 0.70, 1.00, 1.0)

-- Left of the reticle, at its height: where the speed tape's tag will sit.
local speedReadout = UIComponent.Container {
    visible = hudVisible,
    align = { AlignHorizontal.Stretch, AlignVertical.Stretch },
    childrenAlign = { AlignHorizontal.Left, AlignVertical.Center },
    padding = function() return { math.floor(Window:width() * 0.30), 0 } end,
    contents = {
        UIComponent.Text {
            text = function() return hud.speedText end,
            font = "UbuntuBold",
            size = function() return px(22) end,
            color = function() return hud.speedColor end,
        },
    },
}

-- Centre, slightly above: autopilot status over the travel drive text. The
-- autopilot line keeps a fixed height, so the drive text never moves.
local travelBlock = UIComponent.Container {
    visible = hudVisible,
    align = { AlignHorizontal.Stretch, AlignVertical.Stretch },
    layoutType = GuiLayoutType.Vertical,
    childrenAlign = { AlignHorizontal.Center, AlignVertical.Top },
    padding = function() return { 0, math.floor(Window:height() * 0.30) - px(34) - px(28) } end,
    contents = {
        UIComponent.Text {
            text = function() return hud.autopilotText end,
            font = "Unageo-Medium",
            size = function() return px(24) end,
            height = function() return px(34) end,
            color = AUTOPILOT_COLOR,
        },
        UIComponent.Text {
            visible = function() return hud.driveText ~= "" end,
            text = function() return hud.driveText end,
            font = "Unageo-Semibold",
            size = function() return px(46) end,
            color = function() return Color(1.0, 1.0, 1.0, hud.driveAlpha) end,
        },
    },
}

local zoneLine = UIComponent.Container {
    visible = hudVisible,
    align = { AlignHorizontal.Stretch, AlignVertical.Stretch },
    layoutType = GuiLayoutType.Vertical,
    childrenAlign = { AlignHorizontal.Left, AlignVertical.Top },
    padding = function() return { px(40), px(30) } end,
    contents = {
        UIComponent.Text {
            text = function() return hud.zoneText end,
            font = "UbuntuBold",
            size = function() return px(18) end,
            padding = descenderPadding,
            color = ZONE_COLOR,
        },
    },
}

local cameraLabel = UIComponent.Container {
    visible = function() return hudVisible() and hud.cameraText ~= "" end,
    align = { AlignHorizontal.Stretch, AlignVertical.Stretch },
    childrenAlign = { AlignHorizontal.Center, AlignVertical.Top },
    padding = function() return { 0, px(30) } end,
    contents = {
        UIComponent.Text {
            text = function() return hud.cameraText end,
            font = "Unageo-Medium",
            size = function() return px(20) end,
            padding = descenderPadding,
            color = function() return Color(0.80, 0.88, 0.97, hud.cameraAlpha) end,
        },
    },
}

InGame:addContent(speedReadout)
InGame:addContent(travelBlock)
InGame:addContent(zoneLine)
InGame:addContent(cameraLabel)

return InGame
