---@type UIView
local ShipCreation = UICore.View {
    name = "Ship_Creation"
}

---@type UIRouter
local UIRouter = require("UI.HmGui.UICore.UIRouter")

-- todo: handle quickstart logic

function ShipCreation:onInput()
    ---@type Universe
    local Universe = require("Systems.Universe.Universe")

    if InputInstance:isPressed(Button.KeyboardB) then
        if GameState.player.currentShip then
            GameState.player.currentShip:delete() --todo: this needs a fix, ships stay around
        end

        local shipObject = {
            owner = GameState.player.humanPlayer,
            shipName = GameState.player.humanPlayerShipName,
            friction = 0,
            sleepThreshold = {
                [1] = 0,
                [2] = 0
            }
        }

        GameState.player.currentShip = Universe:createShip(GameState.world.currentSystem, nil, shipObject)
    elseif InputInstance:isPressed(Button.KeyboardF) then
        -- Insert the game view into the application canvas to make it visible
        GameState.render.gameView = Systems.Overlay.GameView(GameState.player.humanPlayer, GameState.audio.manager)

        GameState.render.uiCanvas = UI.Canvas()
        GameState.render.uiCanvas
            :add(GameState.render.gameView
                :add(Systems.Controls.Controls.MasterControl(GameState.render.gameView, GameState.player.humanPlayer))
            )
        GameState.render.gameView:setCameraMode(Enums.CameraMode.FirstPerson)

        UIRouter:getCurrentPage():setView("In_Game")
    end
end

function ShipCreation:onUpdate(dt) end

--! ONLY WORKS ON THE FIRST GAME LOAD, WHEN GOING BACK TO MAIN MENU AND THEN LOADING AGAIN THE CAMERA WILL STAY IN FIRST PERSON. WHY?
function ShipCreation:onViewOpen(isPageOpen)
    if isPageOpen then
        return
    end

    GameState:SetState(Enums.GameStates.ShipCreation)

    local shipObject = {
        owner = GameState.player.humanPlayer,
        shipName = GameState.player.humanPlayerShipName,
        friction = 0,
        sleepThreshold = {
            [1] = 0,
            [2] = 0
        }
    }

    ---@type Universe
    local Universe = require("Systems.Universe.Universe")

    -- add player to system via universe
    if not Universe:systemHasPlayer(GameState.world.currentSystem, GameState.player.humanPlayer) then
        Universe:playerEnterSystem(GameState.world.currentSystem, GameState.player.humanPlayer)
    end

    -- add ship / create ship in system via universe
    GameState.player.currentShip = Universe:createShip(GameState.world.currentSystem, nil, shipObject)

    GameState.render.gameView = Systems.Overlay.GameView(GameState.player.humanPlayer, GameState.audio.manager)
    GameState.render.uiCanvas = UI.Canvas()
    GameState.render.uiCanvas
        :add(GameState.render.gameView
            :add(Systems.Controls.Controls.GenTestControl(GameState.render.gameView, GameState.player.humanPlayer)))

    InputInstance:setCursorVisible(true)
end

function ShipCreation:onViewClose(isPageClose) end

local textContainer = UIComponent.Container {
    align = { AlignHorizontal.Stretch, AlignVertical.Stretch },
    childrenAlign = { AlignHorizontal.Center, AlignVertical.Bottom },
    padding = { 0, 100 },
    margin = { 0, 0 },
    stackDirection = Enums.UI.StackDirection.Vertical,
    contents = {
        UIComponent.RawInput { fn = function()
            Gui:setProperty(GuiProperties.TextFont, Cache.Font("Unageo-Medium", 20))
            Gui:setProperty(GuiProperties.TextColor, Color(1, 1, 1, 1.0))
            Gui:text("[F] SPAWN - [B] NEW SHIP")
        end }
    }
}

ShipCreation:addContent(textContainer)

return ShipCreation
