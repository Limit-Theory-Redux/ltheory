function MainMenu:ShowGui()
    -- Add title and Main Menu dialog
    local scaleFactor = (LTheoryRedux.resX / 22) / 72

    Gui:beginStackContainer() -- begin game window
    Gui:setAlignment(AlignHorizontal.Stretch, AlignVertical.Stretch)
    Gui:setPadding(10.0, 10.0)

    Gui:beginHorizontalContainer() -- begin main menu screen
    Gui:setAlignment(AlignHorizontal.Stretch, AlignVertical.Stretch)

    Gui:beginVerticalContainer() -- begin title/menu panel
    Gui:setPercentWidth(30)
    Gui:setVerticalAlignment(AlignVertical.Stretch)
    Gui:setChildrenHorizontalAlignment(AlignHorizontal.Stretch)

    -- Title
    Gui:beginStackContainer() -- begin title panel
    Gui:setPercentSize(99, 20)
    Gui:setAlignment(AlignHorizontal.Center, AlignVertical.Center)
    Gui:setChildrenAlignment(AlignHorizontal.Center, AlignVertical.Center)
    Gui:setBorder(1, Color(1.0, 1.0, 1.0, 1.0))
    --    Gui:setMinSize(5, 5)
    Gui:image(LTheoryRedux.logoname) -- draw the LTR name image
    Gui:setPercentSize(95, 55)
    Gui:endContainer()               -- end title panel

    -- Main Menu
    self:ShowMainMenuInner()

    Gui:endContainer() -- end title/menu panel

    Gui:spacer()

    -- Changelog
    Gui:beginVerticalContainer() -- begin changelog panel
    Gui:setPercentWidth(40)
    Gui:setPercentHeight(95)
    Gui:setAlignment(AlignHorizontal.Stretch, AlignVertical.Top)
    Gui:setChildrenHorizontalAlignment(AlignHorizontal.Stretch)

    Gui:beginVerticalContainer()
    Gui:setPercentHeight(20)
    Gui:endContainer()

    Gui:beginVerticalContainer() -- begin changelog text panel
    Gui:setPercentHeight(80)
    Gui:setSpacing(0)
    Gui:setBgColor(Color(0.1, 0.1, 0.1, 0.5))

    Gui:beginStackContainer() -- begin top text panel
    Gui:setPercentSize(100, 8)
    Gui:setHorizontalAlignment(AlignHorizontal.Center)
    Gui:setChildrenAlignment(AlignHorizontal.Center, AlignVertical.Center)
    Gui:setBgColor(Color(0.2, 0.2, 0.2, 0.3))
    Gui:textEx(Cache.Font('RajdhaniBold', 38), 'Notes for version ' .. Config.gameVersion, Color(1.0, 1.0, 1.0, 1.0))
    Gui:endContainer()        -- end top text panel
    Gui:beginStackContainer() -- begin middle text panel
    Gui:setPercentSize(100, 8)
    Gui:setHorizontalAlignment(AlignHorizontal.Center)
    Gui:setChildrenAlignment(AlignHorizontal.Center, AlignVertical.Center)
    Gui:textEx(Cache.Font('Rajdhani', 28), 'Changelog', Color(1.0, 1.0, 1.0, 1.0))
    Gui:endContainer()        -- end middle text panel
    Gui:beginStackContainer() -- begin details text panel
    Gui:setPercentSize(90, 84)
    Gui:setHorizontalAlignment(AlignHorizontal.Center)
    Gui:setChildrenHorizontalAlignment(AlignHorizontal.Left)
    Gui:textEx(Cache.Font('Rajdhani', 20), '- Lorem ipsum', Color(1.0, 1.0, 1.0, 1.0))
    Gui:endContainer() -- end details text panel

    Gui:endContainer() -- end changelog text panel

    Gui:spacer()

    Gui:endContainer() -- end changelog panel

    Gui:beginStackContainer()
    Gui:setPercentWidth(3)

    Gui:endContainer() -- end main menu screen

    Gui:endContainer() -- end game window
end

function MainMenu:ShowMainMenuInner()
    -- Add Main Menu items
    local scaleFactor = (LTheoryRedux.resX / 24) / 72

    Gui:beginVerticalContainer() -- begin menu/metrics panel
    Gui:setVerticalAlignment(AlignVertical.Stretch)
    Gui:setChildrenHorizontalAlignment(AlignHorizontal.Stretch)

    Gui:clearStyle()
    Gui:setPropertyColor(GuiProperties.TextColor, Color(0.9, 0.9, 0.9, 1.0))
    Gui:setPropertyFont(GuiProperties.TextFont, Cache.Font('RajdhaniSemiBold', 36 * scaleFactor))

    if Gui:button("NEW GAME") then
        self:ShowSeedDialog()
    end
    Gui:setVerticalAlignment(AlignVertical.Stretch) -- set individually on each button child to enforce full stretching

    if Gui:button("LOAD GAME") then
        self:ShowSeedDialog()
    end
    Gui:setVerticalAlignment(AlignVertical.Stretch)

    if Gui:button("SETTINGS") then
        self:ShowSettingsScreen()
    end
    Gui:setVerticalAlignment(AlignVertical.Stretch)

    if Gui:button("CREDITS") then
    end
    Gui:setVerticalAlignment(AlignVertical.Stretch)

    if Gui:button("BACKGROUND") then
        self:SetBackgroundMode(true)
    end
    Gui:setVerticalAlignment(AlignVertical.Stretch)

    if Gui:button("EXIT GAME") then
        LTheoryRedux:exitGame()
    end
    Gui:setVerticalAlignment(AlignVertical.Stretch)

    Gui:clearStyle()

    -- Show the game version (and, for now, current screen resolution)
    Gui:beginHorizontalContainer() -- begin metrics panel
    Gui:setChildrenVerticalAlignment(AlignVertical.Bottom)
    Gui:setMarginEx(5.0, 10.0, 5.0, 10.0)

    self:ShadowText(Config.gameVersion, 'RajdhaniSemiBold', 12 * scaleFactor, 2.0, 0.9, 0.9, 0.9, 1.0)
    Gui:setHorizontalAlignment(AlignHorizontal.Left)

    Gui:spacer()

    self:ShadowText('Resolution = ' .. LTheoryRedux.resX .. ' x ' .. LTheoryRedux.resY, 'RajdhaniSemiBold',
        12 * scaleFactor, 2.0, 0.9, 0.9, 0.9, 1.0)
    Gui:setHorizontalAlignment(AlignHorizontal.Right)

    Gui:endContainer() -- end metrics panel

    Gui:endContainer() -- end menu/metrics panel
end
