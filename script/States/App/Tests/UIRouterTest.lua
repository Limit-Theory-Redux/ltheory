local Test = require('States.Application')
local SoundManager = require('Systems.SFX.SoundManager')
local MusicPlayer = require('Systems.SFX.MusicPlayer')
local UIRouter = require('UI.HmGui.UICore.UIRouter')
local UIPageExample = require('script.UI.HmGui.Pages.Example')
local UIPageMainMenu = require('script.UI.HmGui.Pages.MainMenu')

local useRenderer = true

function Test:onInit()
    self.renderer = Renderer()

    SoundManager:init()
    MusicPlayer:Init() --todo: fix all casing errors

    -- set initial view
    UIPageExample:setView("Main")
    UIPageMainMenu:setView("Title")

    -- add page
    UIRouter:addPage(UIPageExample)
    UIRouter:addPage(UIPageMainMenu)
    UIRouter:setCurrentPage("Example")
end

function Test:onInput(dt)
    UIRouter:input(dt)
end

function Test:onUpdate(dt)
    SoundManager:clean(dt)
    MusicPlayer:OnUpdate(dt) --todo fix casing

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
