---@type UIView
local LoadgameView = UICore.View {
    name = "Loadgame",
    noDarken = true -- the sidebar and the modal carry their own backgrounds
}

---@type UIRouter
local UIRouter = require("UI.HmGui.UICore.UIRouter")
local MenuStyle = require("UI.HmGui.Styles.MenuStyle")

-- The saved games list from the start menu mockups. The game can't save yet, so
-- it says so; the test seeds that used to be listed here are in New Game.

function LoadgameView:onInput() end
function LoadgameView:onUpdate(dt) end
function LoadgameView:onViewOpen(isPageOpen) end
function LoadgameView:onViewClose(isPageClose) end

local function switchToMainScreen()
    UIRouter:getCurrentPage():setView("Main")
end

local MODAL_W = 590

LoadgameView:addContent(UIComponent.MenuScreen {
    contents = {
        UIComponent.MenuSidebar {
            contents = {
                UIComponent.MenuLabel { x = 0, y = 88, w = MenuStyle.SIDEBAR_WIDTH, h = 70, size = 54, align = "center", text = "LOAD GAME", color = MenuStyle.color.text },
                UIComponent.MenuMainNav { selected = 2 },
                UIComponent.MenuVersion(),
            }
        },
        UIComponent.MenuPanel {
            x = 958, y = 108, w = MODAL_W, h = 912, band = 122,
            contents = {
                UIComponent.MenuClose { x = MODAL_W - 48, y = 6, callback = switchToMainScreen },
                UIComponent.MenuLabel { x = 0, y = 36, w = MODAL_W, h = 56, size = 40, align = "center", text = "LOAD GAME", color = MenuStyle.color.text },
                UIComponent.MenuLabel { x = 0, y = 380, w = MODAL_W, h = 44, size = 30, align = "center", text = "No saved games yet" },
                UIComponent.MenuLabel {
                    x = 0, y = 430, w = MODAL_W, h = 30, size = 20, font = "regular", align = "center", color = MenuStyle.color.dim,
                    text = "Saving isn't in the game yet."
                },
                UIComponent.MenuLabel {
                    x = 0, y = 460, w = MODAL_W, h = 30, size = 20, font = "regular", align = "center", color = MenuStyle.color.dim,
                    text = "Your saved games will be listed here."
                },
                UIComponent.MenuButton { x = 229, y = 836, w = 132, h = 44, size = 26, title = "Close", callback = switchToMainScreen },
            }
        },
    }
})

return LoadgameView
