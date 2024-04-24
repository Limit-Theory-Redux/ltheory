---@type UIPage
local MainMenu = UICore.Page {
    name = "Main_Menu"
}

function MainMenu:onInput() end
function MainMenu:onUpdate(dt) end

local menuGrid = UILayout.Grid {
    align = { AlignHorizontal.Stretch, AlignVertical.Stretch },
    padding = { 50, 50 },
    margin = { 0, 0 },
    stackDirection = Enums.UI.StackDirection.Horizontal,
    showGrid = false,
    contents = {
        UIComponent.Container {
            align = { AlignHorizontal.Stretch, AlignVertical.Center },
            padding = { 5, 5 },
            margin = { 0, 0 },
            stackDirection = Enums.UI.StackDirection.Vertical,
            contents = {
                UIComponent.Button { title = "Play", width = 200, height = 40, align = { AlignHorizontal.Center, AlignVertical.Center } },
                UIComponent.Button { title = "Settings", width = 200, height = 40, align = { AlignHorizontal.Center, AlignVertical.Center } },
                UIComponent.Button { title = "Background Mode", width = 200, height = 40, align = { AlignHorizontal.Center, AlignVertical.Center } },
                UIComponent.Button { title = "Exit", width = 200, height = 40, align = { AlignHorizontal.Center, AlignVertical.Center } }
            }
        },
        UIComponent.Container {
            align = { AlignHorizontal.Stretch, AlignVertical.Center },
            padding = { 0, 0 },
            margin = { 0, 0 },
            stackDirection = Enums.UI.StackDirection.Vertical,
            contents = {
                UIComponent.Text { text = "Hello2", align = { AlignHorizontal.Center, AlignVertical.Center } }
            }
        }
    }
}

MainMenu:addContent(menuGrid)

return MainMenu
