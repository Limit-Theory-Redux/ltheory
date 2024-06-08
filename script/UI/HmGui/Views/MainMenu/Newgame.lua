---@type UIView
local NewgameView = UICore.View {
    name = "Newgame"
}

---@type UIRouter
local UIRouter = require("UI.HmGui.UICore.UIRouter")
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

function NewgameView:onInput() end
function NewgameView:onUpdate(dt) end
function NewgameView:onViewOpen(isPageOpen) end
function NewgameView:onViewClose(isPageClose) end

local function getButtonWidth()
    return GameState.render.resX / 1600 * 200
end

local function getButtonHeight()
    return GameState.render.resY / 900 * 40
end

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
                            size = 32,
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
                            width = getButtonWidth,
                            height = getButtonHeight,
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
            margin = { 0, 0 },
            widthInLayout = getRemainingWidthPercentage,
            layoutType = GuiLayoutType.Vertical,
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
                            padding = { 0, 10 },
                            margin = { 0, 0 },
                            layoutType = GuiLayoutType.Vertical,
                            heightInLayout = 2 / 10,
                            contents = {}
                        },
                        UIComponent.Container {
                            align = { AlignHorizontal.Stretch, AlignVertical.Stretch },
                            childrenAlign = { AlignHorizontal.Center, AlignVertical.Center },
                            padding = { 0, 10 },
                            margin = { 0, 0 },
                            layoutType = GuiLayoutType.Vertical,
                            heightInLayout = 7 / 10,
                            contents = {
                                UIComponent.ScrollArea {
                                    align = { AlignHorizontal.Center, AlignVertical.Center },
                                    padding = { 0, 0 },
                                    margin = { 0, 0 },
                                    layoutType = GuiLayoutType.Vertical,
                                    height = 400,
                                    scrollbarFading = true,
                                    showVScrollbar = true,
                                    contents = {
                                        UIComponent.RadioGroup {
                                            selections = function()
                                                local selections = {}
                                                for _, seed in ipairs(seeds) do
                                                    if seed[1] then
                                                        table.insert(selections, seed[1])
                                                    elseif seed[2] then
                                                        table.insert(selections, tostring(seed[2]))
                                                    else
                                                        table.insert(selections, "<unknown>")
                                                    end
                                                end
                                                return selections
                                            end,
                                            align = { AlignHorizontal.Center, AlignVertical.Top },
                                            padding = { 5, 0 },
                                            margin = { 0, 0 },
                                            layoutType = GuiLayoutType.Vertical,
                                            color = {
                                                background = Color(0, 0, 0, 0.3)
                                            },
                                            callback = function(selectedIndex) selectedSeedIndex = selectedIndex end,
                                        },
                                    }
                                },
                                UIComponent.Button {
                                    title = "Select",
                                    align = { AlignHorizontal.Center, AlignVertical.Center },
                                    margin = { 0, 10 },
                                    height = getButtonHeight,
                                    width = getButtonWidth,
                                    toolTip = function()
                                        if selectedSeedIndex then
                                            return "Press to load game with the seed:\n" ..
                                                seeds[selectedSeedIndex][1] .. "\n" ..
                                                tostring(seeds[selectedSeedIndex][2])
                                        end
                                        return "No seed selected"
                                    end,
                                    callback = function()
                                        if selectedSeedIndex then
                                            newGame(seeds[selectedSeedIndex][2])
                                        end
                                    end,
                                },
                            },
                        },
                        UIComponent.Container {
                            align = { AlignHorizontal.Stretch, AlignVertical.Stretch },
                            childrenAlign = { AlignHorizontal.Center, AlignVertical.Center },
                            padding = { 0, 0 },
                            margin = { 0, 0 },
                            heightInLayout = 1 / 10,
                            layoutType = GuiLayoutType.Vertical,
                            contents = {}
                        }
                    }
                },
            }
        }
    }
}

NewgameView:addContent(newgameGrid)

return NewgameView
