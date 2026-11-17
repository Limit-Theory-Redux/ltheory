---@type UIView
local CreditsView = UICore.View {
    name = "Credits",
    noDarken = true -- the modal carries its own background
}

---@type UIRouter
local UIRouter = require("UI.HmGui.UICore.UIRouter")
local MenuStyle = require("UI.HmGui.Styles.MenuStyle")

local logo = Tex2D.Load("./res/images/LTR-logo-full-hori.png")

function CreditsView:onInput() end
function CreditsView:onUpdate(dt) end
function CreditsView:onViewOpen(isPageOpen) end
function CreditsView:onViewClose(isPageClose) end

local function switchToMainScreen()
    UIRouter:getCurrentPage():setView("Main")
end

-- The credits (Config/Credits.lua) as lines: a heading per group, then each
-- run of people with the same role under that role as a subheading.
local headerOrder = { "BASED ON THE IDEAS & WORKS OF", "MAINTAINERS", "CONTRIBUTORS" }

local function creditLines()
    local headers = {}
    local listed = {}
    for _, header in ipairs(headerOrder) do
        if type(Config.credits[header]) == "table" then
            headers[#headers + 1] = header
            listed[header] = true
        end
    end
    local others = {}
    for header, people in pairs(Config.credits) do
        if type(people) == "table" and not listed[header] then others[#others + 1] = header end
    end
    table.sort(others)
    for _, header in ipairs(others) do headers[#headers + 1] = header end

    local lines = {}
    for _, header in ipairs(headers) do
        lines[#lines + 1] = { kind = "heading", text = MenuStyle.sentenceCase(header) }
        local lastRole
        for _, person in ipairs(Config.credits[header]) do
            if person.teams and person.teams ~= lastRole then
                lines[#lines + 1] = { kind = "role", text = person.teams }
                lastRole = person.teams
            end
            lines[#lines + 1] = { kind = "name", text = person.name }
        end
    end
    return lines
end

local lines = creditLines()

local MODAL_W = 590

local creditsList = UIComponent.MenuRaw {
    x = 0, y = 180, w = MODAL_W, h = 730,
    fn = function(ctx)
        local C = MenuStyle.color
        Gui:beginVerticalContainer()
        Gui:setAlignment(AlignHorizontal.Stretch, AlignVertical.Top)
        Gui:setChildrenAlignment(AlignHorizontal.Center, AlignVertical.Top)
        for _, line in ipairs(lines) do
            if line.kind == "heading" then
                Gui:text(line.text, MenuStyle.fontOf("light", 37), C.heading)
                Gui:setAlignment(AlignHorizontal.Center, AlignVertical.Top)
                Gui:setMarginEx(0, 16, 0, 4)
            elseif line.kind == "role" then
                Gui:text(line.text, MenuStyle.fontOf("light", 27), C.subheading)
                Gui:setAlignment(AlignHorizontal.Center, AlignVertical.Top)
                Gui:setMarginEx(0, 8, 0, 2)
            else
                Gui:text(line.text, MenuStyle.fontOf("light", 25), C.text)
                Gui:setAlignment(AlignHorizontal.Center, AlignVertical.Top)
                Gui:setMarginEx(0, 3, 0, 3)
            end
        end
        Gui:endContainer()
    end
}

CreditsView:addContent(UIComponent.MenuScreen {
    contents = {
        UIComponent.MenuPanel {
            x = 705, y = 55, w = MODAL_W, h = 1010, band = 172,
            contents = {
                UIComponent.MenuImage { x = 64, y = 42, w = 462, image = logo },
                UIComponent.MenuClose { x = MODAL_W - 48, y = 6, callback = switchToMainScreen },
                creditsList,
                UIComponent.MenuButton { x = 226, y = 930, w = 138, h = 44, size = 26, title = "Close", callback = switchToMainScreen },
            }
        },
    }
})

return CreditsView
