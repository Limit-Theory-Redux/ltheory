local Test = require('States.Application')
local UIBuilder = require('UI.HmGui.UIBuilder')

local rng = RNG.FromTime()

local useRenderer = true
local testStackDirection = rng:choose({ Enums.UI.StackDirection.X, Enums.UI.StackDirection.Y })

local time = 0
local debugTestShowInS = 1

local function testCallback()
    print("Executed Callback")
    Test.callbackTest = "Callback 1"
    time = 0
end

local function testCallback2()
    print("Executed Callback 2")
    Test.callbackTest = "Callback 2"
    time = 0
end

local function testCallback3()
    print("Executed Callback 3")
    Test.callbackTest = "Callback 3"
    time = 0
end

local function testCallback4()
    print("Executed Callback 4")
    Test.callbackTest = "Callback 4"
    time = 0
end

local function testCallback5()
    print("Executed Callback 5")
    Test.callbackTest = "Callback 5"
    time = 0
end

local function testCallback6()
    print("Executed Callback 6")
    Test.callbackTest = "Callback 6"
    time = 0
end

local testCallbacks = {
    ["testCallback4"] = testCallback4,
    ["testCallback5"] = testCallback5,
    ["testCallback6"] = testCallback6
}

---@type UIComponentContainer
local testContainer = UIComponent.Container {
    padding = { 10, 10 },
    align = { AlignHorizontal.Center, AlignVertical.Center },
    stackDirection = testStackDirection,
    contents = {
        [1] = UIComponent.Text { font = "Exo2Bold", size = 14, color = Color(1, 1, 1, 1),
            text = "Hello World!" },
        [2] = UIComponent.Spacer { size = 16 },
        [3] = UIComponent.Text { font = "Exo2Bold", size = 12, color = Color(1, .4, .4, 1), text = "Hey!" },
        [4] = UIComponent.Button { title = "Button", width = 120, callback = testCallback }
    }
}

---@type UIComponentContainer
local testContainer2 = UIComponent.Container {
    align = { AlignHorizontal.Center, AlignVertical.Center },
    padding = { 10, 10 },
    stackDirection = testStackDirection,
    contents = {
        [1] = UIComponent.Text { font = "Exo2Bold", size = 14, color = Color(1, 1, 1, 1),
            text = "Hello World 2!" },
        [2] = UIComponent.Spacer { size = 16 },
        [3] = UIComponent.Text { font = "Exo2Bold", size = 12, color = Color(1, .4, .4, 1), text = "Hey 2!" },
        [4] = UIComponent.Button { title = "Button", width = 120, callback = testCallback2 }
    }
}

---@type UIComponentContainer
local testContainer3 = UIComponent.Container {
    align = { AlignHorizontal.Center, AlignVertical.Center },
    padding = { 10, 10 },
    stackDirection = testStackDirection,
    contents = {
        [1] = UIComponent.Text { font = "Exo2Bold", size = 14, color = Color(1, 1, 1, 1),
            text = "Hello World 3!" },
        [2] = UIComponent.Spacer { size = 16 },
        [3] = UIComponent.Text { font = "Exo2Bold", size = 12, color = Color(1, .4, .4, 1), text = "Hey 3!" },
        [4] = UIComponent.Button { title = "Button", width = 120, callback = testCallback3 }
    }
}

-- random example for containers from a loop
local generateContainersFromLoop = function()
    local containers = {}

    for id = 4, 6 do
        local container = UIComponent.Container {
            align = { AlignHorizontal.Center, AlignVertical.Center },
            padding = { 10, 10 },
            stackDirection = testStackDirection,
            contents = {
                [1] = UIComponent.Text { font = "Exo2Bold", size = 14, color = Color(1, 1, 1, 1),
                    text = "Hello World " .. id .. "!" },
                [2] = UIComponent.Spacer { size = 16 },
                [3] = UIComponent.Text { font = "Exo2Bold", size = 12, color = Color(1, .4, .4, 1), text = "Hey " .. id .. "!" },
                [4] = UIComponent.Button { title = "Button", width = 120, callback = testCallbacks["testCallback" .. id] }
            }
        }
        table.insert(containers, container)
    end

    return table.unpack(containers)
end

local createWindowCounter = 0

local function createWindow()
    testStackDirection = rng:choose({ Enums.UI.StackDirection.X, Enums.UI.StackDirection.Y })
    createWindowCounter = createWindowCounter + 1
    local testWindow = UIBuilder:buildWindow {
        title = "UI Builder Test " .. createWindowCounter,
        stackDirection = testStackDirection,
        canClose = true,
        containers = {
            testContainer,
            testContainer2,
            testContainer3
        }
    }

    UIBuilder:addWindowToPage {
        page = "TestPage",
        window = testWindow
    }
end

local function switchPage()
    local currentPage = UIBuilder:getCurrentPageName()
    local lastPage = UIBuilder:getLastPageName()

    -- demo for switching pages & using last page
    if lastPage == currentPage then
        local availablePages = UIBuilder:getAvailablePages()

        for _, name in ipairs(availablePages) do
            if currentPage ~= name then
                UIBuilder:setCurrentPage(name)
                break
            end
        end
    else
        UIBuilder:setCurrentPage(lastPage)
    end
end

local switchPageBackContainer = function()
    -- demo for programmatically working with pages/contents while keeping everything clean and understandable
    local availablePages = UIBuilder:getAvailablePages()
    local contentTable = {}

    table.insert(contentTable,
        UIComponent.Text { font = "Exo2Bold", size = 12, color = Color(1, 1, 1, 1), text = "Available Pages" })
    table.insert(contentTable, UIComponent.Spacer { size = 4 })

    for _, name in ipairs(availablePages) do
        table.insert(contentTable,
            UIComponent.Text { font = "Exo2Bold", size = 10, color = Color(1, .4, .4, 1), text = name })
    end

    table.insert(contentTable,
        UIComponent.Button { title = "Switch back to Page 1", width = 120, callback = switchPage })

    return UIComponent.Container {
        align = { AlignHorizontal.Center, AlignVertical.Center },
        padding = { 10, 10 },
        stackDirection = Enums.UI.StackDirection.X,
        contents = contentTable
    }
end

local createWindowContainer = UIComponent.Container {
    align = { AlignHorizontal.Center, AlignVertical.Center },
    padding = { 10, 10 },
    stackDirection = testStackDirection,
    contents = {
        [1] = UIComponent.Button { title = "Create Window", width = 120, callback = createWindow },
        [2] = UIComponent.Button { title = "Switch Page", width = 120, callback = switchPage }
    }
}

function Test:onInit()
    self.renderer = Renderer()

    --Init UIBuilder

    UIBuilder:buildPage {
        name = "TestPage"
    }

    -- add another page to demo page switching
    UIBuilder:buildPage {
        name = "TestPage2"
    }

    local uiBuilderWindow = UIBuilder:buildWindow {
        title = "UI Builder Test Tools",
        stackDirection = rng:choose({ Enums.UI.StackDirection.X, Enums.UI.StackDirection.Y }),
        containers = {
            createWindowContainer
        }
    }

    -- page 2 demo - windows
    local uiBuilderWindow2 = UIBuilder:buildWindow {
        title = "Page 2 - Window 1",
        stackDirection = rng:choose({ Enums.UI.StackDirection.X, Enums.UI.StackDirection.Y }),
        containers = {
            switchPageBackContainer(),
        }
    }

    local uiBuilderWindow3 = UIBuilder:buildWindow {
        title = "Page 2 - Window 2",
        stackDirection = rng:choose({ Enums.UI.StackDirection.X, Enums.UI.StackDirection.Y }),
        canClose = true,
        containers = {
            generateContainersFromLoop(),
        }
    }

    UIBuilder:addWindowToPage {
        page = "TestPage",
        window = uiBuilderWindow
    }

    UIBuilder:addWindowToPage {
        page = "TestPage2",
        window = uiBuilderWindow2
    }

    UIBuilder:addWindowToPage {
        page = "TestPage2",
        window = uiBuilderWindow3
    }

    UIBuilder:setCurrentPage("TestPage")
end

function Test:onInput()
end

function Test:onUpdate(dt)
    if self.callbackTest then
        time = time + dt
    end

    Gui:beginGui(self.resX, self.resY, InputInstance)
    UIBuilder:render()
    Gui:endGui(InputInstance)
end

function Test:onDraw()
    if useRenderer then
        self.renderer:start(self.resX, self.resY)
        Viewport.Push(0, 0, self.resX, self.resY, true)
        Gui:draw()

        if self.callbackTest then
            UI.DrawEx.TextAdditive(
                'NovaRound',
                self.callbackTest,
                20,
                self.resX / 2 - 24, 128, 40, 20,
                1, .4, .4, 1,
                0.5, 0.5
            )

            if time >= debugTestShowInS then
                self.callbackTest = nil
                time = 0
            end
        end

        Viewport.Pop()
        self.renderer:stop()
        self.renderer:present(0, 0, self.resX, self.resY)
    else
        Gui:draw()
    end
end

return Test
