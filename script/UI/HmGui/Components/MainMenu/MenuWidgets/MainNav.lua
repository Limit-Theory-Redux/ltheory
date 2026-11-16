local Nav = require("UI.HmGui.Components.MainMenu.MenuWidgets.Nav")

---The main menu's nav (New Game … Exit Game), shared by the Main, New Game and
---Load Game screens.
---@param args { y: number|nil, selected: integer|nil }
local function MainNav(args)
    local function goTo(view)
        return function()
            require("UI.HmGui.UICore.UIRouter"):getCurrentPage():setView(view)
        end
    end
    return Nav {
        y = args.y or 225,
        selected = args.selected,
        items = {
            { title = "New Game",  callback = goTo("Newgame") },
            { title = "Load Game", callback = goTo("Loadgame") },
            { title = "Settings",  callback = goTo("Settings") },
            { title = "Credits",   callback = goTo("Credits") },
            { title = "Exit Game", callback = function() LimitTheoryRedux:exit() end }, -- pre-exit operations, then exit
        }
    }
end

-- Add to the global UIComponent table
UIComponent.MenuMainNav = MainNav

return MainNav
