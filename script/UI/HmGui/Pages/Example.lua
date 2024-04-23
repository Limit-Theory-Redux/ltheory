---@type UIPage
local Example = UICore.Page {
    name = "Example"
}

local OtherView = require("UI.HmGui.Views.Example.OtherView")
Example:addViewToPage(OtherView)
local MainView = require("UI.HmGui.Views.Example.Main")
Example:addViewToPage(MainView)

return Example
