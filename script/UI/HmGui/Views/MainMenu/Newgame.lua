---@type UIView
local NewgameView = UICore.View {
    name = "Newgame",
    noDarken = true -- the sidebar and the panel carry their own backgrounds
}

---@type UIRouter
local UIRouter = require("UI.HmGui.UICore.UIRouter")
local MenuStyle = require("UI.HmGui.Styles.MenuStyle")
---@type RandomNumberGenerator
local rng = RNG.FromTime()

-- Known seeds for testing, picked in the Test Seed dropdown (which then
-- overrides the Universe Seed field). TODO: replace with a proper list.
local testSeeds = {
    { "Test Seed 1",                                      11487961515238620437ULL },
    { "Test Seed 2",                                      6934033808124312024ULL },
    { "Test Seed 3",                                      8616071071418665380ULL },
    { "Test Seed 4",                                      17682038400513250095ULL },
    { "KEEP black",                                       5022463494542550306ULL },  -- good for testing dynamic lighting
    { "KEEP red",                                         5012768293123392491ULL },
    { "KEEP blue and milky white",                        4933876146649964811ULL },
    { "MAYBE orange-ish",                                 2008422628673393673ULL },
    { "KEEP gold-yellow",                                 5712598467986491931ULL },
    { "KEEP milky-white and light blue",                  8272263000674654607ULL },  -- really pretty
    { "KEEP bluish-green with a bright gold star",        14169804077813660835ULL },
    { "KEEP violet",                                      9806676695553338612ULL },
    { "KEEP blue",                                        14600758714913275339ULL },
    { "KEEP bright green",                                11589761683708427350ULL },
    { "KEEP blue-red-orange",                             3432712644463072838ULL },
    { "MAYBE 'Hubble palette'",                           10630444862697458122ULL },
    { "KEEP even bluish-white with a bright yellow star", 5199604093543988311ULL },
    { "KEEP completely dark with one small blue star",    9471911754066691691ULL },
    { "looks pretty cool",                                15887563511063255006ULL },
    { "looks pretty cool too",                            976665863517979971ULL },
}

local testSeedNames = { "None" }
for _, entry in ipairs(testSeeds) do testSeedNames[#testSeedNames + 1] = entry[1] end

-- 2^64 - 1, the largest seed
local MAX_SEED = 18446744073709551615ULL

---The seed as the player sees it: digits, without LuaJIT's `ULL` suffix.
---@param seed integer
---@return string
local function seedToText(seed)
    return (tostring(seed):gsub("U?LL$", ""))
end

---The seed a text stands for. A whole number from 0 to 2^64 - 1 is used as it is;
---any other text (a word, a longer number) is hashed, so it works as a seed too.
---Empty text gives nil: a random seed.
---@param text string
---@return integer|nil
local function textToSeed(text)
    text = text:match("^%s*(.-)%s*$")
    if text == "" then
        return nil
    end
    if text:match("^%d+$") then
        local seed = 0ULL
        for i = 1, #text do
            local digit = text:byte(i) - 48
            if seed > (MAX_SEED - digit) / 10ULL then
                return Hash.FNVStr64(text) -- larger than 2^64 - 1
            end
            seed = seed * 10ULL + digit
        end
        return seed
    end
    return Hash.FNVStr64(text)
end

-- THE SEED FIELD
-- 1 = None: the player's own seed in the field. Otherwise a test seed, which
-- the field shows (dimmed, read-only) instead of the player's text.
local testSeedIndex = 1
local ownSeedText = ""

local FIELD_W, FIELD_H = 340, 40
local seedView -- rebuilt when the window size changes the font size, or the field's state changes
local seedViewKey

local function usingTestSeed()
    return testSeedIndex > 1
end

---The text in a TextView. `TextData:text()` gives a C string (`const char *`),
---not a Lua string, so it is copied into one.
---@return string
local function textOf(textView)
    return ffi.string(textView.state.textData():text())
end

local function fieldText()
    if usingTestSeed() then
        return seedToText(testSeeds[testSeedIndex - 1][2])
    end
    return ownSeedText
end

---The TextView for the field at this scale (font size), made again when needed.
local function getSeedView(s)
    local size = math.max(8, math.floor(24 * s + 0.5))
    -- a new TextView (new text data) for every choice: the engine copies new
    -- text data into the field, also into an editable one
    local key = size .. (usingTestSeed() and (":test" .. testSeedIndex) or ":own")
    if seedViewKey ~= key then
        if seedView and not usingTestSeed() and seedViewKey and seedViewKey:find(":own") then
            ownSeedText = textOf(seedView) -- keep what the player typed
        end
        seedView = UIComponent.TextView {
            text = fieldText(),
            editable = not usingTestSeed(),
            multiline = false,
            style = {
                font = { size = size, family = "Unageo" },
                brush = usingTestSeed() and MenuStyle.color.disabled or MenuStyle.color.fieldText,
                selectionBrush = MenuStyle.color.selection,
                cursorBrush = MenuStyle.color.text,
            },
            width = function() return (FIELD_W - 20) * s end,
            align = { AlignHorizontal.Left, AlignVertical.Center },
            -- with a callback, the TextView copies what is typed into its text data
            callback = function() end,
        }
        seedViewKey = key
    end
    return seedView
end

---The player's own seed text (also while a test seed hides it).
local function getOwnSeedText()
    if seedView and not usingTestSeed() then
        return textOf(seedView)
    end
    return ownSeedText
end

local function setOwnSeedText(text)
    ownSeedText = text
    if seedView and not usingTestSeed() then
        seedView.state.textData():setText(text)
    end
end

local function rollSeed()
    setOwnSeedText(seedToText(rng:get64()))
end

local function setTestSeed(index)
    if not usingTestSeed() then
        ownSeedText = getOwnSeedText()
    end
    testSeedIndex = index
end

-- A new random seed each time the view opens, so the player can see (and note
-- down or share) the seed of the game they start.
function NewgameView:onInput() end
function NewgameView:onUpdate(dt) end
function NewgameView:onViewOpen(isPageOpen) rollSeed() end
function NewgameView:onViewClose(isPageClose) end

local function switchToMainScreen()
    UIRouter:getCurrentPage():setView("Main")
end

local function startNewGame()
    local seed
    if usingTestSeed() then
        seed = testSeeds[testSeedIndex - 1][2]
    else
        seed = textToSeed(getOwnSeedText())
    end
    seed = seed or rng:get64()

    -- Route through the app state's scene builder (UniverseManager +
    -- Rulesets + SolarSystemVisualizer) via the seam the state exposes
    -- in onInit (GameState.startGame).
    if GameState.startGame then
        GameState.startGame(seed)
    end
end

local seedField = UIComponent.MenuRaw {
    x = 305, y = 130, w = FIELD_W, h = FIELD_H,
    fn = function(ctx)
        local s = ctx.s
        local C = MenuStyle.color
        Gui:setBackgroundColor(usingTestSeed() and C.box or C.field)
        Gui:setBorderWidth(MenuStyle.border(2, s))
        Gui:setBorderColor(C.fieldBorder)
        Gui:beginStackContainer()
        Gui:setAlignment(AlignHorizontal.Left, AlignVertical.Center)
        Gui:setMarginEx(10 * s, 0, 0, 0)
        getSeedView(s):render()
        Gui:endContainer()
    end
}

NewgameView:addContent(UIComponent.MenuScreen {
    contents = {
        UIComponent.MenuSidebar {
            contents = {
                UIComponent.MenuLabel { x = 0, y = 88, w = MenuStyle.SIDEBAR_WIDTH, h = 70, size = 54, align = "center", text = "NEW GAME", color = MenuStyle.color.text },
                UIComponent.MenuMainNav { selected = 1 },
                UIComponent.MenuButton { x = 17, y = 996, w = 140, h = 44, title = "Back", callback = switchToMainScreen },
                UIComponent.MenuButton {
                    x = 178, y = 996, w = 140, h = 44, title = "Start", style = "go",
                    toolTip = "Starts a new game with the universe seed (or the test seed).",
                    callback = startNewGame
                },
            }
        },
        UIComponent.MenuPanel {
            x = 432, y = 52, w = 1508, h = 1018,
            contents = {
                UIComponent.MenuLabel { x = 100, y = 130, h = 40, text = "Universe Seed" },
                seedField,
                UIComponent.MenuButton {
                    x = 665, y = 129, w = 150, h = 42, size = 26, uppercase = false, title = "New Seed",
                    enabled = function() return not usingTestSeed() end,
                    toolTip = "Picks a new random seed.\nYou can also type a number, or any text.",
                    callback = rollSeed
                },
                UIComponent.MenuLabel { x = 100, y = 192, h = 40, text = "Test Seed" },
                UIComponent.MenuDropdown {
                    x = 305, y = 192, w = FIELD_W,
                    toolTip = "Known seeds for testing. A test seed replaces the Universe Seed.",
                    selections = testSeedNames,
                    getIndex = function() return testSeedIndex end,
                    setIndex = setTestSeed
                },
                UIComponent.MenuLabel {
                    x = 305, y = 236, h = 30, size = 19, font = "regular", color = MenuStyle.color.dim,
                    text = "Starts with this test seed. Pick None to use your own.",
                    visible = usingTestSeed
                },
                UIComponent.MenuSlider {
                    x = 100, y = 304, label = "Nebula Brightness",
                    toolTip = "1 = low brightness, 10 = high brightness.",
                    min = 1, max = 10, increment = 1,
                    format = function(v) return string.format("%d", math.floor(v + 0.5)) end,
                    get = function() return GameState.gen.nebulaBrightnessScale end,
                    set = function(v) GameState.gen.nebulaBrightnessScale = v end
                },
            }
        },
    }
})

return NewgameView
