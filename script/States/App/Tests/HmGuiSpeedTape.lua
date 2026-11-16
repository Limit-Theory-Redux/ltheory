--- The HUD's speed tape (`UIComponent.SpeedTape`) in six states side by side:
--- stopped, slow, at max, boosting, and travel mode low and near the zone's
--- top speed. A bright patch behind each tape's lower half shows that the
--- scale stays readable on a bright background.
--- Run: `./bin/ltr.exe HmGuiSpeedTape` (capture with LTHEORY_CAPTURE).
local Test = require('States.Application')
local FlightProfileRegistry = require("Shared.Registries.FlightProfileRegistry")

function Test:getTitle() return 'HmGui Speed Tape' end

local backdrop = Color(0.02, 0.025, 0.04, 1.0)
local bright = Color(0.85, 0.9, 1.0, 1.0)
local dim = Color(0.6, 0.65, 0.7, 1.0)

local COLUMNS, ROWS = 3, 2

function Test:onInit()
    self.titleFont = Cache.Font('Unageo-Medium', 16)

    local max = FlightProfileRegistry:get(Enums.FlightProfile.Solo).maxSpeed
    local zoneTop = Config.game.travelDrive.maxSpeedByZone.planet
    self.cells = {
        { title = 'Stopped', speed = 0 },
        { title = 'Slow: a quarter of max', speed = 0.25 * max },
        { title = 'At max', speed = max },
        { title = 'Boosting', speed = 2.3 * max },
        { title = 'Travel mode x30', speed = 30 * max, travel = true },
        { title = string.format('Travel mode x1500 (zone top x%d)', zoneTop), speed = 1500 * max, travel = true },
    }
    for _, cell in ipairs(self.cells) do
        cell.input = {
            speed = cell.speed,
            maxSpeed = max,
            boostSpeed = 3 * max,
            travelSpeed = zoneTop * max,
            travel = cell.travel == true,
        }
        cell.tape = UIComponent.SpeedTape {
            centre = function() return cell.cx, cell.cy end,
            scale = function() return self.scale end,
        }
        cell.tape:update(cell.input, nil) -- no blending: show the state's colour at once
    end
end

function Test:onInput() end

--- Places each cell's reticle so its tag sits a little left of the cell's middle.
function Test:layout()
    self.scale = self.resY / 1080
    local w, h = self.resX / COLUMNS, self.resY / ROWS
    for i, cell in ipairs(self.cells) do
        local col, row = (i - 1) % COLUMNS, math.floor((i - 1) / COLUMNS)
        cell.left, cell.top = col * w, row * h
        cell.tagX = cell.left + 0.42 * w
        cell.cx = cell.tagX + 408 * self.scale -- the tape's radius: its tag sits on the arc's left
        cell.cy = cell.top + 0.5 * h + 12 * self.scale
    end
end

function Test:buildGui()
    self:layout()
    local s = self.scale
    Gui:beginGui(self.resX, self.resY)

    for _, cell in ipairs(self.cells) do
        Gui:rect()
        Gui:setBackgroundColor(bright)
        Gui:setFixedSize(100 * s, 150 * s)
        Gui:setPosition(cell.tagX - 50 * s, cell.cy + 40 * s)

        -- A positioned widget takes its fixed size, so the title gets a box.
        Gui:beginStackContainer()
        Gui:setFixedSize(self.resX / COLUMNS - 48 * s, 30 * s)
        Gui:setPosition(cell.left + 24 * s, cell.top + 18 * s)
        Gui:text(cell.title, self.titleFont, dim)
        Gui:setAlignment(AlignHorizontal.Left, AlignVertical.Center)
        Gui:endContainer()
    end
    for _, cell in ipairs(self.cells) do
        cell.tape:render()
    end

    Gui:endGui()
end

-- Built and drawn from the render event (`Application` never calls `onUpdate`).
function Test:onRender()
    self:buildGui()
    self:immediateUI(function()
        -- An opaque backdrop, like the game's scene behind the HUD.
        Imm.Rect(0, 0, self.resX, self.resY, backdrop)
        Gui:draw()
    end)
end

return Test
