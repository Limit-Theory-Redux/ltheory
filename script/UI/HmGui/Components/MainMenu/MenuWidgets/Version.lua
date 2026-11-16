local MenuStyle = require("UI.HmGui.Styles.MenuStyle")
local Label = require("UI.HmGui.Components.MainMenu.MenuWidgets.Label")
local C = MenuStyle.color

---The game version, bottom left in the sidebar.
local function Version()
    return Label {
        x = 12, y = 1082, size = 18, font = "regular", color = C.dim,
        text = function() return "v. " .. tostring(Config.gameVersion) end
    }
end

-- Add to the global UIComponent table
UIComponent.MenuVersion = Version

return Version
