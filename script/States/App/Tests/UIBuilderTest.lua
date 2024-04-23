local Test = require('States.Application')
local UIRouter = require('UI.HmGui.UICore.UIRouter')
local UIPageMainMenu = require('UI.HmGui.Pages.MainMenu')

--todo: to be restructured as ui builder example

local rng = RNG.FromTime()

local useRenderer = true

function Test:onInit()
    self.renderer = Renderer()

    -- set initial view
    UIPageMainMenu:setView("Main")

    -- add page
    UIRouter:addPage(UIPageMainMenu)
    UIRouter:setCurrentPage("Main_Menu")
end

function Test:onInput(dt)
    UIRouter:input(dt)
end

function Test:onUpdate(dt)
    Gui:beginGui(self.resX, self.resY, InputInstance)
    UIRouter:update(dt)
    Gui:endGui(InputInstance)
end

function Test:onDraw()
    if useRenderer then
        self.renderer:start(self.resX, self.resY)
        Viewport.Push(0, 0, self.resX, self.resY, true)
        Gui:draw()
        Viewport.Pop()
        self.renderer:stop()
        self.renderer:present(0, 0, self.resX, self.resY)
    else
        Gui:draw()
    end
end

return Test
