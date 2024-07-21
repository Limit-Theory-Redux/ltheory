---@type UIPage
local LoadingScreen = UICore.Page {
    name = "Loading_Screen"
}

---@type UIRouter
local UIRouter = require("UI.HmGui.UICore.UIRouter")
---@type MusicPlayer
local MusicPlayer = require("Systems.SFX.MusicPlayer")

local logo = Tex2D.Load("./res/images/LTR-logo-name.png")

function LoadingScreen:onInput() end

function LoadingScreen:onPageOpen()
    ---@type Universe
    local Universe = require("Systems.Universe.Universe")

    GameState:SetState(Enums.GameStates.LoadingScreen)

    Log.Debug("LimitTheoryRedux: playAmbient")
    MusicPlayer:playAmbient()

    -- create star system from universe with economy
    --* replace with proper loading mechanism (async) at some point
    GameState.gen.debug       = true --! temporary use config values for system generation

    -- Use the "real" system generation sizes for a gameplay star system
    Config.gen.scaleSystem    = Config.gen.scaleSystemReal
    Config.gen.scalePlanet    = Config.gen.scalePlanetReal
    Config.gen.scalePlanetMod = Config.gen.scalePlanetModReal
    GameState.render.zNear    = Config.gen.zNearReal
    GameState.render.zFar     = Config.gen.zFarReal

    -- create star system with economy
    Universe:createStarSystem(true)

    --! currently loading screen doesn´t render because the star system generation and game rendering are in sync
    -- todo: handle quickstart logic

    UIRouter:getPage("Gameplay"):setView("Ship_Creation")
    UIRouter:setCurrentPage("Gameplay")
end

function LoadingScreen:onPageClose() end

function LoadingScreen:onUpdate(dt) end

local function calculateScaledSize(containerWidth, containerHeight, imageWidth, imageHeight)
    local containerRatio = containerWidth / containerHeight
    local imageRatio = imageWidth / imageHeight

    if containerRatio > imageRatio then
        local scaleFactor = containerHeight / imageHeight
        return imageWidth * scaleFactor, imageHeight * scaleFactor
    else
        local scaleFactor = containerWidth / imageWidth
        return imageWidth * scaleFactor, imageHeight * scaleFactor
    end
end

local function getScaledImageSize(containerWidth, containerHeight, imageWidth, imageHeight)
    local availableWidth = containerWidth
    local availableHeight = containerHeight

    local newImageWidth, newImageHeight = calculateScaledSize(availableWidth, availableHeight, imageWidth, imageHeight)
    return newImageWidth, newImageHeight
end

local titleContainer = UIComponent.Container {
    align = { AlignHorizontal.Stretch, AlignVertical.Stretch },
    childrenAlign = { AlignHorizontal.Center, AlignVertical.Center },
    padding = { 400, 400 },
    margin = { 0, 0 },
    layoutType = GuiLayoutType.Vertical,
    contents = {
        UIComponent.RawInput { fn = function()
            Gui:beginStackContainer()
            Gui:image(logo)
            Gui:setFixedSize(getScaledImageSize((GameState.render.resX - 400),
                (GameState.render.resY - 400), 1240, 240))
            Gui:endContainer()
        end
        }
    }
}

LoadingScreen:addContent(titleContainer)

return LoadingScreen
