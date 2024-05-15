---@type UIView
local PlayView = UICore.View {
    name = "Play"
}

---@type UIRouter
local UIRouter = require("UI.HmGui.UICore.UIRouter")
---@type RandomNumberGenerator
local rng = RNG.FromTime()

function PlayView:onInput() end
function PlayView:onUpdate(dt) end
function PlayView:onViewOpen(isPageOpen) end
function PlayView:onViewClose(isPageClose) end

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
    local seed = seed
    if not seed then
        seed = rng:get64()
    end
    ---@type Universe
    local Universe = require("Systems.Universe.Universe")
    -- we want to create a new universe, do this here so loading screen knows what to load
    Universe:init(seed)
    UIRouter:setCurrentPage("Loading_Screen")
end

local playGrid = UILayout.Grid {
    align = { AlignHorizontal.Stretch, AlignVertical.Stretch },
    padding = { 125, 0 },
    margin = { 0, 0 },
    stackDirection = Enums.UI.StackDirection.Horizontal,
    contents = {
        UILayout.Grid {
            align = { AlignHorizontal.Stretch, AlignVertical.Stretch },
            padding = { 0, 0 },
            margin = { 0, 0 },
            widthInLayout = getLayoutContainerWidthPercentage,
            stackDirection = Enums.UI.StackDirection.Vertical,
            contents = {
                UIComponent.Container {
                    align = { AlignHorizontal.Stretch, AlignVertical.Stretch },
                    childrenAlign = { AlignHorizontal.Center, AlignVertical.Center },
                    padding = { 0, 10 },
                    margin = { 0, 0 },
                    stackDirection = Enums.UI.StackDirection.Vertical,
                    heightInLayout = 2 / 10,
                    color = {
                        background = Color(0, 0, 0, 0.3)
                    },
                    contents = {
                        UIComponent.Text {
                            text = "PLAY",
                            size = 32,
                            font = "Unageo-Medium"
                        }
                    }
                },
                UIComponent.Container {
                    align = { AlignHorizontal.Stretch, AlignVertical.Top },
                    padding = { 0, 50 },
                    margin = { 0, 0 },
                    stackDirection = Enums.UI.StackDirection.Vertical,
                    heightInLayout = 7 / 10,
                    color = {
                        background = Color(0, 0, 0, 0.3)
                    },
                    contents = {
                        UIComponent.Button_MainMenu {
                            title = "New Game",
                            width = getButtonWidth,
                            height = getButtonHeight,
                            align = { AlignHorizontal.Center, AlignVertical.Center }
                        },
                        UIComponent.Button_MainMenu {
                            title = "Load Game",
                            width = getButtonWidth,
                            height = getButtonHeight,
                            align = { AlignHorizontal.Center, AlignVertical.Center },
                        },
                        UIComponent.Button_MainMenu {
                            title = "Back",
                            width = getButtonWidth,
                            height = getButtonHeight,
                            callback = switchToMainScreen,
                            align = { AlignHorizontal.Center, AlignVertical.Center }
                        }
                    }
                },
                UIComponent.Container {
                    align = { AlignHorizontal.Stretch, AlignVertical.Stretch },
                    childrenAlign = { AlignHorizontal.Center, AlignVertical.Center },
                    padding = { 0, 0 },
                    margin = { 0, 0 },
                    heightInLayout = 1 / 10,
                    stackDirection = Enums.UI.StackDirection.Vertical,
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
            stackDirection = Enums.UI.StackDirection.Vertical,
            contents = {
                UILayout.Grid {
                    align = { AlignHorizontal.Stretch, AlignVertical.Stretch },
                    padding = { 0, 0 },
                    margin = { 0, 0 },
                    widthInLayout = getLayoutContainerWidthPercentage,
                    stackDirection = Enums.UI.StackDirection.Vertical,
                    contents = {
                        UIComponent.Container {
                            align = { AlignHorizontal.Center, AlignVertical.Center },
                            padding = { 0, 10 },
                            margin = { 0, 0 },
                            stackDirection = Enums.UI.StackDirection.Vertical,
                            heightInLayout = 2 / 10,
                            contents = {}
                        },
                        UIComponent.Container {
                            align = { AlignHorizontal.Stretch, AlignVertical.Top },
                            padding = { 0, 50 },
                            margin = { 0, 0 },
                            stackDirection = Enums.UI.StackDirection.Vertical,
                            heightInLayout = 7 / 10,
                            color = {
                                background = Color(0, 0, 0, 0.3)
                            },
                            contents = {
                                UIComponent.Button_MainMenu {
                                    title = "Test Seed 1",
                                    width = getButtonWidth,
                                    height = getButtonHeight,
                                    align = { AlignHorizontal.Center, AlignVertical.Center },
                                    callback = function() newGame(11487961515238620437ULL) end
                                },
                                UIComponent.Button_MainMenu {
                                    title = "Test Seed 2",
                                    width = getButtonWidth,
                                    height = getButtonHeight,
                                    align = { AlignHorizontal.Center, AlignVertical.Center },
                                    callback = function() newGame(6934033808124312024ULL) end
                                },
                                UIComponent.Button_MainMenu {
                                    title = "Test Seed 3",
                                    width = getButtonWidth,
                                    height = getButtonHeight,
                                    align = { AlignHorizontal.Center, AlignVertical.Center },
                                    callback = function() newGame(8616071071418665380ULL) end
                                },
                                UIComponent.Button_MainMenu {
                                    title = "Random Seed",
                                    width = getButtonWidth,
                                    height = getButtonHeight,
                                    align = { AlignHorizontal.Center, AlignVertical.Center },
                                    callback = function() newGame() end
                                }
                            }
                        },
                        UIComponent.Container {
                            align = { AlignHorizontal.Stretch, AlignVertical.Stretch },
                            childrenAlign = { AlignHorizontal.Center, AlignVertical.Center },
                            padding = { 0, 0 },
                            margin = { 0, 0 },
                            heightInLayout = 1 / 10,
                            stackDirection = Enums.UI.StackDirection.Vertical,
                            contents = {}
                        }
                    }
                },
            }
        }
    }
}

PlayView:addContent(playGrid)

return PlayView
