---@type UIView
local TitleView = UICore.View {
    name = "Title"
}

---@type UIRouter
local UIRouter = require("UI.HmGui.UICore.UIRouter")
local MusicPlayer = require('Systems.SFX.MusicPlayer')
local Bindings = require('States.ApplicationBindings')

local logo = Tex2D.Load("./res/images/LTR-logo-name.png")

local logoOpacity = 0
local textOpacity = 0
local logoReachedMaxOpacity = false
local logoScale = 0
local logoStartScale = 0.5
local logoTargetScale = 1.0
local timeOpened = nil
local timeLogoReachedMaxOpacity = nil
local animationInMs = 2000
local animationTextInMs = 2000
local animationInverse = false
local skipTitle = GameState.skipTitleScreen

local function inOutCubic(t, b, c, d)
    t = t / d * 2
    if t < 1 then
        return c / 2 * t * t * t + b
    else
        t = t - 2
        return c / 2 * (t * t * t + 2) + b
    end
end

function TitleView:onInput()
    if Bindings.All:get() == 1 then
        skipTitle = true
    end
end

function TitleView:onUpdate(dt)
    if not timeOpened then
        return
    end

    local elapsedTimeSinceOpen = timeOpened:getElapsedMs()

    if logoOpacity < 1 and not logoReachedMaxOpacity then
        logoOpacity = math.max(0, math.min(inOutCubic(elapsedTimeSinceOpen, 0, 1, 2000), 1))
        textOpacity = logoOpacity
    else
        if not timeLogoReachedMaxOpacity then
            logoReachedMaxOpacity = true
            timeLogoReachedMaxOpacity = TimeStamp.Now()
        end

        if timeLogoReachedMaxOpacity:getElapsedMs() < animationTextInMs then
            if not animationInverse then
                textOpacity = math.max(0,
                    math.min(inOutCubic(timeLogoReachedMaxOpacity:getElapsedMs(), 1, 0 - 1, animationTextInMs), 1))
            else
                textOpacity = math.max(0,
                    math.min(inOutCubic(timeLogoReachedMaxOpacity:getElapsedMs(), 0, 1 - 0, animationTextInMs), 1))
            end
        elseif timeLogoReachedMaxOpacity:getElapsedMs() > animationTextInMs then
            timeLogoReachedMaxOpacity = TimeStamp.Now()
            animationInverse = not animationInverse
        end
    end

    if elapsedTimeSinceOpen < animationInMs then
        logoScale = inOutCubic(elapsedTimeSinceOpen, logoStartScale, logoTargetScale - logoStartScale, animationInMs)
    else
        logoScale = logoTargetScale
    end

    if skipTitle then
        UIRouter:getCurrentPage():setView("Main")
    end
end

function TitleView:onViewOpen(isPageOpen)
    if isPageOpen then
        MusicPlayer:QueueTrack(GameState.audio.menuTheme, true)
        timeOpened = TimeStamp.Now()
    end
end

function TitleView:onViewClose(isPageClose)
    if isPageClose then
        MusicPlayer:ClearQueue()
    end
end

local function calculateScaledSize(containerWidth, containerHeight, imageWidth, imageHeight)
    local containerRatio = containerWidth / containerHeight
    local imageRatio = imageWidth / imageHeight

    if containerRatio > imageRatio then
        local scaleFactor = containerHeight / imageHeight
        return imageWidth * scaleFactor, imageHeight * scaleFactor
    else
        local scaleFactor = containerWidth / imageWidth
        return imageWidth * scaleFactor, imageHeight * scaleFactor
    end
end

local function getScaledImageSize(containerWidth, containerHeight, imageWidth, imageHeight)
    local availableWidth = containerWidth
    local availableHeight = containerHeight

    local newImageWidth, newImageHeight = calculateScaledSize(availableWidth, availableHeight, imageWidth, imageHeight)
    return newImageWidth, newImageHeight
end

local titleContainer = UIComponent.Container {
    align = { AlignHorizontal.Stretch, AlignVertical.Stretch },
    childrenAlign = { AlignHorizontal.Center, AlignVertical.Center },
    padding = { 400, 400 },
    margin = { 0, 0 },
    stackDirection = Enums.UI.StackDirection.Vertical,
    contents = {
        UIComponent.RawInput { fn = function()
            Gui:beginStackContainer()
            Gui:setBorder(0.0001, Color(1.0, 1.0, 1.0, logoOpacity)) --! using border as theres currently no other way
            Gui:image(logo)
            Gui:setFixedSize(getScaledImageSize((GameState.render.resX - 400) * logoScale,
                (GameState.render.resY - 400) * logoScale, 1240, 240))
            Gui:endContainer()
        end
        }
    }
}

local textContainer = UIComponent.Container {
    align = { AlignHorizontal.Stretch, AlignVertical.Stretch },
    childrenAlign = { AlignHorizontal.Center, AlignVertical.Bottom },
    padding = { 0, 100 },
    margin = { 0, 0 },
    stackDirection = Enums.UI.StackDirection.Vertical,
    contents = {
        UIComponent.RawInput { fn = function()
            Gui:setPropertyFont(GuiProperties.TextFont, Cache.Font("Unageo-Medium", 24))
            Gui:setPropertyColor(GuiProperties.TextColor, Color(1, 1, 1, textOpacity))
            Gui:text("PRESS ANY KEY TO CONTINUE")
        end }
    }
}

TitleView:addContent(titleContainer)
TitleView:addContent(textContainer)

return TitleView
