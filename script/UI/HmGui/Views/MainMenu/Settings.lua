---@type UIView
local SettingsView = UICore.View {
    name = "Settings"
}

---@type UIRouter
local UIRouter = require("UI.HmGui.UICore.UIRouter")

function SettingsView:onInput() end
function SettingsView:onUpdate(dt) end
function SettingsView:onViewOpen(isPageOpen) end
function SettingsView:onViewClose(isPageClose) end

local function getButtonWidth()
    return GameState.render.resX / 1600 * 200
end

local function getButtonHeight()
    return GameState.render.resY / 900 * 40
end

local function switchToMainScreen()
    UIRouter:getCurrentPage():setView("Main")
end

local settingOptions = {
    { false, nil, "Audio" },               -- checkbox for audio toggle
    { false, nil, "Fullscreen" },          -- checkbox for fullscreen toggle
    { 0,     nil, "Supersampling" },       -- value for enum of supersampling (anti-aliasing) mode
    { 1,     nil, "Nebula Brightness" },   -- value for brightness scale of background nebula
    { 0,     nil, "Cursor Style" },        -- value for enum of cursor style
    { 0,     nil, "HUD Style" },           -- value for enum of HUD style
    { false, nil, "Unique Ships" },        -- checkbox for unique ships toggle
    { 0,     nil, "Asteroid Fields" },     -- value for number of asteroid fields
    { 0,     nil, "Asteroids per Field" }, -- value for number of asteroids per field
    { 0,     nil, "Planets" },             -- value for number of planets
    { 0,     nil, "Stations" },            -- value for number of stations
    { 0,     nil, "AI Players" },          -- value for number of AI Players
    { 0,     nil, "EconNPCs" },            -- value for number of EconNPCs
    { 0,     nil, "EscortNPCs" },          -- value for number of EscortNPCs
    { 0,     nil, "Ship Size" },           -- value for hull type of player's ship
}

local container = UIComponent.Container {
    align = { AlignHorizontal.Stretch, AlignVertical.Stretch },
    padding = { 50, 50 },
    margin = { 0, 0 },
    stackDirection = Enums.UI.StackDirection.Horizontal,
    contents = {
        UIComponent.Container {
            align = { AlignHorizontal.Center, AlignVertical.Center },
            padding = { 50, 10 },
            margin = { 0, 0 },
            stackDirection = Enums.UI.StackDirection.Vertical,
            contents = {
                UIComponent.Button_MainMenu {
                    title = "Back",
                    width = getButtonWidth,
                    height = getButtonHeight,
                    align = { AlignHorizontal.Center, AlignVertical.Center },
                    callback = switchToMainScreen
                },
            }
        },
        UIComponent.Container {
            align = { AlignHorizontal.Center, AlignVertical.Center },
            padding = { 5, 5 },
            margin = { 0, 0 },
            stackDirection = Enums.UI.StackDirection.Vertical,
            contents = {
                UIComponent.RawInput { fn = function()
                    for _, setting in ipairs(settingOptions) do
                        Gui:text(setting[3])
                    end
                end }
            }
        }
    }
}

SettingsView:addContent(container)

return SettingsView
