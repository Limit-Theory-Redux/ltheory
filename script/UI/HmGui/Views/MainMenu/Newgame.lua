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

local ShipHullsText = {
    "Solo",
    "Small",
    "Compact",
    "Medium",
    "Large",
    "VeryLarge",
}

local selectedSeedIndex = nil

function NewgameView:onInput() end
function NewgameView:onUpdate(dt) end
function NewgameView:onViewOpen(isPageOpen) end
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
    ---@type Universe
    local Universe = require("Systems.Universe.Universe")

    -- we want to create a new universe, do this here so loading screen knows what to load
    Universe:init(seed)
    UIRouter:setCurrentPage("Loading_Screen")
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
            contents = {
                UIComponent.Container {
                    align = { AlignHorizontal.Stretch, AlignVertical.Stretch },
                    childrenAlign = { AlignHorizontal.Center, AlignVertical.Center },
                    padding = { 0, 10 },
                    margin = { 0, 0 },
                    layoutType = GuiLayoutType.Vertical,
                    heightInLayout = 2 / 10,
                    color = {
                        background = Color(0, 0, 0, 0.3)
                    },
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
                    color = {
                        background = Color(0, 0, 0, 0.3)
                    },
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
                    color = {
                        background = Color(0, 0, 0, 0.3)
                    },
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
                UILayout.Grid {
                    align = { AlignHorizontal.Stretch, AlignVertical.Center },
                    stackDirection = GuiLayoutType.Horizontal,
                    contents = {
                        UILayout.Grid {
                            align = { AlignHorizontal.Center, AlignVertical.Center },
                            widthInLayout = 0.35, -- 35% of the parent grid width
                            layoutType = GuiLayoutType.Vertical,
                            contents = {
                                UIComponent.Text {
                                    text = "Nebula Brightness",
                                    size = 24,
                                    align = { AlignHorizontal.Left, AlignVertical.Center }
                                },
                                UIComponent.Text {
                                    text = "Asteroid Fields",
                                    size = 24,
                                    align = { AlignHorizontal.Left, AlignVertical.Center }
                                },
                                UIComponent.Text {
                                    text = "Asteroids Per Field",
                                    size = 24,
                                    align = { AlignHorizontal.Left, AlignVertical.Center }
                                },
                                UIComponent.Text {
                                    text = "Planets",
                                    size = 24,
                                    align = { AlignHorizontal.Left, AlignVertical.Center }
                                },
                                UIComponent.Text {
                                    text = "Stations",
                                    size = 24,
                                    align = { AlignHorizontal.Left, AlignVertical.Center }
                                },
                                UIComponent.Text {
                                    text = "AI Players",
                                    size = 24,
                                    align = { AlignHorizontal.Left, AlignVertical.Center }
                                },
                                UIComponent.Text {
                                    text = "EconNPCs",
                                    size = 24,
                                    align = { AlignHorizontal.Left, AlignVertical.Center }
                                },
                                UIComponent.Text {
                                    text = "EscortNPCs",
                                    size = 24,
                                    align = { AlignHorizontal.Left, AlignVertical.Center }
                                },
                                UIComponent.Text {
                                    text = "Ship Size",
                                    size = 24,
                                    align = { AlignHorizontal.Left, AlignVertical.Center }
                                },
                                UIComponent.Text {
                                    text = "Unique Ships",
                                    size = 24,
                                    align = { AlignHorizontal.Left, AlignVertical.Center }
                                }
                            }
                        },
                        UILayout.Grid {
                            align = { AlignHorizontal.Center, AlignVertical.Center },
                            widthInLayout = 0.65, -- 65% of the parent grid width
                            layoutType = GuiLayoutType.Vertical,
                            contents = {
                                UIComponent.Slider {
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
                                },
                                UIComponent.Slider {
                                    size = ResponsiveSize(300, 20),
                                    margin = { 0, 18 },
                                    align = { AlignHorizontal.Center, AlignVertical.Center },
                                    sound = Config.audio.sounds.click,
                                    toolTip = function()
                                        return
                                        "Number of asteroid fields in the new star system."
                                    end,
                                    increment = 1,
                                    minValue = 0,
                                    maxValue = 20,
                                    currentValue = function() return GameState.gen.nFields end,
                                    callback = function(v) GameState.gen.nFields = v end
                                },
                                UIComponent.Slider {
                                    size = ResponsiveSize(300, 20),
                                    margin = { 0, 18 },
                                    align = { AlignHorizontal.Center, AlignVertical.Center },
                                    sound = Config.audio.sounds.click,
                                    toolTip = function()
                                        return
                                        "Number of asteroids in each asteroid field."
                                    end,
                                    increment = 1,
                                    minValue = 0,
                                    maxValue = 200,
                                    currentValue = function() return GameState.gen.nAsteroids end,
                                    callback = function(v) GameState.gen.nAsteroids = v end
                                },
                                UIComponent.Slider {
                                    size = ResponsiveSize(300, 20),
                                    margin = { 0, 18 },
                                    align = { AlignHorizontal.Center, AlignVertical.Center },
                                    sound = Config.audio.sounds.click,
                                    toolTip = function()
                                        return
                                        "Number of planets in the new star system."
                                    end,
                                    increment = 1,
                                    minValue = 0,
                                    maxValue = 1,
                                    currentValue = function() return GameState.gen.nPlanets end,
                                    callback = function(v) GameState.gen.nPlanets = v end
                                },
                                UIComponent.Slider {
                                    size = ResponsiveSize(300, 20),
                                    margin = { 0, 18 },
                                    align = { AlignHorizontal.Center, AlignVertical.Center },
                                    sound = Config.audio.sounds.click,
                                    toolTip = function()
                                        return
                                        "Number of space stations in the new star system."
                                    end,
                                    increment = 1,
                                    minValue = 0,
                                    maxValue = 40,
                                    currentValue = function() return GameState.gen.nStations end,
                                    callback = function(v) GameState.gen.nStations = v end
                                },
                                UIComponent.Slider {
                                    size = ResponsiveSize(300, 20),
                                    margin = { 0, 18 },
                                    align = { AlignHorizontal.Center, AlignVertical.Center },
                                    sound = Config.audio.sounds.click,
                                    toolTip = function()
                                        return
                                        "Number of AI Players in the new star system."
                                    end,
                                    increment = 1,
                                    minValue = 0,
                                    maxValue = 10,
                                    currentValue = function() return GameState.gen.nAIPlayers end,
                                    callback = function(v) GameState.gen.nAIPlayers = v end
                                },
                                UIComponent.Slider {
                                    size = ResponsiveSize(300, 20),
                                    margin = { 0, 18 },
                                    align = { AlignHorizontal.Center, AlignVertical.Center },
                                    sound = Config.audio.sounds.click,
                                    toolTip = function()
                                        return
                                        "Number of economics-oriented NPC ships in the new star system."
                                    end,
                                    increment = 1,
                                    minValue = 0,
                                    maxValue = 50,
                                    currentValue = function() return GameState.gen.nEconNPCs end,
                                    callback = function(v) GameState.gen.nEconNPCs = v end
                                },
                                UIComponent.Slider {
                                    size = ResponsiveSize(300, 20),
                                    margin = { 0, 18 },
                                    align = { AlignHorizontal.Center, AlignVertical.Center },
                                    sound = Config.audio.sounds.click,
                                    toolTip = function()
                                        return
                                        "Number of combat-oriented NPC ships in the new star system."
                                    end,
                                    increment = 1,
                                    minValue = 0,
                                    maxValue = 50,
                                    currentValue = function() return GameState.gen.nEscortNPCs end,
                                    callback = function(v) GameState.gen.nEscortNPCs = v end
                                },
--                              UIComponent.Slider {
--                                  size = ResponsiveSize(300, 20),
--                                  margin = { 0, 18 },
--                                  align = { AlignHorizontal.Center, AlignVertical.Center },
--                                  sound = Config.audio.sounds.click,
--                                  toolTip = function()
--                                      return
--                                      "Size class of the player's ship."
--                                  end,
--                                  increment = 1,
--                                  minValue = 1,
--                                  maxValue = Enums.ShipHulls.VeryLarge,
--                                  currentValue = function() return GameState.gen.shipHull end,
--                                  callback = function(v) GameState.player.shipHull = v end
--                              },
                                UIComponent.Dropdown {
                                    width = 300,
                                    height = 20,
                                    margin = { 0, 18 },
                                    font = { name = "Unageo-Medium", size = 20 },
                                    align = { AlignHorizontal.Center, AlignVertical.Center },
                                    sound = Config.audio.sounds.click,
                                    toolTip = function()
                                        return
                                        "Size class of the player's ship."
                                    end,
                                    selections = ShipHullsText,
                                    selectedIndex = function() return GameState.gen.shipHull end,
                                    callback = function(v) GameState.player.shipHull = v end
                                },
                                UIComponent.Switch {
                                    size = ResponsiveSize(80, 20),
                                    margin = { 0, 18 },
                                    align = { AlignHorizontal.Center, AlignVertical.Center },
                                    sound = Config.audio.sounds.click,
                                    toolTip = function()
                                        return
                                        "Controls whether a unique mesh is generated for each NPC ship."
                                    end,
                                    currentValue = function() return GameState.gen.uniqueShips end,
                                    callback = function(v) GameState.gen.uniqueShips = v end
                                }
                            }
                        }
                    }
                },
                UIComponent.Button {
                    title = "Start New Game",
                    align = { AlignHorizontal.Center, AlignVertical.Center },
                    margin = { 0, 10 },
                    size = ResponsiveSize(300, 60),
                    font = { name = "Unageo-Medium", size = 24 },
                    toolTip = function() return "Press to start a new game with a random seed." end,
                    callback = function() newGame(rng:get64()) end,
                }
            }
        }
    }
}

NewgameView:addContent(newgameGrid)

return NewgameView
