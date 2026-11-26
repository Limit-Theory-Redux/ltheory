-- Sensor tuning panel: the numbers behind the target's reading, and the dials,
-- for tuning the sensor model in a scene. A developer's view, not a player's.
--
-- Top: what the sensor hears of the target per channel (EM, heat, light) as
-- an SNR against the "heard" line (SNR 1), the strongest one bright; how much
-- the background raises the noise, and why; how many resolution cells the
-- target fills against the T2 and T3 marks; the scan echo against the T3 and
-- T4 marks and the pulse echo against "found" (SNR 1); the tiers it has
-- reached and why. Bottom: the dials, each with ticks where the fighter,
-- frigate and capital ship have it, the selected one framed.
--
--   local panel = UIComponent.SensorTuningPanel {}
--   -- every frame, before rendering (nil hides it):
--   panel:update({ sensor = sensor, hull = "frigate", selected = 2 })
--
-- Places and sizes are pixels on a 1920 x 1080 screen, scaled with the window
-- height and kept relative to the bottom left corner.

local SensorModel = require("Modules.Sensors.SensorModel")
local SensorText = require("Modules.Sensors.SensorText")
local SensorTuning = require("Modules.Sensors.SensorTuning")

---@class SensorTuningView
---@field sensor SensorComponent
---@field hull string the hull whose dials the sensor started from (a key of `SensorModel.hulls`)
---@field selected integer the selected dial, an index into `SensorTuning.dials`

---@class UIComponentSensorTuningPanel: UIComponent
---@field state UIComponentState
---@field update fun(self: UIComponentSensorTuningPanel, view: SensorTuningView|nil) nil hides the panel
---@field render fun(self: UIComponentSensorTuningPanel)

---@class UIComponentSensorTuningPanelConstructor
---@field visible boolean|(fun(): boolean)|nil
---@field scale number|(fun(): number)|nil sizes relative to 1080p; default window height / 1080

-- The panel's top left corner and size, and its columns: labels, bars (and
-- their width), values (right aligned at this edge).
local PANEL = { x = 36, y = 564, w = 480, h = 480 }
local COL = { label = 14, bar = 84, barW = 300, value = 466, dialW = 270 }
-- Rows: the middle of each bar, from the panel's top
local ROWS = {
    title = 26, channels = { 52, 70, 88 }, heard = 106, noise = 128, cells = 150, cellMarks = 168,
    rule1 = 182, scan = 202, scanMarks = 220, pulse = 238, tiers = 264, rule2 = 290,
    dialsTitle = 312, dials = 336, dialStep = 20, footer = 464,
}
-- Log bars: the powers of ten at their ends
local SNR_BAR = { -1, 5 }   -- the channels and the pulse echo; SNR 1 sits at a sixth
local CELLS_BAR = { -2, 2 }
local SCAN_BAR = { 0, 6 }
local BAR_H, DIAL_H = 7, 4
local FONT, FONT_BOLD = "Ubuntu", "UbuntuBold"

local function rgb(r, g, b) return { r / 255, g / 255, b / 255 } end

local STYLE = {
    hud = rgb(26, 153, 255),
    cyan = rgb(51, 230, 255),
    pale = rgb(190, 225, 255),
    grey = rgb(143, 163, 187),
    dim = rgb(91, 107, 128),
    amber = rgb(255, 170, 51),
    amberPale = rgb(255, 213, 154),
    back = { 0.03, 0.05, 0.08 },
}

local CHANNELS = { { "EM", "EM" }, { "heat", "HEAT" }, { "light", "LIGHT" } }

local function color(c, alpha) return Color(c[1], c[2], c[3], alpha or 1) end

---@param _ table
---@param args UIComponentSensorTuningPanelConstructor
---@return UIComponentSensorTuningPanel|nil
local function new(_, args)
    if not args then return end

    local panel = {}
    panel.state = UICore.ComponentState {
        visible = args.visible,
        scale = args.scale or function() return Window:height() / 1080 end,
    }

    ---@type SensorTuningView|nil
    local view

    function panel:update(newView)
        view = newView
    end

    function panel:render()
        if not view or not self.state.visible() then return end

        local s = self.state.scale()
        local H = Window:height()
        local ox, oy = PANEL.x * s, H - 1080 * s + PANEL.y * s -- the panel's corner, kept to the bottom left
        local function X(x) return ox + x * s end
        local function Y(y) return oy + y * s end

        -- Shapes, at places within the panel (1080p pixels)
        local function rect(c, alpha, x, y, w, h)
            Gui:rect()
            Gui:setBackgroundColor(color(c, alpha))
            Gui:setFixedSize(math.max(1, w * s), math.max(1, h * s))
            Gui:setPosition(X(x), Y(y))
        end

        local function circle(c, alpha, x, y, r)
            Gui:circle(color(c, alpha), false)
            Gui:setFixedSize(2 * r * s, 2 * r * s)
            Gui:setPosition(X(x) - r * s, Y(y) - r * s)
        end

        --- A box's outline, one pixel wide.
        local function frame(c, alpha, x, y, w, h)
            rect(c, alpha, x, y, w, 1)
            rect(c, alpha, x, y + h - 1, w, 1)
            rect(c, alpha, x, y, 1, h)
            rect(c, alpha, x + w - 1, y, 1, h)
        end

        --- A mark across a bar at `at` (0 to 1), dotted for a "heard" line.
        local function mark(c, x, y, at, half, dotted)
            local mx = x + at * COL.barW
            if dotted then
                for k = -half, half - 2, 4 do rect(c, 0.9, mx, y + k, 1, 2) end
            else
                rect(c, 0.9, mx, y - half, 1, 2 * half)
            end
            return mx
        end

        --- Text with its middle on row y; align "left", "right" or "center".
        local function text(str, x, y, size, c, alpha, align, bold)
            local font = Cache.Font(bold and FONT_BOLD or FONT, math.max(8, math.floor(size * s + 0.5)))
            local w, h = 440 * s, size * 1.6 * s
            local left = X(x)
            if align == "right" then left = left - w elseif align == "center" then left = left - w / 2 end
            Gui:beginStackContainer()
            Gui:setClipping(false)
            Gui:setFixedSize(w, h)
            Gui:setPosition(left, Y(y) - h / 2)
            Gui:text(str, font, color(c, alpha))
            Gui:setAlignment(
                align == "right" and AlignHorizontal.Right or align == "center" and AlignHorizontal.Center or AlignHorizontal.Left,
                AlignVertical.Center)
            Gui:endContainer()
        end

        --- A row's bar: a faint track, and its fill up to `at` (0 to 1).
        local function bar(c, alpha, y, at)
            rect(c, 0.12, COL.bar, y - BAR_H / 2, COL.barW, BAR_H)
            if at > 0 then rect(c, alpha, COL.bar, y - BAR_H / 2, at * COL.barW, BAR_H) end
        end

        local sensor = view.sensor
        local state = sensor:getState()
        local dials = sensor:getDials()
        local target = sensor:getTarget()
        local reading = target and sensor:getReadings()[target]

        Gui:beginStackContainer()
        Gui:setClipping(false)

        rect(STYLE.back, 0.82, 0, 0, PANEL.w, PANEL.h)
        frame(STYLE.hud, 0.45, 0, 0, PANEL.w, PANEL.h)
        text("BEHIND THE SCENES", COL.label, ROWS.title, 13, STYLE.hud, 1, "left", true)

        if reading then
            local p = reading.passive
            local class = SensorModel.shipClass(target)
            text(string.format("T%d · %s · %s", reading.tier,
                class > 0 and "CLASS " .. SensorText.roman[class] or string.upper(SensorModel.kinds[target.kind].name),
                string.upper(SensorText.distance(p.distance))),
                COL.value, ROWS.title, 13, STYLE.pale, 1, "right", true)

            -- What reaches the sensor per channel, as SNR, against the "heard" line
            local per = dials.sensitivity * p.near * p.dimmed / p.noise
            for i, ch in ipairs(CHANNELS) do
                local y = ROWS.channels[i]
                local off = ch[1] == "heat" and not dials.heatBand
                local snr = p.channels[ch[1]] * per
                local strongest = ch[1] == p.via and p.snr >= 1
                text(ch[2], COL.label, y, 12, STYLE.grey)
                bar(STYLE.cyan, strongest and 1 or 0.55, y, off and 0 or SensorTuning.logBar(snr, SNR_BAR[1], SNR_BAR[2]))
                text(off and "off" or SensorText.number(snr), COL.value, y, 12, STYLE.pale, 1, "right")
            end
            local heardX = mark(STYLE.pale, COL.bar, (ROWS.channels[1] + ROWS.channels[3]) / 2,
                SensorTuning.logBar(1, SNR_BAR[1], SNR_BAR[2]), 26, true)
            text("heard", heardX, ROWS.heard, 11, STYLE.pale, 0.9, "center")

            -- How much the background raises the noise, and what it is
            local why = #p.surroundings.backgroundWhy > 0 and table.concat(p.surroundings.backgroundWhy, ", ")
                or (p.surroundings.by ~= "" and p.surroundings.by or "open space")
            text("NOISE", COL.label, ROWS.noise, 12, STYLE.grey)
            rect(p.backgroundFactor > 1.01 and STYLE.amber or STYLE.dim, 0.9, COL.bar, ROWS.noise - BAR_H / 2,
                math.max(4, 40 * math.min(4, 1 + math.log10(p.backgroundFactor))), BAR_H)
            text(string.format("×%s · %s", SensorText.number(p.backgroundFactor), why), COL.bar + 176, ROWS.noise, 12,
                STYLE.pale, 1, "left")
            if state.resolutionRule == "dim" then
                text(string.format("dims ×%s", SensorText.number(p.dimmed)), COL.value, ROWS.noise, 12, STYLE.pale, 1, "right")
            end

            -- Its size in resolution cells, against T2 and T3
            text("CELLS", COL.label, ROWS.cells, 12, STYLE.grey)
            bar(STYLE.cyan, 0.8, ROWS.cells, SensorTuning.logBar(p.cells, CELLS_BAR[1], CELLS_BAR[2]))
            for _, m in ipairs({ { SensorModel.constants.classCells, "T2" }, { SensorModel.constants.identifyCells, "T3" } }) do
                local mx = mark(STYLE.pale, COL.bar, ROWS.cells, SensorTuning.logBar(m[1], CELLS_BAR[1], CELLS_BAR[2]), 8)
                text(m[2], mx, ROWS.cellMarks, 11, STYLE.pale, 0.9, "center")
            end
            text(SensorModel.formatCells(p.cells), COL.value, ROWS.cells, 12, STYLE.pale, 1, "right")

            rect(STYLE.hud, 0.25, COL.label, ROWS.rule1, PANEL.w - 2 * COL.label, 1)

            -- The active echoes: the scan against T3 and T4, the pulse against "found"
            local scan = SensorModel.scanEcho(state, p)
            text("SCAN", COL.label, ROWS.scan, 12, STYLE.amber)
            bar(STYLE.amber, 0.8, ROWS.scan, SensorTuning.logBar(scan.snr, SCAN_BAR[1], SCAN_BAR[2]))
            for _, m in ipairs({ { SensorModel.constants.scanT3Snr, "T3" }, { SensorModel.constants.scanT4Snr, "T4" } }) do
                local mx = mark(STYLE.amberPale, COL.bar, ROWS.scan, SensorTuning.logBar(m[1], SCAN_BAR[1], SCAN_BAR[2]), 8)
                text(m[2], mx, ROWS.scanMarks, 11, STYLE.amberPale, 0.9, "center")
            end
            text(SensorText.number(scan.snr), COL.value, ROWS.scan, 12, STYLE.amberPale, 1, "right")

            local pulse = SensorModel.pulseEcho(state, target, p)
            text("PULSE", COL.label, ROWS.pulse, 12, STYLE.amber)
            bar(STYLE.amber, 0.8, ROWS.pulse, SensorTuning.logBar(pulse.snr, SNR_BAR[1], SNR_BAR[2]))
            mark(STYLE.amberPale, COL.bar, ROWS.pulse, SensorTuning.logBar(1, SNR_BAR[1], SNR_BAR[2]), 8, true)
            text(SensorText.number(pulse.snr), COL.value, ROWS.pulse, 12, STYLE.amberPale, 1, "right")

            -- The tiers it has reached, and why
            for t = 1, 4 do
                local x = COL.bar + (t - 1) * 42
                if t <= reading.tier then
                    rect(STYLE.cyan, 0.85, x, ROWS.tiers - 9, 36, 18)
                    text("T" .. t, x + 18, ROWS.tiers, 12, STYLE.back, 1, "center", true)
                else
                    frame(STYLE.dim, 1, x, ROWS.tiers - 9, 36, 18)
                    text("T" .. t, x + 18, ROWS.tiers, 12, STYLE.grey, 1, "center")
                end
            end
            text(reading.why, COL.value, ROWS.tiers, 12, STYLE.pale, 0.9, "right")
        else
            text("NO TARGET · TAB PICKS ONE", COL.value, ROWS.title, 13, STYLE.grey, 1, "right", true)
        end

        rect(STYLE.hud, 0.25, COL.label, ROWS.rule2, PANEL.w - 2 * COL.label, 1)

        -- The dials: a slider each, ticks where the hulls have it, the selected one framed
        local hull = SensorModel.hulls[view.hull]
        local edited = false
        for _, spec in ipairs(SensorTuning.dials) do
            if hull and dials[spec.key] ~= hull[spec.key] then edited = true end
        end
        text(string.format("DIALS · %s%s", string.upper(SensorModel.hullNames[view.hull] or "?"), edited and " · EDITED" or ""),
            COL.label, ROWS.dialsTitle, 13, edited and STYLE.amber or STYLE.hud, 1, "left", true)

        for i, spec in ipairs(SensorTuning.dials) do
            local y = ROWS.dials + (i - 1) * ROWS.dialStep
            local selected = i == view.selected
            local c = selected and STYLE.amber or STYLE.pale
            if selected then
                rect(STYLE.amber, 0.08, COL.label - 6, y - 9, PANEL.w - 2 * COL.label + 12, 18)
                frame(STYLE.amber, 0.7, COL.label - 6, y - 9, PANEL.w - 2 * COL.label + 12, 18)
            end
            text(spec.name, COL.label, y, 12, selected and STYLE.amberPale or STYLE.grey)
            local value = dials[spec.key]
            if spec.toggle then
                rect(value and STYLE.cyan or STYLE.dim, value and 0.8 or 0.6, COL.bar, y - 6, 26, 12)
                circle(STYLE.back, 1, COL.bar + (value and 19 or 7), y, 4)
            else
                rect(selected and STYLE.amber or STYLE.hud, selected and 0.25 or 0.15, COL.bar, y - DIAL_H / 2, COL.dialW, DIAL_H)
                for _, key in ipairs(SensorModel.hullOrder) do
                    local tx = COL.bar + SensorTuning.position(spec, SensorModel.hulls[key][spec.key]) * COL.dialW
                    rect(STYLE.pale, 0.5, tx, y - 6, 1, 12)
                end
                circle(c, 1, COL.bar + SensorTuning.position(spec, value) * COL.dialW, y, 4.5)
            end
            text(spec.format(value), COL.value, y, 12, selected and STYLE.amberPale or STYLE.pale, 1, "right")
        end

        text("ticks: fighter, frigate, capital ship", COL.label, ROWS.footer, 11, STYLE.dim)
        text("↑ ↓ dial · ← → change · Backspace hull values · 8 hide", COL.value, ROWS.footer, 11, STYLE.dim, 1,
            "right")

        Gui:endContainer()
        Gui:setPosition(0, 0)
        Gui:setPercentSize(100, 100)
    end

    return panel
end

UIComponent.SensorTuningPanel = setmetatable({ style = STYLE }, { __call = new })

return UIComponent.SensorTuningPanel
