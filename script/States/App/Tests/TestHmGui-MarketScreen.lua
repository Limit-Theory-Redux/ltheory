local Test = require('States.Application')

local rng = RNG.FromTime()

local useRenderer = true
local drawExitMenu = false

local station_options = 1
local station_options_market = 1
local station_options_config = 1
local station_options_occupy = 1

function Test:onInit()
    self.renderer = Renderer()
end

function Test:onInput() end

local code = [[
static void MemPool_Grow (MemPool* self) {
  uint16 newBlockIndex = self->blockCount++;
  self->capacity += self->blockSize;

  /* Grow the list of pool blocks. */
  self->blocks = (void**)MemRealloc(self->blocks, self->blockCount * sizeof(void*));

  /* Allocate a new block and place at the back of the list. */
  void* newBlock = MemAlloc(self->cellSize * self->blockSize);
  self->blocks[newBlockIndex] = newBlock;

  /* Wire up the free list for this block. Note that we can assume the existing
   * free list is empty if the pool is requesting to grow, hence we overwrite
   * the existing list head. The block's initial freelist is wired sequentially
   * ( 0 -> 1 -> 2 ) for optimal cache locality. */
  void** prev = &self->freeList;
  char* pCurr = (char*)newBlock;
  for (uint32 i = 0; i < self->blockSize; ++i) {
    *prev = (void*)pCurr;
    prev = (void**)pCurr;
    pCurr += self->cellSize;
  }
  *prev = 0;
}
]]

function Test:showMenuDrop()
    Gui:beginWindow("Screen Title")
    if station_options == 1 then
        Gui:textEx(Cache.Font('Iceland', 64), 'MARKETPLACE', 1.0, 1.0, 1.0, 1.0)
    elseif station_options == 2 then
        Gui:textEx(Cache.Font('Iceland', 64), 'SHIP CONFIGURATION', 1.0, 1.0, 1.0, 1.0)
    elseif station_options == 3 then
        Gui:textEx(Cache.Font('Iceland', 64), 'OCCUPATIONAL', 1.0, 1.0, 1.0, 1.0)
    else
        Gui:textEx(Cache.Font('Iceland', 64), 'WTF?', 1.0, 1.0, 1.0, 1.0)
    end
    Gui:setAlign(0.5, 0.0)
    self:showMenuDropInner()
    Gui:endWindow()
    Gui:setStretch(0.97, 0.95)
    Gui:setAlign(0.5, 0.5)
end

function Test:showMenuDropInner()
    Gui:beginVerticalContainer()
    Gui:beginHorizontalContainer() -- upper group: 1) location text, 2) station services type buttons, 3) cash on hand text
    Gui:beginVerticalContainer()
    Gui:textEx(Cache.Font('Iceland', 32), 'Docked at', 1.0, 1.0, 1.0, 1.0)
    Gui:setAlign(0.5, 0.2)
    Gui:textEx(Cache.Font('Iceland', 32), 'Titan Station', 1.0, 1.0, 1.0, 1.0)
    Gui:setAlign(0.5, 0.2)
    Gui:textEx(Cache.Font('Iceland', 24), 'Parnell System', 1.0, 1.0, 1.0, 1.0)
    Gui:setAlign(0.5, 0.2)
    Gui:endContainer()
    Gui:setStretch(0.7, 0.0)
    Gui:beginHorizontalContainer()
    if station_options == 1 then -- Station services: Marketplace
        if station_options_market == 1 then
            Gui:pushFont(Cache.Font('RajdhaniBold', 24))
            Gui:pushTextColor(0.9, 0.2, 0.8, 1.0)
        else
            Gui:pushFont(Cache.Font('Rajdhani', 24))
            Gui:pushTextColor(0.7, 0.0, 0.6, 1.0)
        end
        if Gui:button("COMMODITIES") then station_options_market = 1 end
        Gui:popStyle(2)
        if station_options_market == 2 then
            Gui:pushFont(Cache.Font('RajdhaniBold', 24))
            Gui:pushTextColor(0.9, 0.2, 0.8, 1.0)
        else
            Gui:pushFont(Cache.Font('Rajdhani', 24))
            Gui:pushTextColor(0.7, 0.0, 0.6, 1.0)
        end
        if Gui:button("DATA / INTEL") then station_options_market = 2 end
        Gui:popStyle(2)
        if station_options_market == 3 then
            Gui:pushFont(Cache.Font('RajdhaniBold', 24))
            Gui:pushTextColor(0.9, 0.2, 0.8, 1.0)
        else
            Gui:pushFont(Cache.Font('Rajdhani', 24))
            Gui:pushTextColor(0.7, 0.0, 0.6, 1.0)
        end
        if Gui:button("EXCHANGE ORDERS") then station_options_market = 3 end
        Gui:popStyle(2)
        if station_options_market == 4 then
            Gui:pushFont(Cache.Font('RajdhaniBold', 24))
            Gui:pushTextColor(0.9, 0.2, 0.8, 1.0)
        else
            Gui:pushFont(Cache.Font('Rajdhani', 24))
            Gui:pushTextColor(0.7, 0.0, 0.6, 1.0)
        end
        if Gui:button("BLANK") then station_options_market = 4 end
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
        Gui:popStyle(2)
        if station_options_config == 2 then
            Gui:pushFont(Cache.Font('RajdhaniBold', 24))
            Gui:pushTextColor(0.9, 0.8, 0.2, 1.0)
        else
            Gui:pushFont(Cache.Font('Rajdhani', 24))
            Gui:pushTextColor(0.7, 0.6, 0.0, 1.0)
        end
        if Gui:button("LOADOUT") then station_options_config = 2 end
        Gui:popStyle(2)
        if station_options_config == 3 then
            Gui:pushFont(Cache.Font('RajdhaniBold', 24))
            Gui:pushTextColor(0.9, 0.8, 0.2, 1.0)
        else
            Gui:pushFont(Cache.Font('Rajdhani', 24))
            Gui:pushTextColor(0.7, 0.6, 0.0, 1.0)
        end
        if Gui:button("BUY/SELL EQUIPMENT") then station_options_config = 3 end
        Gui:popStyle(2)
        if station_options_config == 4 then
            Gui:pushFont(Cache.Font('RajdhaniBold', 24))
            Gui:pushTextColor(0.9, 0.8, 0.2, 1.0)
        else
            Gui:pushFont(Cache.Font('Rajdhani', 24))
            Gui:pushTextColor(0.7, 0.6, 0.0, 1.0)
        end
        if Gui:button("SHIPYARD") then station_options_config = 4 end
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
        Gui:popStyle(2)
        if station_options_occupy == 2 then
            Gui:pushFont(Cache.Font('RajdhaniBold', 24))
            Gui:pushTextColor(0.9, 0.2, 0.2, 1.0)
        else
            Gui:pushFont(Cache.Font('Rajdhani', 24))
            Gui:pushTextColor(0.7, 0.0, 0.0, 1.0)
        end
        if Gui:button("PROJECTS") then station_options_occupy = 2 end
        Gui:popStyle(2)
        if station_options_occupy == 3 then
            Gui:pushFont(Cache.Font('RajdhaniBold', 24))
            Gui:pushTextColor(0.9, 0.2, 0.2, 1.0)
        else
            Gui:pushFont(Cache.Font('Rajdhani', 24))
            Gui:pushTextColor(0.7, 0.0, 0.0, 1.0)
        end
        if Gui:button("CONTRACTS") then station_options_occupy = 3 end
        Gui:popStyle(2)
        if station_options_occupy == 4 then
            Gui:pushFont(Cache.Font('RajdhaniBold', 24))
            Gui:pushTextColor(0.9, 0.2, 0.2, 1.0)
        else
            Gui:pushFont(Cache.Font('Rajdhani', 24))
            Gui:pushTextColor(0.7, 0.0, 0.0, 1.0)
        end
        if Gui:button("FACTIONS") then station_options_occupy = 4 end
        Gui:popStyle(2)
    end
    Gui:endContainer()
    Gui:setStretch(1.0, 1.0)
    Gui:textEx(Cache.Font('Iceland', 24), '$300,561,000', 1.0, 1.0, 1.0, 1.0)
    Gui:setAlign(2.0, 0.5)
    Gui:setStretch(0.1, 0.0)
    Gui:endContainer()
    Gui:setStretch(1.0, 0.1)

    -- divider (ImGui.Divider is a thing that exists. Gui:divider does not. So this is a kludge.
    Gui:setSpacing(16)
    Gui:rect(100.0, 6.0, 0.3, 0.3, 0.3, 1.0);
    Gui:setStretch(1.0, 0.0)
    Gui:setSpacing(16)

    Gui:beginHorizontalContainer() -- lower group, 2 windows: 1) Station buttons, 2) selected top button details
    Gui:beginHorizontalContainer() -- Station buttons
    Gui:setAlign(0.0, 0.0)
    Gui:setStretch(0.1, 1.0)
    Gui:beginVerticalContainer()
    Gui:beginVerticalContainer()
    Gui:beginVerticalContainer()
    Gui:pushFont(Cache.Font('RajdhaniBold', 32))
    if station_options == 1 then
        Gui:pushTextColor(1.0, 0.3, 0.9, 1.0)
    else
        Gui:pushTextColor(0.5, 0.0, 0.4, 1.0)
    end
    if Gui:button("MARKET") then station_options = 1 end
    Gui:popStyle(2)
    Gui:setStretch(1.0, 1.8)
    Gui:pushFont(Cache.Font('RajdhaniBold', 32))
    if station_options == 2 then
        Gui:pushTextColor(0.9, 0.8, 0.2, 1.0)
    else
        Gui:pushTextColor(0.5, 0.4, 0.0, 1.0)
    end
    if Gui:button("SHIP CONFIGURATION") then station_options = 2 end
    Gui:popStyle(2)
    Gui:setStretch(1.0, 1.8)
    Gui:pushFont(Cache.Font('RajdhaniBold', 32))
    if station_options == 3 then
        Gui:pushTextColor(0.9, 0.2, 0.2, 1.0)
    else
        Gui:pushTextColor(0.5, 0.0, 0.0, 1.0)
    end
    if Gui:button("OCCUPATION INFO") then station_options = 3 end
    Gui:popStyle(2)
    Gui:setStretch(1.0, 1.8)
    Gui:endContainer()
    Gui:setAlign(0.5, 0.4)
    Gui:setStretch(1.0, 0.7)
    Gui:setSpacing(50)
    Gui:beginVerticalContainer()
    Gui:pushTextColor(0.2, 1.0, 0.3, 1.0)
    Gui:pushFont(Cache.Font('RajdhaniBold', 36))
    Gui:button("LAUNCH")
    Gui:popStyle(2)
    Gui:setAlign(0.5, 0.0)
    Gui:setStretch(0.5, 0.0)
    Gui:endContainer()
    Gui:setAlign(0.5, 3.0)
    Gui:setStretch(1.0, 0.6)
    Gui:endContainer()
    Gui:setAlign(0.5, 1.0)
    Gui:setStretch(2.0, 1.0)
    Gui:endContainer()
    Gui:setStretch(0.1, 1.0)
    Gui:endContainer()

    Gui:beginVerticalContainer() -- selected station service details: name and info
    if station_options_market == 1 then
        Gui:textEx(Cache.Font('Iceland', 24), 'COMMODITY EXCHANGE', 1.0, 1.0, 1.0, 1.0)
    elseif station_options_market == 2 then
        Gui:textEx(Cache.Font('Iceland', 24), 'DATA/INTEL', 1.0, 1.0, 1.0, 1.0)
    elseif station_options_market == 3 then
        Gui:textEx(Cache.Font('Iceland', 24), 'EXCHANGE ORDERS', 1.0, 1.0, 1.0, 1.0)
    else
        Gui:textEx(Cache.Font('Iceland', 24), 'BLANK', 1.0, 1.0, 1.0, 1.0)
    end
    Gui:setAlign(0.0, 0.5)
    Gui:setStretch(1.0, 0.0)

    Gui:setSpacing(16)

    Gui:beginHorizontalContainer()      -- selected station service details
    if station_options_market == 1 then -- Commodities Screen
        Gui:beginHorizontalContainer()
        Gui:beginStackContainer()       -- Commodity Panel
        -- Create a nice border
        Gui:rect(1.0, 1.0, 0.7, 0.7, 0.7, 1.0);
        Gui:setStretch(1.0, 1.0)
        Gui:rect(1.0, 1.0, 0.1, 0.1, 0.1, 1.0);
        Gui:setStretch(0.992, 0.997)
        Gui:setAlign(0.5, 0.5)

        Gui:beginVerticalContainer()   -- Commodity Window Panel
        Gui:beginHorizontalContainer() -- Commodity Window Panel; search filter
        Gui:textEx(Cache.Font('Exo2', 20), 'Search:', 1.0, 1.0, 1.0, 1.0)
        Gui:setStretch(0.5, 0.0)
        Gui:setAlign(0.5, 0.6)

        Gui:beginStackContainer()
        Gui:rect(1.0, 1.0, 0.2, 0.2, 0.2, 1.0);
        Gui:setStretch(1.0, 0.6)
        Gui:setAlign(0.5, 0.5)
        Gui:textEx(Cache.Font('Exo2', 16), 'Current Station', 1.0, 1.0, 1.0, 1.0)
        Gui:setStretch(0.7, 0.0)
        Gui:setAlign(0.2, 0.5)
        Gui:endContainer()
        Gui:setStretch(1.2, 0.8)
        Gui:setAlign(0.0, 1.0)

        Gui:pushFont(Cache.Font('Exo2Bold', 16))
        Gui:button("V")
        Gui:popStyle(1)
        Gui:setStretch(0.0, 0.0)
        Gui:setAlign(1.0, 1.0)
        Gui:endContainer()
        Gui:setStretch(0.965, 0.2)
        Gui:setAlign(0.5, 0.5)

        Gui:rect(1.0, 1.0, 0.7, 0.7, 0.7, 1.0);
        Gui:setStretch(0.995, 0.05)
        Gui:setAlign(0.5, 1.0)

        Gui:beginHorizontalContainer() -- Commodity Window Panel; commodity search textbox
        Gui:beginStackContainer()
        Gui:rect(1.0, 1.0, 0.2, 0.2, 0.2, 1.0);
        Gui:setStretch(0.98, 1.0)
        Gui:setAlign(0.5, 0.1)
        Gui:textEx(Cache.Font('Exo2', 16), 'Search Commodity', 1.0, 1.0, 1.0, 1.0)
        Gui:setStretch(0.7, 1.0)
        Gui:setAlign(0.2, 1.0)
        Gui:endContainer()
        Gui:endContainer()
        Gui:setStretch(0.965, 0.3)
        Gui:setAlign(0.5, 0.0)

        Gui:rect(1.0, 1.0, 0.7, 0.7, 0.7, 1.0);
        Gui:setStretch(0.995, 0.05)
        Gui:setAlign(0.5, 0.0)

        Gui:beginStackContainer()
        Gui:rect(1.0, 1.0, 0.15, 0.15, 0.15, 1.0);

        Gui:beginStackContainer() -- Commodity List Panel
        -- The BeginScroll() calculation is a hack to keep the length of the scrollable area
        -- inside the bound of its enclosing GroupStack. It glitches when the full game window
        -- is vertically resized too far (but not all the way to the full vertical extent).
        Gui:beginScroll(634 + (self.resY - 900))
        Gui:textEx(Cache.Font('Exo2', 14), 'Agricultural Products', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Alloys', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Animals, Livestock', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Animals, Rare', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Art, Audio Recordings', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Art, Paintings', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Art, Sculptures', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Chemicals', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Clothing', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Computer Parts', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Computers', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Construction Materials', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Consumables', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Crystals, Gems, Non-Precious', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Crystals, Gems, Precious', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Crystals, Raw', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Currency, Coinage', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Currency, Paper', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Drones', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Electronics', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Elements', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Energy Systems', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Fertilizers', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Foods, Animal', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Foods, Gourmet', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Foods, Plants', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Fuels', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Furniture', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Gases', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Hydrocarbons, Raw', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Liquids', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Medical Equipment', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Metals', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Minerals', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Nitrogen Compounds', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Ores, Metal, Ferrous', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Ores, Metal, Non-Ferrous', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Ores, Non-Metal', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Ores, Radioactive', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Petrochemicals', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Pharmaceuticals', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Plants', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Plants, Rare', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Plastics', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Polymers', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Robots', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Ship Engines', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Ship Parts', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Ship Shields', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Software Components', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Software', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Special Alloys', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Spices', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Textiles', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Unit Prefabs', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Utility', 1.0, 1.0, 1.0, 1.0)
        Gui:endScroll(InputInstance)
        Gui:endContainer()
        Gui:endContainer()
        Gui:endContainer() -- end Commodity Window Panel
        Gui:setStretch(1.0, 1.0)
        Gui:endContainer() -- end Commodity Panel
        Gui:setStretch(1.0, 1.0)

        Gui:beginVerticalContainer() -- Construction Materials Table Panel
        Gui:beginHorizontalContainer()
        Gui:textEx(Cache.Font('Exo2', 18), 'Construction Materials', 1.0, 1.0, 1.0, 1.0)
        Gui:setStretch(0.5, 0.0)
        Gui:setAlign(0.2, 0.5)
        Gui:pushFont(Cache.Font('Exo2Bold', 16))
        Gui:button("Price Chart")
        Gui:setStretch(0.0, 0.0)
        Gui:setAlign(1.0, 0.5)
        Gui:popStyle(1)
        Gui:endContainer()
        Gui:setStretch(0.965, 0.0)
        Gui:setAlign(0.5, 0.0)

        Gui:beginStackContainer() -- Construction Materials Panel
        -- Create a nice border with internal borders
        Gui:rect(1.0, 1.0, 0.7, 0.7, 0.7, 1.0);
        Gui:setStretch(1.0, 1.0)
        Gui:rect(1.0, 1.0, 0.1, 0.1, 0.1, 1.0);
        Gui:setStretch(0.992, 0.997)
        Gui:setAlign(0.5, 0.5)

        Gui:rect(1.0, 1.0, 0.7, 0.7, 0.7, 1.0);
        Gui:setStretch(0.0025, 0.997)
        Gui:setAlign(0.31, 0.05)

        Gui:rect(1.0, 1.0, 0.7, 0.7, 0.7, 1.0);
        Gui:setStretch(0.0025, 0.997)
        Gui:setAlign(0.53, 0.05)

        Gui:rect(1.0, 1.0, 0.7, 0.7, 0.7, 1.0);
        Gui:setStretch(0.0025, 0.997)
        Gui:setAlign(0.74, 0.05)

        Gui:rect(1.0, 1.0, 0.7, 0.7, 0.7, 1.0);
        Gui:setStretch(0.995, 0.002)
        Gui:setAlign(0.5, 0.054)

        Gui:beginVerticalContainer()
        Gui:beginHorizontalContainer()
        Gui:beginVerticalContainer()
        Gui:textEx(Cache.Font('Exo2', 18), 'Commodity', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Ticker', 1.0, 1.0, 1.0, 1.0)
        Gui:endContainer()
        Gui:beginVerticalContainer()
        Gui:textEx(Cache.Font('Exo2', 18), 'Ask', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Amount', 1.0, 1.0, 1.0, 1.0)
        Gui:endContainer()
        Gui:beginVerticalContainer()
        Gui:textEx(Cache.Font('Exo2', 18), 'Bid', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Amount', 1.0, 1.0, 1.0, 1.0)
        Gui:endContainer()
        Gui:beginVerticalContainer()
        Gui:textEx(Cache.Font('Exo2', 14), 'Supply', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Demand', 1.0, 1.0, 1.0, 1.0)
        Gui:endContainer()
        Gui:endContainer()
        Gui:setStretch(0.965, 0.0)
        Gui:setAlign(0.5, 1.0)

        Gui:beginStackContainer() -- Commodity List Panel
        -- The BeginScroll() calculation is a hack to keep the length of the scrollable area
        -- inside the bound of its enclosing GroupStack. It glitches when the full game window
        -- is vertically resized too far (but not all the way to the full vertical extent).
        Gui:beginScroll(500 + (self.resY - 900))
        Gui:beginHorizontalContainer()
        Gui:beginVerticalContainer()
        Gui:textEx(Cache.Font('Exo2', 14), 'Nano Fiber', 1.0, 1.0, 1.0, 1.0)
        Gui:setStretch(0.0, 0.0)
        Gui:setAlign(0.0, 0.5)
        Gui:textEx(Cache.Font('Exo2', 12), 'NF.Titan', 1.0, 1.0, 1.0, 1.0)
        Gui:setStretch(0.0, 0.0)
        Gui:setAlign(0.0, 0.5)
        Gui:endContainer()
        Gui:setStretch(1.0, 0.0)
        Gui:setAlign(0.0, 0.5)
        Gui:beginVerticalContainer()
        Gui:textEx(Cache.Font('Exo2', 14), '9.99', 1.0, 1.0, 1.0, 1.0)
        Gui:setStretch(0.0, 0.0)
        Gui:setAlign(1.0, 0.5)
        Gui:textEx(Cache.Font('Exo2', 12), '2,300', 1.0, 1.0, 1.0, 1.0)
        Gui:setStretch(0.0, 0.0)
        Gui:setAlign(1.0, 0.5)
        Gui:endContainer()
        Gui:setStretch(1.0, 0.0)
        Gui:setAlign(0.4, 0.5)
        Gui:beginVerticalContainer()
        Gui:textEx(Cache.Font('Exo2', 14), '6.50', 1.0, 1.0, 1.0, 1.0)
        Gui:setStretch(0.0, 0.0)
        Gui:setAlign(1.0, 0.5)
        Gui:textEx(Cache.Font('Exo2', 12), '1300', 1.0, 1.0, 1.0, 1.0)
        Gui:setStretch(0.0, 0.0)
        Gui:setAlign(1.0, 0.5)
        Gui:endContainer()
        Gui:setStretch(1.0, 0.0)
        Gui:setAlign(0.8, 0.5)
        Gui:beginVerticalContainer()
        Gui:textEx(Cache.Font('Exo2', 14), '2,300', 1.0, 1.0, 1.0, 1.0)
        Gui:setStretch(0.0, 0.0)
        Gui:setAlign(1.0, 0.5)
        Gui:textEx(Cache.Font('Exo2', 14), '1300', 1.0, 1.0, 1.0, 1.0)
        Gui:setStretch(0.0, 0.0)
        Gui:setAlign(1.0, 0.5)
        Gui:endContainer()
        Gui:setStretch(1.0, 0.0)
        Gui:setAlign(1.0, 0.5)
        Gui:endContainer()
        Gui:setStretch(1.0, 0.0)
        Gui:setAlign(0.0, 0.0)
        Gui:beginHorizontalContainer()
        Gui:beginVerticalContainer()
        Gui:textEx(Cache.Font('Exo2', 14), 'Steel', 1.0, 1.0, 1.0, 1.0)
        Gui:setStretch(0.0, 0.0)
        Gui:setAlign(0.0, 0.5)
        Gui:textEx(Cache.Font('Exo2', 12), 'STL.Titan', 1.0, 1.0, 1.0, 1.0)
        Gui:setStretch(0.0, 0.0)
        Gui:setAlign(0.0, 0.5)
        Gui:endContainer()
        Gui:setStretch(1.0, 0.0)
        Gui:setAlign(0.0, 0.5)
        Gui:beginVerticalContainer()
        Gui:textEx(Cache.Font('Exo2', 14), '850', 1.0, 1.0, 1.0, 1.0)
        Gui:setStretch(0.0, 0.0)
        Gui:setAlign(1.0, 0.5)
        Gui:textEx(Cache.Font('Exo2', 12), '10', 1.0, 1.0, 1.0, 1.0)
        Gui:setStretch(0.0, 0.0)
        Gui:setAlign(1.0, 0.5)
        Gui:endContainer()
        Gui:setStretch(1.0, 0.0)
        Gui:setAlign(0.4, 0.5)
        Gui:beginVerticalContainer()
        Gui:textEx(Cache.Font('Exo2', 14), '817', 1.0, 1.0, 1.0, 1.0)
        Gui:setStretch(0.0, 0.0)
        Gui:setAlign(1.0, 0.5)
        Gui:textEx(Cache.Font('Exo2', 12), '13', 1.0, 1.0, 1.0, 1.0)
        Gui:setStretch(0.0, 0.0)
        Gui:setAlign(1.0, 0.5)
        Gui:endContainer()
        Gui:setStretch(1.0, 0.0)
        Gui:setAlign(0.8, 0.5)
        Gui:beginVerticalContainer()
        Gui:textEx(Cache.Font('Exo2', 14), '3001', 1.0, 1.0, 1.0, 1.0)
        Gui:setStretch(0.0, 0.0)
        Gui:setAlign(1.0, 0.5)
        Gui:textEx(Cache.Font('Exo2', 14), '306', 1.0, 1.0, 1.0, 1.0)
        Gui:setStretch(0.0, 0.0)
        Gui:setAlign(1.0, 0.5)
        Gui:endContainer()
        Gui:setStretch(1.0, 0.0)
        Gui:setAlign(1.0, 0.5)
        Gui:endContainer()
        Gui:setStretch(1.0, 0.0)
        Gui:setAlign(0.0, 0.0)
        Gui:endScroll(InputInstance)
        Gui:endContainer()
        -- Gui:setStretch(0.95, 0.95)
        -- Gui:setAlign(1.0, 0.05)
        Gui:endContainer()
        Gui:setStretch(0.95, 0.9)
        Gui:setAlign(0.1, 0.05)
        Gui:endContainer() -- end Commodity Window Panel
        -- Gui:setStretch(1.0, 1.0)
        -- Gui:setAlign(0.5, 1.0)
        Gui:endContainer() -- end Construction Materials Panel
        Gui:setStretch(1.0, 1.0)

        Gui:beginStackContainer() -- Order Book (offers/requests) Panel
        -- Create a nice border
        Gui:rect(1.0, 1.0, 0.7, 0.7, 0.7, 1.0);
        Gui:setStretch(1.0, 1.0)
        Gui:rect(1.0, 1.0, 0.1, 0.1, 0.1, 1.0);
        Gui:setStretch(0.992, 0.997)
        Gui:setAlign(0.5, 0.5)

        Gui:beginVerticalContainer()   -- Commodity Window Panel
        Gui:beginHorizontalContainer() -- Commodity Window Panel; search filter
        Gui:textEx(Cache.Font('Exo2', 20), 'Search:', 1.0, 1.0, 1.0, 1.0)
        Gui:setStretch(0.5, 0.0)
        Gui:setAlign(0.5, 0.49)
        Gui:beginStackContainer()
        Gui:rect(1.0, 1.0, 0.2, 0.2, 0.2, 1.0);
        Gui:setStretch(1.0, 0.6)
        Gui:setAlign(0.5, 0.5)
        Gui:textEx(Cache.Font('Exo2', 16), 'Current Station', 1.0, 1.0, 1.0, 1.0)
        Gui:setStretch(0.7, 0.0)
        Gui:setAlign(0.2, 0.5)
        Gui:endContainer()
        Gui:setStretch(1.2, 1.0)
        Gui:setAlign(0.0, 0.5)
        Gui:pushFont(Cache.Font('Exo2Bold', 16))
        Gui:button("V")
        Gui:setStretch(0.0, 0.0)
        Gui:setAlign(1.0, 0.5)
        Gui:popStyle(1)
        Gui:endContainer()
        Gui:setStretch(0.965, 0.2)
        Gui:setAlign(0.5, 0.05)

        Gui:rect(1.0, 1.0, 0.7, 0.7, 0.7, 1.0);
        Gui:setStretch(0.995, 0.05)
        Gui:setAlign(0.5, 0.0)

        Gui:beginHorizontalContainer() -- Commodity Window Panel; commodity search textbox
        Gui:beginStackContainer()
        Gui:rect(1.0, 1.0, 0.2, 0.2, 0.2, 1.0);
        Gui:setStretch(0.98, 0.9)
        Gui:setAlign(0.5, 0.5)
        Gui:textEx(Cache.Font('Exo2', 16), 'Search Commodity', 1.0, 1.0, 1.0, 1.0)
        Gui:setStretch(0.7, 0.0)
        Gui:setAlign(0.2, 0.5)
        Gui:endContainer()
        Gui:endContainer()
        Gui:setStretch(0.965, 0.3)
        Gui:setAlign(0.5, 0.0)

        Gui:rect(1.0, 1.0, 0.7, 0.7, 0.7, 1.0);
        Gui:setStretch(0.995, 0.05)
        Gui:setAlign(0.5, 0.0)

        Gui:beginStackContainer()
        Gui:rect(1.0, 1.0, 0.15, 0.15, 0.15, 1.0);

        Gui:beginStackContainer() -- Commodity List Panel
        -- The BeginScroll() calculation is a hack to keep the length of the scrollable area
        -- inside the bound of its enclosing GroupStack. It glitches when the full game window
        -- is vertically resized too far (but not all the way to the full vertical extent).
        Gui:beginScroll(634 + (self.resY - 900))
        Gui:textEx(Cache.Font('Exo2', 14), 'Agricultural Products', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Alloys', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Animals, Livestock', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Animals, Rare', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Art, Audio Recordings', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Art, Paintings', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Art, Sculptures', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Chemicals', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Clothing', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Computer Parts', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Computers', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Construction Materials', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Consumables', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Crystals, Gems, Non-Precious', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Crystals, Gems, Precious', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Crystals, Raw', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Currency, Coinage', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Currency, Paper', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Drones', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Electronics', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Elements', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Energy Systems', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Fertilizers', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Foods, Animal', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Foods, Gourmet', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Foods, Plants', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Fuels', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Furniture', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Gases', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Hydrocarbons, Raw', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Liquids', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Medical Equipment', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Metals', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Minerals', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Nitrogen Compounds', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Ores, Metal, Ferrous', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Ores, Metal, Non-Ferrous', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Ores, Non-Metal', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Ores, Radioactive', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Petrochemicals', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Pharmaceuticals', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Plants', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Plants, Rare', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Plastics', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Polymers', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Robots', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Ship Engines', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Ship Parts', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Ship Shields', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Software Components', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Software', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Special Alloys', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Spices', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Textiles', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Unit Prefabs', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Utility', 1.0, 1.0, 1.0, 1.0)
        Gui:endScroll(InputInstance)
        Gui:endContainer()
        Gui:endContainer()
        Gui:endContainer() -- end Commodity Window Panel
        Gui:setStretch(1.0, 1.0)
        Gui:endContainer() -- Order Book (offers/requests) Panel
        Gui:setStretch(1.0, 1.0)

        Gui:beginStackContainer() -- Place Order Panel
        -- Create a nice border
        Gui:rect(1.0, 1.0, 0.7, 0.7, 0.7, 1.0);
        Gui:setStretch(1.0, 1.0)
        Gui:rect(1.0, 1.0, 0.1, 0.1, 0.1, 1.0);
        Gui:setStretch(0.992, 0.997)
        Gui:setAlign(0.5, 0.5)

        Gui:beginVerticalContainer()   -- Commodity Window Panel
        Gui:beginHorizontalContainer() -- Commodity Window Panel; search filter
        Gui:textEx(Cache.Font('Exo2', 20), 'Search:', 1.0, 1.0, 1.0, 1.0)
        Gui:setStretch(0.5, 0.0)
        Gui:setAlign(0.5, 0.49)
        Gui:beginStackContainer()
        Gui:rect(1.0, 1.0, 0.2, 0.2, 0.2, 1.0);
        Gui:setStretch(1.0, 0.6)
        Gui:setAlign(0.5, 0.5)
        Gui:textEx(Cache.Font('Exo2', 16), 'Current Station', 1.0, 1.0, 1.0, 1.0)
        Gui:setStretch(0.7, 0.0)
        Gui:setAlign(0.2, 0.5)
        Gui:endContainer()
        Gui:setStretch(1.2, 1.0)
        Gui:setAlign(0.0, 0.5)
        Gui:pushFont(Cache.Font('Exo2Bold', 16))
        Gui:button("V")
        Gui:setStretch(0.0, 0.0)
        Gui:setAlign(1.0, 0.5)
        Gui:popStyle(1)
        Gui:endContainer()
        Gui:setStretch(0.965, 0.2)
        Gui:setAlign(0.5, 0.05)

        Gui:rect(1.0, 1.0, 0.7, 0.7, 0.7, 1.0);
        Gui:setStretch(0.995, 0.05)
        Gui:setAlign(0.5, 0.0)

        Gui:beginHorizontalContainer() -- Commodity Window Panel; commodity search textbox
        Gui:beginStackContainer()
        Gui:rect(1.0, 1.0, 0.2, 0.2, 0.2, 1.0);
        Gui:setStretch(0.98, 0.9)
        Gui:setAlign(0.5, 0.5)
        Gui:textEx(Cache.Font('Exo2', 16), 'Search Commodity', 1.0, 1.0, 1.0, 1.0)
        Gui:setStretch(0.7, 0.0)
        Gui:setAlign(0.2, 0.5)
        Gui:endContainer()
        Gui:endContainer()
        Gui:setStretch(0.965, 0.3)
        Gui:setAlign(0.5, 0.0)

        Gui:rect(1.0, 1.0, 0.7, 0.7, 0.7, 1.0);
        Gui:setStretch(0.995, 0.05)
        Gui:setAlign(0.5, 0.0)

        Gui:beginStackContainer()
        Gui:rect(1.0, 1.0, 0.15, 0.15, 0.15, 1.0);

        Gui:beginStackContainer() -- Commodity List Panel
        -- The BeginScroll() calculation is a hack to keep the length of the scrollable area
        -- inside the bound of its enclosing GroupStack. It glitches when the full game window
        -- is vertically resized too far (but not all the way to the full vertical extent).
        Gui:beginScroll(634 + (self.resY - 900))
        Gui:textEx(Cache.Font('Exo2', 14), 'Agricultural Products', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Alloys', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Animals, Livestock', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Animals, Rare', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Art, Audio Recordings', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Art, Paintings', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Art, Sculptures', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Chemicals', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Clothing', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Computer Parts', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Computers', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Construction Materials', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Consumables', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Crystals, Gems, Non-Precious', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Crystals, Gems, Precious', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Crystals, Raw', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Currency, Coinage', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Currency, Paper', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Drones', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Electronics', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Elements', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Energy Systems', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Fertilizers', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Foods, Animal', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Foods, Gourmet', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Foods, Plants', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Fuels', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Furniture', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Gases', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Hydrocarbons, Raw', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Liquids', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Medical Equipment', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Metals', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Minerals', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Nitrogen Compounds', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Ores, Metal, Ferrous', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Ores, Metal, Non-Ferrous', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Ores, Non-Metal', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Ores, Radioactive', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Petrochemicals', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Pharmaceuticals', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Plants', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Plants, Rare', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Plastics', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Polymers', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Robots', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Ship Engines', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Ship Parts', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Ship Shields', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Software Components', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Software', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Special Alloys', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Spices', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Textiles', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Unit Prefabs', 1.0, 1.0, 1.0, 1.0)
        Gui:textEx(Cache.Font('Exo2', 14), 'Utility', 1.0, 1.0, 1.0, 1.0)
        Gui:endScroll(InputInstance)
        Gui:endContainer()
        Gui:endContainer()
        Gui:endContainer() -- end Commodity Window Panel
        Gui:setStretch(1.0, 1.0)
        Gui:endContainer() -- end Place Order Panel
        Gui:setStretch(1.0, 1.0)

        Gui:endContainer()                  -- end Commodity Screen
        Gui:setStretch(1.0, 1.0)
    elseif station_options_market == 2 then -- Data/Intel Screen
        Gui:beginScroll(200)
        Gui:checkbox("Thing1", true)
        Gui:checkbox("Thing2", false)
        Gui:endScroll(InputInstance)        -- end Data/Intel Screen
    elseif station_options_market == 3 then -- Exchange Orders Screen
        Gui:beginScroll(200)
        Gui:button("Parnellite")
        Gui:button("Glorboscite")
        Gui:button("Lonsdaleite")
        Gui:button("Ketracel White")
        Gui:endScroll(InputInstance)        -- end Exchange Orders Screen
    elseif station_options_market == 4 then -- Blank Screen
        Gui:beginScroll(200)
        Gui:endScroll(InputInstance)        -- end Blank Screen
    end
    Gui:endContainer()
    Gui:setStretch(1.0, 1.0)
    Gui:endContainer()
    Gui:setStretch(1.0, 1.0)
    Gui:endContainer()
    Gui:setStretch(1.0, 1.0)
    Gui:endContainer()
    Gui:setStretch(1.0, 1.0)
end

function Test:showGameCtrlInner()
    Gui:beginVerticalContainer()
    Gui:pushTextColor(1.0, 1.0, 1.0, 1.0)
    Gui:pushFont(Cache.Font('Exo2Bold', 18))
    if Gui:button("Cancel") then
        drawExitMenu = false
    end
    Gui:setSpacing(8)
    if Gui:button("Quit") then
        Test:quit()
    end
    Gui:endContainer()
end

function Test:showCtrlMenu()
    Gui:beginWindow("Game Control")
    Gui:textEx(Cache.Font('Iceland', 20), 'Game Control', 0.3, 0.4, 0.5, 1.0)
    Gui:setAlign(0.5, 0.5)
    Gui:setSpacing(16)
    self:showGameCtrlInner()
    Gui:endWindow()
    Gui:setAlign(0.5, 0.5)
end

function Test:showMetrics()
    Gui:beginWindow("Metrics")
    Gui:text(format("fps: %.2f", 1.0 / self.dt))
    Gui:endWindow()
end

function Test:onUpdate(dt)
    Profiler.Begin('Gui:update')
    if InputInstance:isPressed(Button.KeyboardEscape) then
        drawExitMenu = not drawExitMenu
    end
    Gui:beginGui(self.resX, self.resY)
    if drawExitMenu then
        Gui:beginStackContainer()
        self:showCtrlMenu()
        Gui:endContainer()
    else
        -- self:showMetrics()
        self:showMenuDrop()
    end
    Gui:endGui(InputInstance)
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
