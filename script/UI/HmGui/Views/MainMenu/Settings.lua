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

local function getLayoutContainerWidthPercentage() --todo: needs replacement with a more sophisticated layout system
    return GameState.render.resX / 1600 * 175 * 2 / GameState.render.resX
end

local function getRemainingWidthPercentage()
    return 1 - getLayoutContainerWidthPercentage()
end

local selectedSetting = "Audio"

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

local settingsGrid = UILayout.Grid {
    align = { AlignHorizontal.Stretch, AlignVertical.Stretch },
    padding = { 50, 50 },
    margin = { 0, 0 },
    stackDirection = Enums.UI.StackDirection.Horizontal,
    showGrid = false,
    contents = {
        UIComponent.Container {
            align = { AlignHorizontal.Default, AlignVertical.Center },
            padding = { 50, 10 },
            margin = { 0, 0 },
            widthInLayout = getLayoutContainerWidthPercentage,
            stackDirection = Enums.UI.StackDirection.Vertical,
            contents = {
                UIComponent.Button_MainMenu {
                    title = "Audio",
                    width = getButtonWidth,
                    height = getButtonHeight,
                    align = { AlignHorizontal.Center, AlignVertical.Center },
                    callback = function() selectedSetting = "Audio" end
                },
                UIComponent.Button_MainMenu {
                    title = "Interface",
                    width = getButtonWidth,
                    height = getButtonHeight,
                    align = { AlignHorizontal.Center, AlignVertical.Center },
                    callback = function() selectedSetting = "Interface" end
                },
                UIComponent.Button_MainMenu {
                    title = "Graphics",
                    width = getButtonWidth,
                    height = getButtonHeight,
                    align = { AlignHorizontal.Center, AlignVertical.Center },
                    callback = function() selectedSetting = "Graphics" end
                },
                UIComponent.Button_MainMenu {
                    title = "Keybinding",
                    width = getButtonWidth,
                    height = getButtonHeight,
                    align = { AlignHorizontal.Center, AlignVertical.Center },
                    callback = function() selectedSetting = "Keybinding" end
                },
                UIComponent.Button_MainMenu {
                    title = "Back",
                    margin = { 0, 10 },
                    width = getButtonWidth,
                    height = getButtonHeight,
                    align = { AlignHorizontal.Center, AlignVertical.Center },
                    callback = switchToMainScreen
                },
            }
        },
        UIComponent.Container {
            align = { AlignHorizontal.Stretch, AlignVertical.Stretch },
            childrenAlign = { AlignHorizontal.Center, AlignVertical.Center },
            padding = { 50, 50 },
            margin = { 0, 0 },
            widthInLayout = getRemainingWidthPercentage,
            stackDirection = Enums.UI.StackDirection.Vertical,
            contents = {
                UIComponent.RawInput { fn = function()
                    --! mockup
                    Gui:beginStackContainer()
                    Gui:setPropertyFont(GuiProperties.TextFont, Cache.Font("Exo2", 28))
                    Gui:text(selectedSetting)
                    Gui:setAlignment(AlignHorizontal.Center, AlignVertical.Top)
                    Gui:setMargin(0, 20)
                    Gui:rect(Color(.1, .1, .115, .25))
                    Gui:setPercentSize(100, 100)
                    Gui:endContainer()
                    Gui:setPercentSize(100, 100)
                end }
            }
        }
    }
}

SettingsView:addContent(settingsGrid)

return SettingsView
