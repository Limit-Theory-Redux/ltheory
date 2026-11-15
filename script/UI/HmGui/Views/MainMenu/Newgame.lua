---@type UIView
local NewgameView = UICore.View {
    name = "Newgame"
}

---@type UIRouter
local UIRouter = require("UI.HmGui.UICore.UIRouter")
---@type ResponsiveSize
local ResponsiveSize = require("Types.ResponsiveSize")
---@type RandomNumberGenerator
local rng = RNG.FromTime()

local seeds = {
    -- TODO: replace with proper list
    { "Random Seed",                                      nil },
    { "Test Seed 1",                                      11487961515238620437ULL },
    { "Test Seed 2",                                      6934033808124312024ULL },
    { "Test Seed 3",                                      8616071071418665380ULL },
    { "Test Seed 4",                                      17682038400513250095ULL },
    { "KEEP black",                                       5022463494542550306ULL },  -- KEEP black (good for testing dynamic lighting)
    { "KEEP red",                                         5012768293123392491ULL },  -- KEEP red
    { "KEEP blue and milky white",                        4933876146649964811ULL },  -- KEEP blue and milky white
    { "MAYBE orange-ish",                                 2008422628673393673ULL },  -- MAYBE orange-ish
    { "KEEP gold-yellow",                                 5712598467986491931ULL },  -- KEEP gold-yellow
    { "KEEP milky-white and light blue",                  8272263000674654607ULL },  -- KEEP milky-white and light blue (really pretty)
    { "KEEP bluish-green with a bright gold star",        14169804077813660835ULL }, -- KEEP bluish-green with a bright gold star
    { "KEEP violet",                                      9806676695553338612ULL },  -- KEEP violet
    { "KEEP blue",                                        14600758714913275339ULL }, -- KEEP blue
    { "KEEP bright green",                                11589761683708427350ULL }, -- KEEP bright green
    { "KEEP blue-red-orange",                             3432712644463072838ULL },  -- KEEP blue-red-orange
    { "MAYBE 'Hubble palette'",                           10630444862697458122ULL }, -- MAYBE "Hubble palette"
    { "KEEP even bluish-white with a bright yellow star", 5199604093543988311ULL },  -- KEEP even bluish-white with a bright yellow star
    { "KEEP completely dark with one small blue star",    9471911754066691691ULL },  -- KEEP completely dark with one small blue star
    { "looks pretty cool",                                15887563511063255006ULL }, -- looks pretty cool
    { "looks pretty cool too",                            976665863517979971ULL },   -- looks pretty cool too
}


local selectedSeedIndex = nil

-- 2^64 - 1, the largest seed
local MAX_SEED = 18446744073709551615ULL

---The seed as the player sees it: digits, without LuaJIT's `ULL` suffix.
---@param seed integer
---@return string
local function seedToText(seed)
    return (tostring(seed):gsub("U?LL$", ""))
end

---The seed a text stands for. A whole number from 0 to 2^64 - 1 is used as it is;
---any other text (a word, a longer number) is hashed, so it works as a seed too.
---Empty text gives nil: a random seed.
---@param text string
---@return integer|nil
local function textToSeed(text)
    text = text:match("^%s*(.-)%s*$")
    if text == "" then
        return nil
    end
    if text:match("^%d+$") then
        local seed = 0ULL
        for i = 1, #text do
            local digit = text:byte(i) - 48
            if seed > (MAX_SEED - digit) / 10ULL then
                return Hash.FNVStr64(text) -- larger than 2^64 - 1
            end
            seed = seed * 10ULL + digit
        end
        return seed
    end
    return Hash.FNVStr64(text)
end

-- The universe seed: filled with a random seed whenever the view opens, so the
-- player can see (and note down or share) the seed of the game they start.
local seedView = UIComponent.TextView {
    text = "",
    editable = true,
    multiline = false,
    style = { font = { size = 20 } },
    backgroundColor = Color(0.08, 0.09, 0.11, 0.9),
    borderWidth = 1,
    borderColor = Color(0.45, 0.48, 0.52, 1.0),
    width = 300,
    align = { AlignHorizontal.Center, AlignVertical.Center },
}

local function rollSeed()
    seedView.state.textData():setText(seedToText(rng:get64()))
end

function NewgameView:onInput() end
function NewgameView:onUpdate(dt) end
function NewgameView:onViewOpen(isPageOpen) rollSeed() end
function NewgameView:onViewClose(isPageClose) end

local function getLayoutContainerWidthPercentage() --todo: needs replacement with a more sophisticated layout system
    return GameState.render.resX / 1600 * 170 * 2 / GameState.render.resX
end

local function getRemainingWidthPercentage()
    return 1 - getLayoutContainerWidthPercentage()
end

local function switchToMainScreen()
    UIRouter:getCurrentPage():setView("Main")
end

---@param seed integer|nil
local function newGame(seed)
    local seed = seed or rng:get64()

    -- Route through the app state's scene builder (UniverseManager +
    -- Rulesets + SolarSystemVisualizer) via the seam the state exposes
    -- in onInit (GameState.startGame). The legacy LoadingScreen flow
    -- and its DebugControl.ltheory hook are gone.
    if GameState.startGame then
        GameState.startGame(seed)
    end
end

local newgameGrid = UILayout.Grid {
    align = { AlignHorizontal.Stretch, AlignVertical.Stretch },
    padding = { 125, 0 },
    margin = { 0, 0 },
    stackDirection = GuiLayoutType.Horizontal,
    contents = {
        UILayout.Grid {
            align = { AlignHorizontal.Stretch, AlignVertical.Stretch },
            padding = { 0, 0 },
            margin = { 0, 0 },
            widthInLayout = getLayoutContainerWidthPercentage,
            layoutType = GuiLayoutType.Vertical,
            color = {
                background = Color(0, 0, 0, 0.3)
            },
            contents = {
                UIComponent.Container {
                    align = { AlignHorizontal.Stretch, AlignVertical.Stretch },
                    childrenAlign = { AlignHorizontal.Center, AlignVertical.Center },
                    padding = { 0, 10 },
                    margin = { 0, 0 },
                    layoutType = GuiLayoutType.Vertical,
                    heightInLayout = 2 / 10,
                    contents = {
                        UIComponent.Text {
                            text = "NEW GAME",
                            size = 40,
                            font = "Unageo-Medium"
                        }
                    }
                },
                UIComponent.Container {
                    align = { AlignHorizontal.Stretch, AlignVertical.Top },
                    padding = { 0, 50 },
                    margin = { 0, 0 },
                    layoutType = GuiLayoutType.Vertical,
                    heightInLayout = 7 / 10,
                    contents = {
                        UIComponent.Button_MainMenu {
                            title = "Back",
                            size = ResponsiveSize(300, 60),
                            font = { name = "Unageo-Medium", size = 24 },
                            callback = switchToMainScreen,
                            align = { AlignHorizontal.Center, AlignVertical.Center }
                        },
                    }
                },
                UIComponent.Container {
                    align = { AlignHorizontal.Stretch, AlignVertical.Stretch },
                    childrenAlign = { AlignHorizontal.Center, AlignVertical.Center },
                    padding = { 0, 0 },
                    margin = { 0, 0 },
                    heightInLayout = 1 / 10,
                    layoutType = GuiLayoutType.Vertical,
                    contents = {
                        UIComponent.Text {
                            text = Config.gameVersion,
                            align = { AlignHorizontal.Center, AlignVertical.Center },
                            font = "Exo2",
                            size = 12
                        }
                    }
                }
            }
        },
        UIComponent.Container {
            align = { AlignHorizontal.Stretch, AlignVertical.Stretch },
            childrenAlign = { AlignHorizontal.Center, AlignVertical.Center },
            padding = { 0, 0 },
            margin = { 80, 0 },
            widthInLayout = getRemainingWidthPercentage,
            layoutType = GuiLayoutType.Vertical,
            contents = {
                UILayout.SettingList {
                    rows = {
                        UILayout.SettingRow {
                            label = "Universe Seed",
                            control = UIComponent.Container {
                                align = { AlignHorizontal.Center, AlignVertical.Center },
                                childrenAlign = { AlignHorizontal.Center, AlignVertical.Center },
                                padding = { 0, 0 },
                                margin = { 0, 18 },
                                spacing = 10,
                                layoutType = GuiLayoutType.Horizontal,
                                contents = {
                                    seedView,
                                    UIComponent.Button {
                                        title = "New Seed",
                                        size = ResponsiveSize(140, 34),
                                        font = { name = "Unageo-Medium", size = 18 },
                                        sound = Config.audio.sounds.click,
                                        toolTip = function()
                                            return
                                            "Picks a new random seed.\nYou can also type a number, or any text."
                                        end,
                                        callback = rollSeed,
                                    }
                                }
                            }
                        },
                        UILayout.SettingRow {
                            label = "Nebula Brightness",
                            control = UIComponent.Slider {
                                size = ResponsiveSize(300, 20),
                                margin = { 0, 18 },
                                align = { AlignHorizontal.Center, AlignVertical.Center },
                                sound = Config.audio.sounds.click,
                                toolTip = function()
                                    return
                                    "1 = low brightness, 10 = high brightness."
                                end,
                                increment = 1,
                                minValue = 1,
                                maxValue = 10,
                                currentValue = function() return GameState.gen.nebulaBrightnessScale end,
                                callback = function(v) GameState.gen.nebulaBrightnessScale = v end
                            }
                        },
                    }
                },
                UIComponent.Button {
                    title = "Start New Game",
                    margin = { 0, 10 },
                    size = ResponsiveSize(300, 60),
                    font = { name = "Unageo-Medium", size = 24 },
                    toolTip = function() return "Starts a new game with the universe seed above." end,
                    callback = function() newGame(textToSeed(seedView.state.textData():text())) end,
                }
            }
        }
    }
}

NewgameView:addContent(newgameGrid)

return NewgameView
