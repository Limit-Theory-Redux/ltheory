local Test = require('States.Application')

local rng = RNG.FromTime()

local useRenderer = true
local drawExitMenu = false

local overlayAlpha = 0.7

local station_options = 1
local station_options_market = 1
local station_options_config = 1
local station_options_occupy = 1

local player_account = 123456789

local listCommodities = {
    "Agricultural Products",
    "Alloys",
    "Animals, Livestock",
    "Animals, Rare",
    "Art, Audio Recordings",
    "Art, Paintings",
    "Art, Sculptures",
    "Chemicals",
    "Clothing",
    "Computer Parts",
    "Computers",
    "Construction Materials",
    "Consumables",
    "Crystals, Gems, Non-Precious",
    "Crystals, Gems, Precious",
    "Crystals, Raw",
    "Currency, Coinage",
    "Currency, Paper",
    "Drones",
    "Electronics",
    "Elements",
    "Energy Systems",
    "Fertilizers",
    "Foods, Animal",
    "Foods, Gourmet",
    "Foods, Plants",
    "Fuels",
    "Furniture",
    "Gases",
    "Hydrocarbons, Raw",
    "Liquids",
    "Medical Equipment",
    "Metals",
    "Minerals",
    "Nitrogen Compounds",
    "Ores, Metal, Ferrous",
    "Ores, Metal, Non-Ferrous",
    "Ores, Non-Metal",
    "Ores, Radioactive",
    "Petrochemicals",
    "Pharmaceuticals",
    "Plants",
    "Plants, Rare",
    "Plastics",
    "Polymers",
    "Robots",
    "Ship Engines",
    "Ship Parts",
    "Ship Shields",
    "Software Components",
    "Software",
    "Special Alloys",
    "Spices",
    "Textiles",
    "Unit Prefabs",
    "Utility",
}

function Test:onInit()
    self.renderer = Renderer()
end

function Test:onInput() end

function Test:showMarketOverlay()
    Gui:beginStackContainer() -- begin window panel
    Gui:setAlignment(AlignHorizontal.Stretch, AlignVertical.Stretch)
    Gui:setChildrenAlignment(AlignHorizontal.Stretch, AlignVertical.Stretch)

    -- TODO: Remove this temporary image when this overlay is integrated with the full game
    local bg = Tex2D.Load("./res/images/Background-for-testing.png")
    Gui:image(bg)

    Gui:beginStackContainer() -- begin overlay panel
    Gui:setPercentSize(97, 94)
    Gui:setAlignment(AlignHorizontal.Center, AlignVertical.Center)
    Gui:setBorder(12, 0.2, 0.2, 0.7, overlayAlpha)

    Gui:beginVerticalContainer() -- begin Station Services panels
    Gui:setAlignment(AlignHorizontal.Stretch, AlignVertical.Stretch)
    Gui:setSpacing(0)

    self:showMarketOverlayTitle()
    self:showMarketOverlayMenuBar()
    self:showMarketOverlayDivider()
    self:showMarketOverlayData()

    Gui:endContainer() -- end Station Services panels

    Gui:endContainer() -- end overlay panel

    -- TODO: Remove these temporary metrics when this overlay is integrated with the full game
    self:showMetrics()

    Gui:endContainer() -- end window panel
end

function Test:showMarketOverlayTitle()
    Gui:beginStackContainer()
    Gui:setHorizontalAlignment(AlignHorizontal.Stretch)
    Gui:setBgColor(0.25, 0.32, 0.35, 0.5)
    Gui:setPadding(0, 8)

    if station_options == 1 then
        Gui:textEx(Cache.Font("Iceland", 64), "MARKET", 1.0, 1.0, 1.0, 1.0)
    elseif station_options == 2 then
        Gui:textEx(Cache.Font("Iceland", 64), "SHIP CONFIG", 1.0, 1.0, 1.0, 1.0)
    elseif station_options == 3 then
        Gui:textEx(Cache.Font("Iceland", 64), "MANAGEMENT", 1.0, 1.0, 1.0, 1.0)
    end
    Gui:setAlignment(AlignHorizontal.Center, AlignVertical.Center)

    Gui:endContainer()
end

function Test:showMarketOverlayMenuBar()
    -- Divide the horizontal menu bar area into three sections: location, menu buttons, and money
    Gui:beginHorizontalContainer() -- begin horizontal menu bar area
    Gui:setHorizontalAlignment(AlignHorizontal.Stretch)
    Gui:setChildrenVerticalAlignment(AlignVertical.Stretch)
    Gui:setBgColor(0.25, 0.32, 0.35, 0.5)
    Gui:setMargin(0, 0)
    Gui:setPadding(0, 0)
    --    Gui:setBorder(0, 0.0, 0.0, 0.0, 0.0)

    -- Section 1 is the player ship's current location
    Gui:beginVerticalContainer() -- begin menu bar section 1
    Gui:setFixedWidth(300)
    Gui:setHorizontalAlignment(AlignHorizontal.Left)
    Gui:setChildrenAlignment(AlignHorizontal.Center, AlignVertical.Center)
    Gui:setPadding(15, 15)

    Gui:textEx(Cache.Font("Iceland", 32), "Docked at", 1.0, 1.0, 1.0, 1.0)
    Gui:textEx(Cache.Font("Iceland", 32), "Titan Station", 1.0, 1.0, 1.0, 1.0)
    Gui:textEx(Cache.Font("Iceland", 24), "Parnell System", 1.0, 1.0, 1.0, 1.0)

    Gui:endContainer() -- end menu bar section 1

    -- Section 2 contains the menu buttons (such as "Commodities") associated with this market function (such as "Market")
    Gui:beginHorizontalContainer() -- begin menu bar section 2
    Gui:setHorizontalAlignment(AlignHorizontal.Stretch)
    Gui:setChildrenAlignment(AlignHorizontal.Center, AlignVertical.Center)
    Gui:setPadding(15, 15)

    if station_options == 1 then -- Station services: Marketplace
        if station_options_market == 1 then
            Gui:pushFont(Cache.Font('RajdhaniBold', 24))
            Gui:pushTextColor(1.0, 0.0, 1.0, 1.0)
        else
            Gui:pushFont(Cache.Font('Rajdhani', 24))
            Gui:pushTextColor(1.0, 0.0, 1.0, 1.0)
        end
        if Gui:button("Commodities") then
            player_account = 123456789
            station_options_market = 1
        end
        Gui:setFixedSize(280, 60)
        Gui:setBgColor(0.0, 0.0, 0.0, 0.5)
        Gui:setBorder(2, 1.0, 0.0, 1.0, 1.0)
        Gui:popStyle(2)
        if station_options_market == 2 then
            Gui:pushFont(Cache.Font('RajdhaniBold', 24))
            Gui:pushTextColor(1.0, 0.0, 1.0, 1.0)
        else
            Gui:pushFont(Cache.Font('Rajdhani', 24))
            Gui:pushTextColor(1.0, 0.0, 1.0, 1.0)
        end
        if Gui:button("Data / Intel") then
            player_account = 123456
            station_options_market = 2
        end
        Gui:setFixedSize(280, 60)
        Gui:setBgColor(0.0, 0.0, 0.0, 0.5)
        Gui:setBorder(2, 1.0, 0.0, 1.0, 1.0)
        Gui:popStyle(2)
        if station_options_market == 3 then
            Gui:pushFont(Cache.Font('RajdhaniBold', 24))
            Gui:pushTextColor(1.0, 0.0, 1.0, 1.0)
        else
            Gui:pushFont(Cache.Font('Rajdhani', 24))
            Gui:pushTextColor(1.0, 0.0, 1.0, 1.0)
        end
        if Gui:button("Exchange Orders") then
            player_account = 123
            station_options_market = 3
        end
        Gui:setFixedSize(280, 60)
        Gui:setBgColor(0.0, 0.0, 0.0, 0.5)
        Gui:setBorder(2, 1.0, 0.0, 1.0, 1.0)
        Gui:popStyle(2)
        if station_options_market == 4 then
            Gui:pushFont(Cache.Font('RajdhaniBold', 24))
            Gui:pushTextColor(1.0, 0.0, 1.0, 1.0)
        else
            Gui:pushFont(Cache.Font('Rajdhani', 24))
            Gui:pushTextColor(1.0, 0.0, 1.0, 1.0)
        end
        if Gui:button("Blank") then
            player_account = 1
            station_options_market = 4
        end
        Gui:setFixedSize(280, 60)
        Gui:setBgColor(0.0, 0.0, 0.0, 0.5)
        Gui:setBorder(2, 1.0, 0.0, 1.0, 1.0)
        Gui:popStyle(2)
    elseif station_options == 2 then -- Station services: ship config
        if station_options_config == 1 then
            Gui:pushFont(Cache.Font('RajdhaniBold', 24))
            Gui:pushTextColor(0.9, 0.8, 0.2, 1.0)
        else
            Gui:pushFont(Cache.Font('Rajdhani', 24))
            Gui:pushTextColor(0.7, 0.6, 0.0, 1.0)
        end
        if Gui:button("SHIP OVERVIEW") then station_options_config = 1 end
        Gui:setFixedSize(280, 60)
        Gui:setBgColor(0.0, 0.0, 0.0, 0.5)
        Gui:setBorder(2, 0.7, 0.6, 0.0, 1.0)
        Gui:popStyle(2)
        if station_options_config == 2 then
            Gui:pushFont(Cache.Font('RajdhaniBold', 24))
            Gui:pushTextColor(0.9, 0.8, 0.2, 1.0)
        else
            Gui:pushFont(Cache.Font('Rajdhani', 24))
            Gui:pushTextColor(0.7, 0.6, 0.0, 1.0)
        end
        if Gui:button("LOADOUT") then station_options_config = 2 end
        Gui:setFixedSize(280, 60)
        Gui:setBgColor(0.0, 0.0, 0.0, 0.5)
        Gui:setBorder(2, 0.7, 0.6, 0.0, 1.0)
        Gui:popStyle(2)
        if station_options_config == 3 then
            Gui:pushFont(Cache.Font('RajdhaniBold', 24))
            Gui:pushTextColor(0.9, 0.8, 0.2, 1.0)
        else
            Gui:pushFont(Cache.Font('Rajdhani', 24))
            Gui:pushTextColor(0.7, 0.6, 0.0, 1.0)
        end
        if Gui:button("BUY/SELL EQUIPMENT") then station_options_config = 3 end
        Gui:setFixedSize(280, 60)
        Gui:setBgColor(0.0, 0.0, 0.0, 0.5)
        Gui:setBorder(2, 0.7, 0.6, 0.0, 1.0)
        Gui:popStyle(2)
        if station_options_config == 4 then
            Gui:pushFont(Cache.Font('RajdhaniBold', 24))
            Gui:pushTextColor(0.9, 0.8, 0.2, 1.0)
        else
            Gui:pushFont(Cache.Font('Rajdhani', 24))
            Gui:pushTextColor(0.7, 0.6, 0.0, 1.0)
        end
        if Gui:button("SHIPYARD") then station_options_config = 4 end
        Gui:setFixedSize(280, 60)
        Gui:setBgColor(0.0, 0.0, 0.0, 0.5)
        Gui:setBorder(2, 0.7, 0.6, 0.0, 1.0)
        Gui:popStyle(2)
    elseif station_options == 3 then -- Station services: Occupational
        if station_options_occupy == 1 then
            Gui:pushFont(Cache.Font('RajdhaniBold', 24))
            Gui:pushTextColor(0.9, 0.2, 0.2, 1.0)
        else
            Gui:pushFont(Cache.Font('Rajdhani', 24))
            Gui:pushTextColor(0.7, 0.0, 0.0, 1.0)
        end
        if Gui:button("FLEETS") then station_options_occupy = 1 end
        Gui:setFixedSize(280, 60)
        Gui:setBgColor(0.0, 0.0, 0.0, 0.5)
        Gui:setBorder(2, 0.7, 0.0, 0.0, 1.0)
        Gui:popStyle(2)
        if station_options_occupy == 2 then
            Gui:pushFont(Cache.Font('RajdhaniBold', 24))
            Gui:pushTextColor(0.9, 0.2, 0.2, 1.0)
        else
            Gui:pushFont(Cache.Font('Rajdhani', 24))
            Gui:pushTextColor(0.7, 0.0, 0.0, 1.0)
        end
        if Gui:button("PROJECTS") then station_options_occupy = 2 end
        Gui:setFixedSize(280, 60)
        Gui:setBgColor(0.0, 0.0, 0.0, 0.5)
        Gui:setBorder(2, 0.7, 0.0, 0.0, 1.0)
        Gui:popStyle(2)
        if station_options_occupy == 3 then
            Gui:pushFont(Cache.Font('RajdhaniBold', 24))
            Gui:pushTextColor(0.9, 0.2, 0.2, 1.0)
        else
            Gui:pushFont(Cache.Font('Rajdhani', 24))
            Gui:pushTextColor(0.7, 0.0, 0.0, 1.0)
        end
        if Gui:button("CONTRACTS") then station_options_occupy = 3 end
        Gui:setFixedSize(280, 60)
        Gui:setBgColor(0.0, 0.0, 0.0, 0.5)
        Gui:setBorder(2, 0.7, 0.0, 0.0, 1.0)
        Gui:popStyle(2)
        if station_options_occupy == 4 then
            Gui:pushFont(Cache.Font('RajdhaniBold', 24))
            Gui:pushTextColor(0.9, 0.2, 0.2, 1.0)
        else
            Gui:pushFont(Cache.Font('Rajdhani', 24))
            Gui:pushTextColor(0.7, 0.0, 0.0, 1.0)
        end
        if Gui:button("FACTIONS") then station_options_occupy = 4 end
        Gui:setFixedSize(280, 60)
        Gui:setBgColor(0.0, 0.0, 0.0, 0.5)
        Gui:setBorder(2, 0.7, 0.0, 0.0, 1.0)
        Gui:popStyle(2)
    end

    Gui:endContainer() -- end menu bar section 2

    -- Section 3 contains the player's current financial balance
    Gui:beginStackContainer() -- begin menu bar section 3
    Gui:setFixedWidth(200)
    Gui:setHorizontalAlignment(AlignHorizontal.Right)
    Gui:setChildrenAlignment(AlignHorizontal.Right, AlignVertical.Center)
    Gui:setPadding(15, 15)

    Gui:textEx(Cache.Font("Iceland", 24), "$" .. player_account, 1.0, 1.0, 1.0, 1.0)

    Gui:endContainer() -- end menu bar section 3

    Gui:spacer()

    Gui:endContainer() -- end horizontal menu bar area
end

function Test:showMarketOverlayDivider()
    Gui:beginStackContainer()
    Gui:setHorizontalAlignment(AlignHorizontal.Stretch)
    Gui:setBgColor(0.25, 0.32, 0.35, 0.5)
    Gui:setMargin(0, 0)
    Gui:setPadding(0, 0)
    Gui:setBorder(0, 1.0, 1.0, 1.0, 1.0)

    Gui:horizontalDivider(Color(6, 0.8, 0.8, 0.8, 0.5))
    Gui:setHorizontalAlignment(AlignHorizontal.Stretch)

    Gui:endContainer()
end

function Test:showMarketOverlayData()
    -- This is where the bulk of the UI code will go to draw the many various Marketplace elements

    -- Must wrap child data panels inside this StackContainer because its parent uses setPercentSize() rather than .Stretch
    Gui:beginStackContainer()                                        -- begin child panel
    Gui:setAlignment(AlignHorizontal.Stretch, AlignVertical.Stretch) -- this child panel should stretch to parent, but doesn't
    Gui:setBgColor(0.6, 0.0, 0.0, 0.4)
    --    Gui:setBorder(2, 1.0, 0.0, 0.0, 1.0)

    Gui:beginHorizontalContainer() -- begin market data area
    Gui:setAlignment(AlignHorizontal.Stretch, AlignVertical.Stretch)
    Gui:setBgColor(0.1, 0.1, 0.1, 0.8)

    Gui:beginVerticalContainer() -- begin market category buttons in left-hand panel
    Gui:setVerticalAlignment(AlignVertical.Stretch)
    Gui:setMargin(8, 10)

    Gui:beginVerticalContainer() -- begin market category buttons top panel
    Gui:setFixedWidth(300)
    Gui:setVerticalAlignment(AlignVertical.Stretch)
    Gui:setChildrenAlignment(AlignHorizontal.Stretch, AlignVertical.Center)

    Gui:pushFont(Cache.Font('RajdhaniBold', 32))
    if station_options == 1 then
        Gui:pushTextColor(1.0, 0.3, 0.9, 1.0)
    else
        Gui:pushTextColor(0.5, 0.0, 0.4, 1.0)
    end
    if Gui:button("MARKET") then station_options = 1 end
    Gui:setPercentHeight(25)
    Gui:popStyle(2)
    Gui:pushFont(Cache.Font('RajdhaniBold', 32))
    if station_options == 2 then
        Gui:pushTextColor(0.9, 0.8, 0.2, 1.0)
    else
        Gui:pushTextColor(0.5, 0.4, 0.0, 1.0)
    end
    if Gui:button("SHIP CONFIG") then station_options = 2 end
    Gui:setPercentHeight(25)
    Gui:popStyle(2)
    Gui:pushFont(Cache.Font('RajdhaniBold', 32))
    if station_options == 3 then
        Gui:pushTextColor(0.9, 0.2, 0.2, 1.0)
    else
        Gui:pushTextColor(0.5, 0.0, 0.0, 1.0)
    end
    if Gui:button("MANAGEMENT") then station_options = 3 end
    Gui:setPercentHeight(25)
    Gui:popStyle(2)
    Gui:endContainer()        -- end market category top panel

    Gui:beginStackContainer() -- begin market category buttons bottom panel
    Gui:setPercentHeight(25)
    Gui:setHorizontalAlignment(AlignHorizontal.Stretch)
    Gui:setChildrenAlignment(AlignHorizontal.Stretch, AlignVertical.Stretch)

    Gui:pushTextColor(0.2, 1.0, 0.3, 1.0)
    Gui:pushFont(Cache.Font('RajdhaniBold', 36))
    if Gui:button("RETURN") then
        Test:quit()
    end
    Gui:popStyle(2)
    Gui:endContainer() -- end market category bottom panel

    Gui:endContainer() -- end market category buttons panel

    Gui:beginStackContainer()
    Gui:setVerticalAlignment(AlignVertical.Stretch)

    Gui:verticalDivider(Color(2, 0.8, 0.8, 0.8, 0.5))
    Gui:setAlignment(AlignHorizontal.Left, AlignVertical.Stretch)

    Gui:endContainer()

    Gui:beginVerticalContainer() -- begin market category details panel
    Gui:setAlignment(AlignHorizontal.Stretch, AlignVertical.Stretch)
    Gui:setPadding(10, 10)

    if station_options == 1 then
        if station_options_market == 1 then
            Gui:textEx(Cache.Font("Iceland", 24), "COMMODITY EXCHANGE", 1.0, 1.0, 1.0, 1.0)
        elseif station_options_market == 2 then
            Gui:textEx(Cache.Font("Iceland", 24), "DATA / INTEL", 1.0, 1.0, 1.0, 1.0)
        elseif station_options_market == 3 then
            Gui:textEx(Cache.Font("Iceland", 24), "EXCHANGE ORDERS", 1.0, 1.0, 1.0, 1.0)
        else
            Gui:textEx(Cache.Font("Iceland", 24), "BLANK", 1.0, 1.0, 1.0, 1.0)
        end
    else
        Gui:textEx(Cache.Font("Iceland", 24), " ", 1.0, 1.0, 1.0, 1.0)
    end
    Gui:setAlignment(AlignHorizontal.Left, AlignVertical.Center)
    Gui:setPercentHeight(2)
    Gui:setMargin(10, 8)

    if station_options == 1 then
        if station_options_market == 1 then
            Gui:beginHorizontalContainer() -- begin Commodities subpanel
            Gui:setAlignment(AlignHorizontal.Stretch, AlignVertical.Stretch)
            Gui:setChildrenAlignment(AlignHorizontal.Stretch, AlignVertical.Stretch)

            -- Commodities Search subpanel
            Gui:beginVerticalContainer() -- begin Commodities Search subpanel
            Gui:setPercentWidth(24)
            Gui:setBorder(2, 1.0, 1.0, 1.0, 1.0)

            Gui:beginHorizontalContainer() -- begin Commodities search dropdown subpanel
            Gui:setHorizontalAlignment(AlignHorizontal.Stretch)
            Gui:setChildrenVerticalAlignment(AlignVertical.Bottom)
            Gui:setMargin(10, 6)
            Gui:textEx(Cache.Font("Exo2", 20), "Search:", 1.0, 1.0, 1.0, 1.0)
            Gui:setHorizontalAlignment(AlignHorizontal.Left)
            Gui:spacer()
            Gui:textEx(Cache.Font("Exo2", 20), "[dropdown menu]", 1.0, 1.0, 1.0, 1.0)
            Gui:setHorizontalAlignment(AlignHorizontal.Right)
            Gui:endContainer() -- end Commodities search dropdown subpanel

            Gui:horizontalDivider(Color(2, 0.8, 0.8, 0.8, 0.5))

            Gui:beginStackContainer() -- begin Commodities search text subpanel
            Gui:setHorizontalAlignment(AlignHorizontal.Stretch)
            Gui:setBgColor(0.3, 0.3, 0.3, 0.8)
            Gui:setPadding(4, 6)
            Gui:setMargin(10, 6)
            Gui:textEx(Cache.Font("Exo2", 20), "[textbox \"Search Commodity\"]", 1.0, 1.0, 1.0, 1.0)
            Gui:setHorizontalAlignment(AlignHorizontal.Left)
            Gui:endContainer() -- end Commodities search text subpanel

            Gui:horizontalDivider(Color(2, 0.8, 0.8, 0.8, 0.5))

            Gui:beginVerticalContainer() -- begin Commodities search list subpanel
            Gui:setMargin(10, 6)
            --            Gui:beginWindow("Commodities List")
            --            Gui:beginScroll(512)
            Gui:textEx(Cache.Font("Exo2", 14), listCommodities[1], 1.0, 1.0, 1.0, 1.0)
            --            for i = 1, #listCommodities do
            --                Gui:textEx(Cache.Font("Exo2", 14), listCommodities[i], 1.0, 1.0, 1.0, 1.0)
            --            end
            --            Gui:endScroll(Input)
            --            Gui:endWindow()
            Gui:endContainer() -- end Commodities search list subpanel

            Gui:endContainer() -- end Commodities Search subpanel

            -- Commodities Construction subpanel
            Gui:beginVerticalContainer() -- begin Commodities Construction subpanel
            Gui:setPercentWidth(25)

            Gui:beginHorizontalContainer() -- begin Commodities Construction controls subpanel
            Gui:setPercentHeight(5)
            Gui:setHorizontalAlignment(AlignHorizontal.Stretch)
            Gui:setPadding(8, 8)

            Gui:textEx(Cache.Font("Exo2", 18), "Construction Materials", 1.0, 1.0, 1.0, 1.0)
            Gui:setAlignment(AlignHorizontal.Left, AlignVertical.Center)
            Gui:spacer()
            Gui:pushFont(Cache.Font("Exo2Bold", 18))
            Gui:pushTextColor(1.0, 1.0, 1.0, 1.0)
            Gui:button("Price Chart")
            Gui:popStyle(2)
            Gui:setAlignment(AlignHorizontal.Right, AlignVertical.Center)

            Gui:endContainer()           -- end Commodities Construction controls subpanel

            Gui:beginVerticalContainer() -- begin Commodities Construction listing subpanel
            Gui:setAlignment(AlignHorizontal.Stretch, AlignVertical.Stretch)
            Gui:setChildrenAlignment(AlignHorizontal.Stretch, AlignVertical.Stretch)
            Gui:setBorder(2, 1.0, 1.0, 1.0, 1.0)

            local materialSelected = "STL.Titan" -- temp

            Gui:endContainer()                   -- end Commodities Construction listing subpanel

            Gui:endContainer()                   -- end Commodities Construction subpanel

            -- Commodities Order Book subpanel
            Gui:beginVerticalContainer() -- begin Commodities Order Book subpanel
            Gui:setPercentWidth(25)
            Gui:setPadding(12, 0)

            Gui:beginHorizontalContainer() -- begin Commodities Construction controls subpanel
            Gui:setPercentHeight(0.5)
            Gui:endContainer()             -- end Commodities Construction controls subpanel

            Gui:textEx(Cache.Font("Exo2", 18), "Order Book (" .. materialSelected .. ")", 1.0, 1.0, 1.0, 1.0)

            Gui:beginStackContainer() -- begin Commodities Order Book Bids subpanel
            Gui:setPercentHeight(42)
            Gui:setHorizontalAlignment(AlignHorizontal.Stretch)
            Gui:setBorder(2, 1.0, 1.0, 0.0, 1.0)
            Gui:endContainer() -- end Commodities Order Book Bids subpanel

            Gui:spacer()

            Gui:beginStackContainer() -- begin Commodities Order Book Asks subpanel
            Gui:setPercentHeight(45)
            Gui:setHorizontalAlignment(AlignHorizontal.Stretch)
            Gui:setBorder(2, 1.0, 0.0, 1.0, 1.0)
            Gui:endContainer() -- end Commodities Order Book Asks subpanel

            Gui:endContainer() -- end Commodities Order Book subpanel

            -- Commodities Place Order subpanel
            Gui:beginVerticalContainer() -- begin Commodities Place Order subpanel
            Gui:setPercentWidth(24)
            Gui:setBorder(2, 0.0, 1.0, 0.0, 1.0)



            Gui:endContainer() -- end Commodities Place Order subpanel


            Gui:endContainer() -- end Commodities subpanel
        end
    end

    Gui:endContainer() -- end market category details panel

    Gui:endContainer() -- end market data area

    Gui:endContainer() -- end child panel
end

--** Begin game control menu functions **--

function Test:showMetrics()
    Gui:textEx(Cache.Font("Exo2", 12), format("FPS: %.2f", 1.0 / self.dt), 1.0, 1.0, 1.0, 1.0)
    Gui:setAlignment(AlignHorizontal.Left, AlignVertical.Top)

    Gui:textEx(Cache.Font("Exo2", 12), self.resX .. " x " .. self.resY, 1.0, 1.0, 1.0, 1.0)
    Gui:setAlignment(AlignHorizontal.Right, AlignVertical.Top)
end

function Test:showCtrlDialogInner()
    Gui:beginHorizontalContainer()
    Gui:setBorderWidth(2)
    Gui:pushTextColor(1.0, 1.0, 1.0, 1.0)
    Gui:pushFont(Cache.Font('Exo2Bold', 18))
    if Gui:button("Cancel") then
        drawExitMenu = false
    end
    Gui:setSpacing(24)
    if Gui:button("Quit") then
        Test:quit()
    end
    Gui:popStyle(2)
    Gui:endContainer()
end

function Test:showCtrlDialog()
    Gui:beginStackContainer()
    Gui:setAlignment(AlignHorizontal.Center, AlignVertical.Center)
    Gui:beginWindow("Game Control")
    Gui:setBorderWidth(2)
    Gui:textEx(Cache.Font("Iceland", 20), "Game Control", 0.6, 0.7, 1.0, 1.0)
    Gui:setAlignment(AlignHorizontal.Center, AlignVertical.Center)
    Gui:setSpacing(24)
    self:showCtrlDialogInner()
    Gui:endWindow()
    Gui:endContainer()
end

function Test:onUpdate(dt)
    Profiler.Begin('Gui:update')
    if Input:isPressed(Button.KeyboardEscape) then
        drawExitMenu = true
    end
    Gui:beginGui(self.resX, self.resY)
    if drawExitMenu then
        self:showCtrlDialog()
    else
        self:showMarketOverlay()
    end
    Gui:endGui()
    Profiler.End()
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
