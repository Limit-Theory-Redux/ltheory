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
    HmGui.BeginWindow("Screen Title", InputInstance)
    if station_options == 1 then
        HmGui.TextEx(Cache.Font('Iceland', 64), 'MARKETPLACE', 1.0, 1.0, 1.0, 1.0)
    elseif station_options == 2 then
        HmGui.TextEx(Cache.Font('Iceland', 64), 'SHIP CONFIGURATION', 1.0, 1.0, 1.0, 1.0)
    elseif station_options == 3 then
        HmGui.TextEx(Cache.Font('Iceland', 64), 'OCCUPATIONAL', 1.0, 1.0, 1.0, 1.0)
    else
        HmGui.TextEx(Cache.Font('Iceland', 64), 'WTF?', 1.0, 1.0, 1.0, 1.0)
    end
    HmGui.SetAlign(0.5, 0.0)
    self:showMenuDropInner()
    HmGui.EndWindow()
    HmGui.SetStretch(0.97, 0.95)
    HmGui.SetAlign(0.5, 0.5)
end

function Test:showMenuDropInner()
    HmGui.BeginGroupY()
    HmGui.BeginGroupX() -- upper group: 1) location text, 2) station services type buttons, 3) cash on hand text
    HmGui.BeginGroupY()
    HmGui.TextEx(Cache.Font('Iceland', 32), 'Docked at', 1.0, 1.0, 1.0, 1.0)
    HmGui.SetAlign(0.5, 0.2)
    HmGui.TextEx(Cache.Font('Iceland', 32), 'Titan Station', 1.0, 1.0, 1.0, 1.0)
    HmGui.SetAlign(0.5, 0.2)
    HmGui.TextEx(Cache.Font('Iceland', 24), 'Parnell System', 1.0, 1.0, 1.0, 1.0)
    HmGui.SetAlign(0.5, 0.2)
    HmGui.EndGroup()
    HmGui.SetStretch(0.7, 0.0)
    HmGui.BeginGroupX()
    if station_options == 1 then -- Station services: Marketplace
        if station_options_market == 1 then
            HmGui.PushFont(Cache.Font('RajdhaniBold', 24))
            HmGui.PushTextColor(0.9, 0.2, 0.8, 1.0)
        else
            HmGui.PushFont(Cache.Font('Rajdhani', 24))
            HmGui.PushTextColor(0.7, 0.0, 0.6, 1.0)
        end
        if HmGui.Button("COMMODITIES") then station_options_market = 1 end
        HmGui.PopStyle(2)
        if station_options_market == 2 then
            HmGui.PushFont(Cache.Font('RajdhaniBold', 24))
            HmGui.PushTextColor(0.9, 0.2, 0.8, 1.0)
        else
            HmGui.PushFont(Cache.Font('Rajdhani', 24))
            HmGui.PushTextColor(0.7, 0.0, 0.6, 1.0)
        end
        if HmGui.Button("DATA / INTEL") then station_options_market = 2 end
        HmGui.PopStyle(2)
        if station_options_market == 3 then
            HmGui.PushFont(Cache.Font('RajdhaniBold', 24))
            HmGui.PushTextColor(0.9, 0.2, 0.8, 1.0)
        else
            HmGui.PushFont(Cache.Font('Rajdhani', 24))
            HmGui.PushTextColor(0.7, 0.0, 0.6, 1.0)
        end
        if HmGui.Button("EXCHANGE ORDERS") then station_options_market = 3 end
        HmGui.PopStyle(2)
        if station_options_market == 4 then
            HmGui.PushFont(Cache.Font('RajdhaniBold', 24))
            HmGui.PushTextColor(0.9, 0.2, 0.8, 1.0)
        else
            HmGui.PushFont(Cache.Font('Rajdhani', 24))
            HmGui.PushTextColor(0.7, 0.0, 0.6, 1.0)
        end
        if HmGui.Button("BLANK") then station_options_market = 4 end
        HmGui.PopStyle(2)
    elseif station_options == 2 then -- Station services: ship config
        if station_options_config == 1 then
            HmGui.PushFont(Cache.Font('RajdhaniBold', 24))
            HmGui.PushTextColor(0.9, 0.8, 0.2, 1.0)
        else
            HmGui.PushFont(Cache.Font('Rajdhani', 24))
            HmGui.PushTextColor(0.7, 0.6, 0.0, 1.0)
        end
        if HmGui.Button("SHIP OVERVIEW") then station_options_config = 1 end
        HmGui.PopStyle(2)
        if station_options_config == 2 then
            HmGui.PushFont(Cache.Font('RajdhaniBold', 24))
            HmGui.PushTextColor(0.9, 0.8, 0.2, 1.0)
        else
            HmGui.PushFont(Cache.Font('Rajdhani', 24))
            HmGui.PushTextColor(0.7, 0.6, 0.0, 1.0)
        end
        if HmGui.Button("LOADOUT") then station_options_config = 2 end
        HmGui.PopStyle(2)
        if station_options_config == 3 then
            HmGui.PushFont(Cache.Font('RajdhaniBold', 24))
            HmGui.PushTextColor(0.9, 0.8, 0.2, 1.0)
        else
            HmGui.PushFont(Cache.Font('Rajdhani', 24))
            HmGui.PushTextColor(0.7, 0.6, 0.0, 1.0)
        end
        if HmGui.Button("BUY/SELL EQUIPMENT") then station_options_config = 3 end
        HmGui.PopStyle(2)
        if station_options_config == 4 then
            HmGui.PushFont(Cache.Font('RajdhaniBold', 24))
            HmGui.PushTextColor(0.9, 0.8, 0.2, 1.0)
        else
            HmGui.PushFont(Cache.Font('Rajdhani', 24))
            HmGui.PushTextColor(0.7, 0.6, 0.0, 1.0)
        end
        if HmGui.Button("SHIPYARD") then station_options_config = 4 end
        HmGui.PopStyle(2)
    elseif station_options == 3 then -- Station services: Occupational
        if station_options_occupy == 1 then
            HmGui.PushFont(Cache.Font('RajdhaniBold', 24))
            HmGui.PushTextColor(0.9, 0.2, 0.2, 1.0)
        else
            HmGui.PushFont(Cache.Font('Rajdhani', 24))
            HmGui.PushTextColor(0.7, 0.0, 0.0, 1.0)
        end
        if HmGui.Button("FLEETS") then station_options_occupy = 1 end
        HmGui.PopStyle(2)
        if station_options_occupy == 2 then
            HmGui.PushFont(Cache.Font('RajdhaniBold', 24))
            HmGui.PushTextColor(0.9, 0.2, 0.2, 1.0)
        else
            HmGui.PushFont(Cache.Font('Rajdhani', 24))
            HmGui.PushTextColor(0.7, 0.0, 0.0, 1.0)
        end
        if HmGui.Button("PROJECTS") then station_options_occupy = 2 end
        HmGui.PopStyle(2)
        if station_options_occupy == 3 then
            HmGui.PushFont(Cache.Font('RajdhaniBold', 24))
            HmGui.PushTextColor(0.9, 0.2, 0.2, 1.0)
        else
            HmGui.PushFont(Cache.Font('Rajdhani', 24))
            HmGui.PushTextColor(0.7, 0.0, 0.0, 1.0)
        end
        if HmGui.Button("CONTRACTS") then station_options_occupy = 3 end
        HmGui.PopStyle(2)
        if station_options_occupy == 4 then
            HmGui.PushFont(Cache.Font('RajdhaniBold', 24))
            HmGui.PushTextColor(0.9, 0.2, 0.2, 1.0)
        else
            HmGui.PushFont(Cache.Font('Rajdhani', 24))
            HmGui.PushTextColor(0.7, 0.0, 0.0, 1.0)
        end
        if HmGui.Button("FACTIONS") then station_options_occupy = 4 end
        HmGui.PopStyle(2)
    end
    HmGui.EndGroup()
    HmGui.SetStretch(1.0, 1.0)
    HmGui.TextEx(Cache.Font('Iceland', 24), '$300,561,000', 1.0, 1.0, 1.0, 1.0)
    HmGui.SetAlign(2.0, 0.5)
    HmGui.SetStretch(0.1, 0.0)
    HmGui.EndGroup()
    HmGui.SetStretch(1.0, 0.1)

    -- divider (ImGui.Divider is a thing that exists. HmGui.Divider does not. So this is a kludge.
    HmGui.SetSpacing(16)
    HmGui.Rect(100.0, 6.0, 0.3, 0.3, 0.3, 1.0);
    HmGui.SetStretch(1.0, 0.0)
    HmGui.SetSpacing(16)

    HmGui.BeginGroupX() -- lower group, 2 windows: 1) Station buttons, 2) selected top button details
    HmGui.BeginGroupX() -- Station buttons
    HmGui.SetAlign(0.0, 0.0)
    HmGui.SetStretch(0.1, 1.0)
    HmGui.BeginGroupY()
    HmGui.BeginGroupY()
    HmGui.BeginGroupY()
    HmGui.PushFont(Cache.Font('RajdhaniBold', 32))
    if station_options == 1 then
        HmGui.PushTextColor(1.0, 0.3, 0.9, 1.0)
    else
        HmGui.PushTextColor(0.5, 0.0, 0.4, 1.0)
    end
    if HmGui.Button("MARKET") then station_options = 1 end
    HmGui.PopStyle(2)
    HmGui.SetStretch(1.0, 1.8)
    HmGui.PushFont(Cache.Font('RajdhaniBold', 32))
    if station_options == 2 then
        HmGui.PushTextColor(0.9, 0.8, 0.2, 1.0)
    else
        HmGui.PushTextColor(0.5, 0.4, 0.0, 1.0)
    end
    if HmGui.Button("SHIP CONFIGURATION") then station_options = 2 end
    HmGui.PopStyle(2)
    HmGui.SetStretch(1.0, 1.8)
    HmGui.PushFont(Cache.Font('RajdhaniBold', 32))
    if station_options == 3 then
        HmGui.PushTextColor(0.9, 0.2, 0.2, 1.0)
    else
        HmGui.PushTextColor(0.5, 0.0, 0.0, 1.0)
    end
    if HmGui.Button("OCCUPATION INFO") then station_options = 3 end
    HmGui.PopStyle(2)
    HmGui.SetStretch(1.0, 1.8)
    HmGui.EndGroup()
    HmGui.SetAlign(0.5, 0.4)
    HmGui.SetStretch(1.0, 0.7)
    HmGui.SetSpacing(50)
    HmGui.BeginGroupY()
    HmGui.PushTextColor(0.2, 1.0, 0.3, 1.0)
    HmGui.PushFont(Cache.Font('RajdhaniBold', 36))
    HmGui.Button("LAUNCH")
    HmGui.PopStyle(2)
    HmGui.SetAlign(0.5, 0.0)
    HmGui.SetStretch(0.5, 0.0)
    HmGui.EndGroup()
    HmGui.SetAlign(0.5, 3.0)
    HmGui.SetStretch(1.0, 0.6)
    HmGui.EndGroup()
    HmGui.SetAlign(0.5, 1.0)
    HmGui.SetStretch(2.0, 1.0)
    HmGui.EndGroup()
    HmGui.SetStretch(0.1, 1.0)
    HmGui.EndGroup()

    HmGui.BeginGroupY() -- selected station service details: name and info
    if station_options_market == 1 then
        HmGui.TextEx(Cache.Font('Iceland', 24), 'COMMODITY EXCHANGE', 1.0, 1.0, 1.0, 1.0)
    elseif station_options_market == 2 then
        HmGui.TextEx(Cache.Font('Iceland', 24), 'DATA/INTEL', 1.0, 1.0, 1.0, 1.0)
    elseif station_options_market == 3 then
        HmGui.TextEx(Cache.Font('Iceland', 24), 'EXCHANGE ORDERS', 1.0, 1.0, 1.0, 1.0)
    else
        HmGui.TextEx(Cache.Font('Iceland', 24), 'BLANK', 1.0, 1.0, 1.0, 1.0)
    end
    HmGui.SetAlign(0.0, 0.5)
    HmGui.SetStretch(1.0, 0.0)

    HmGui.SetSpacing(16)

    HmGui.BeginGroupX()                 -- selected station service details
    if station_options_market == 1 then -- Commodities Screen
        HmGui.BeginGroupX()
        HmGui.BeginGroupStack()         -- Commodity Panel
        -- Create a nice border
        HmGui.Rect(1.0, 1.0, 0.7, 0.7, 0.7, 1.0);
        HmGui.SetStretch(1.0, 1.0)
        HmGui.Rect(1.0, 1.0, 0.1, 0.1, 0.1, 1.0);
        HmGui.SetStretch(0.992, 0.997)
        HmGui.SetAlign(0.5, 0.5)

        HmGui.BeginGroupY() -- Commodity Window Panel
        HmGui.BeginGroupX() -- Commodity Window Panel; search filter
        HmGui.TextEx(Cache.Font('Exo2', 20), 'Search:', 1.0, 1.0, 1.0, 1.0)
        HmGui.SetStretch(0.5, 0.0)
        HmGui.SetAlign(0.5, 0.6)

        HmGui.BeginGroupStack()
        HmGui.Rect(1.0, 1.0, 0.2, 0.2, 0.2, 1.0);
        HmGui.SetStretch(1.0, 0.6)
        HmGui.SetAlign(0.5, 0.5)
        HmGui.TextEx(Cache.Font('Exo2', 16), 'Current Station', 1.0, 1.0, 1.0, 1.0)
        HmGui.SetStretch(0.7, 0.0)
        HmGui.SetAlign(0.2, 0.5)
        HmGui.EndGroup()
        HmGui.SetStretch(1.2, 0.8)
        HmGui.SetAlign(0.0, 1.0)

        HmGui.PushFont(Cache.Font('Exo2Bold', 16))
        HmGui.Button("V")
        HmGui.PopStyle(1)
        HmGui.SetStretch(0.0, 0.0)
        HmGui.SetAlign(1.0, 1.0)
        HmGui.EndGroup()
        HmGui.SetStretch(0.965, 0.2)
        HmGui.SetAlign(0.5, 0.5)

        HmGui.Rect(1.0, 1.0, 0.7, 0.7, 0.7, 1.0);
        HmGui.SetStretch(0.995, 0.05)
        HmGui.SetAlign(0.5, 1.0)

        HmGui.BeginGroupX() -- Commodity Window Panel; commodity search textbox
        HmGui.BeginGroupStack()
        HmGui.Rect(1.0, 1.0, 0.2, 0.2, 0.2, 1.0);
        HmGui.SetStretch(0.98, 1.0)
        HmGui.SetAlign(0.5, 0.1)
        HmGui.TextEx(Cache.Font('Exo2', 16), 'Search Commodity', 1.0, 1.0, 1.0, 1.0)
        HmGui.SetStretch(0.7, 1.0)
        HmGui.SetAlign(0.2, 1.0)
        HmGui.EndGroup()
        HmGui.EndGroup()
        HmGui.SetStretch(0.965, 0.3)
        HmGui.SetAlign(0.5, 0.0)

        HmGui.Rect(1.0, 1.0, 0.7, 0.7, 0.7, 1.0);
        HmGui.SetStretch(0.995, 0.05)
        HmGui.SetAlign(0.5, 0.0)

        HmGui.BeginGroupStack()
        HmGui.Rect(1.0, 1.0, 0.15, 0.15, 0.15, 1.0);

        HmGui.BeginGroupStack() -- Commodity List Panel
        -- The BeginScroll() calculation is a hack to keep the length of the scrollable area
        -- inside the bound of its enclosing GroupStack. It glitches when the full game window
        -- is vertically resized too far (but not all the way to the full vertical extent).
        HmGui.BeginScroll(634 + (self.resY - 900))
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Agricultural Products', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Alloys', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Animals, Livestock', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Animals, Rare', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Art, Audio Recordings', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Art, Paintings', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Art, Sculptures', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Chemicals', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Clothing', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Computer Parts', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Computers', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Construction Materials', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Consumables', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Crystals, Gems, Non-Precious', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Crystals, Gems, Precious', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Crystals, Raw', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Currency, Coinage', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Currency, Paper', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Drones', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Electronics', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Elements', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Energy Systems', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Fertilizers', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Foods, Animal', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Foods, Gourmet', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Foods, Plants', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Fuels', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Furniture', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Gases', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Hydrocarbons, Raw', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Liquids', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Medical Equipment', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Metals', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Minerals', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Nitrogen Compounds', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Ores, Metal, Ferrous', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Ores, Metal, Non-Ferrous', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Ores, Non-Metal', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Ores, Radioactive', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Petrochemicals', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Pharmaceuticals', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Plants', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Plants, Rare', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Plastics', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Polymers', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Robots', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Ship Engines', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Ship Parts', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Ship Shields', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Software Components', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Software', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Special Alloys', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Spices', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Textiles', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Unit Prefabs', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Utility', 1.0, 1.0, 1.0, 1.0)
        HmGui.EndScroll(InputInstance)
        HmGui.EndGroup()
        HmGui.EndGroup()
        HmGui.EndGroup() -- end Commodity Window Panel
        HmGui.SetStretch(1.0, 1.0)
        HmGui.EndGroup() -- end Commodity Panel
        HmGui.SetStretch(1.0, 1.0)

        HmGui.BeginGroupY() -- Construction Materials Table Panel
        HmGui.BeginGroupX()
        HmGui.TextEx(Cache.Font('Exo2', 18), 'Construction Materials', 1.0, 1.0, 1.0, 1.0)
        HmGui.SetStretch(0.5, 0.0)
        HmGui.SetAlign(0.2, 0.5)
        HmGui.PushFont(Cache.Font('Exo2Bold', 16))
        HmGui.Button("Price Chart")
        HmGui.SetStretch(0.0, 0.0)
        HmGui.SetAlign(1.0, 0.5)
        HmGui.PopStyle(1)
        HmGui.EndGroup()
        HmGui.SetStretch(0.965, 0.0)
        HmGui.SetAlign(0.5, 0.0)

        HmGui.BeginGroupStack() -- Construction Materials Panel
        -- Create a nice border with internal borders
        HmGui.Rect(1.0, 1.0, 0.7, 0.7, 0.7, 1.0);
        HmGui.SetStretch(1.0, 1.0)
        HmGui.Rect(1.0, 1.0, 0.1, 0.1, 0.1, 1.0);
        HmGui.SetStretch(0.992, 0.997)
        HmGui.SetAlign(0.5, 0.5)

        HmGui.Rect(1.0, 1.0, 0.7, 0.7, 0.7, 1.0);
        HmGui.SetStretch(0.0025, 0.997)
        HmGui.SetAlign(0.31, 0.05)

        HmGui.Rect(1.0, 1.0, 0.7, 0.7, 0.7, 1.0);
        HmGui.SetStretch(0.0025, 0.997)
        HmGui.SetAlign(0.53, 0.05)

        HmGui.Rect(1.0, 1.0, 0.7, 0.7, 0.7, 1.0);
        HmGui.SetStretch(0.0025, 0.997)
        HmGui.SetAlign(0.74, 0.05)

        HmGui.Rect(1.0, 1.0, 0.7, 0.7, 0.7, 1.0);
        HmGui.SetStretch(0.995, 0.002)
        HmGui.SetAlign(0.5, 0.054)

        HmGui.BeginGroupY()
        HmGui.BeginGroupX()
        HmGui.BeginGroupY()
        HmGui.TextEx(Cache.Font('Exo2', 18), 'Commodity', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Ticker', 1.0, 1.0, 1.0, 1.0)
        HmGui.EndGroup()
        HmGui.BeginGroupY()
        HmGui.TextEx(Cache.Font('Exo2', 18), 'Ask', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Amount', 1.0, 1.0, 1.0, 1.0)
        HmGui.EndGroup()
        HmGui.BeginGroupY()
        HmGui.TextEx(Cache.Font('Exo2', 18), 'Bid', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Amount', 1.0, 1.0, 1.0, 1.0)
        HmGui.EndGroup()
        HmGui.BeginGroupY()
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Supply', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Demand', 1.0, 1.0, 1.0, 1.0)
        HmGui.EndGroup()
        HmGui.EndGroup()
        HmGui.SetStretch(0.965, 0.0)
        HmGui.SetAlign(0.5, 1.0)

        HmGui.BeginGroupStack() -- Commodity List Panel
        -- The BeginScroll() calculation is a hack to keep the length of the scrollable area
        -- inside the bound of its enclosing GroupStack. It glitches when the full game window
        -- is vertically resized too far (but not all the way to the full vertical extent).
        HmGui.BeginScroll(500 + (self.resY - 900))
        HmGui.BeginGroupX()
        HmGui.BeginGroupY()
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Nano Fiber', 1.0, 1.0, 1.0, 1.0)
        HmGui.SetStretch(0.0, 0.0)
        HmGui.SetAlign(0.0, 0.5)
        HmGui.TextEx(Cache.Font('Exo2', 12), 'NF.Titan', 1.0, 1.0, 1.0, 1.0)
        HmGui.SetStretch(0.0, 0.0)
        HmGui.SetAlign(0.0, 0.5)
        HmGui.EndGroup()
        HmGui.SetStretch(1.0, 0.0)
        HmGui.SetAlign(0.0, 0.5)
        HmGui.BeginGroupY()
        HmGui.TextEx(Cache.Font('Exo2', 14), '9.99', 1.0, 1.0, 1.0, 1.0)
        HmGui.SetStretch(0.0, 0.0)
        HmGui.SetAlign(1.0, 0.5)
        HmGui.TextEx(Cache.Font('Exo2', 12), '2,300', 1.0, 1.0, 1.0, 1.0)
        HmGui.SetStretch(0.0, 0.0)
        HmGui.SetAlign(1.0, 0.5)
        HmGui.EndGroup()
        HmGui.SetStretch(1.0, 0.0)
        HmGui.SetAlign(0.4, 0.5)
        HmGui.BeginGroupY()
        HmGui.TextEx(Cache.Font('Exo2', 14), '6.50', 1.0, 1.0, 1.0, 1.0)
        HmGui.SetStretch(0.0, 0.0)
        HmGui.SetAlign(1.0, 0.5)
        HmGui.TextEx(Cache.Font('Exo2', 12), '1300', 1.0, 1.0, 1.0, 1.0)
        HmGui.SetStretch(0.0, 0.0)
        HmGui.SetAlign(1.0, 0.5)
        HmGui.EndGroup()
        HmGui.SetStretch(1.0, 0.0)
        HmGui.SetAlign(0.8, 0.5)
        HmGui.BeginGroupY()
        HmGui.TextEx(Cache.Font('Exo2', 14), '2,300', 1.0, 1.0, 1.0, 1.0)
        HmGui.SetStretch(0.0, 0.0)
        HmGui.SetAlign(1.0, 0.5)
        HmGui.TextEx(Cache.Font('Exo2', 14), '1300', 1.0, 1.0, 1.0, 1.0)
        HmGui.SetStretch(0.0, 0.0)
        HmGui.SetAlign(1.0, 0.5)
        HmGui.EndGroup()
        HmGui.SetStretch(1.0, 0.0)
        HmGui.SetAlign(1.0, 0.5)
        HmGui.EndGroup()
        HmGui.SetStretch(1.0, 0.0)
        HmGui.SetAlign(0.0, 0.0)
        HmGui.BeginGroupX()
        HmGui.BeginGroupY()
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Steel', 1.0, 1.0, 1.0, 1.0)
        HmGui.SetStretch(0.0, 0.0)
        HmGui.SetAlign(0.0, 0.5)
        HmGui.TextEx(Cache.Font('Exo2', 12), 'STL.Titan', 1.0, 1.0, 1.0, 1.0)
        HmGui.SetStretch(0.0, 0.0)
        HmGui.SetAlign(0.0, 0.5)
        HmGui.EndGroup()
        HmGui.SetStretch(1.0, 0.0)
        HmGui.SetAlign(0.0, 0.5)
        HmGui.BeginGroupY()
        HmGui.TextEx(Cache.Font('Exo2', 14), '850', 1.0, 1.0, 1.0, 1.0)
        HmGui.SetStretch(0.0, 0.0)
        HmGui.SetAlign(1.0, 0.5)
        HmGui.TextEx(Cache.Font('Exo2', 12), '10', 1.0, 1.0, 1.0, 1.0)
        HmGui.SetStretch(0.0, 0.0)
        HmGui.SetAlign(1.0, 0.5)
        HmGui.EndGroup()
        HmGui.SetStretch(1.0, 0.0)
        HmGui.SetAlign(0.4, 0.5)
        HmGui.BeginGroupY()
        HmGui.TextEx(Cache.Font('Exo2', 14), '817', 1.0, 1.0, 1.0, 1.0)
        HmGui.SetStretch(0.0, 0.0)
        HmGui.SetAlign(1.0, 0.5)
        HmGui.TextEx(Cache.Font('Exo2', 12), '13', 1.0, 1.0, 1.0, 1.0)
        HmGui.SetStretch(0.0, 0.0)
        HmGui.SetAlign(1.0, 0.5)
        HmGui.EndGroup()
        HmGui.SetStretch(1.0, 0.0)
        HmGui.SetAlign(0.8, 0.5)
        HmGui.BeginGroupY()
        HmGui.TextEx(Cache.Font('Exo2', 14), '3001', 1.0, 1.0, 1.0, 1.0)
        HmGui.SetStretch(0.0, 0.0)
        HmGui.SetAlign(1.0, 0.5)
        HmGui.TextEx(Cache.Font('Exo2', 14), '306', 1.0, 1.0, 1.0, 1.0)
        HmGui.SetStretch(0.0, 0.0)
        HmGui.SetAlign(1.0, 0.5)
        HmGui.EndGroup()
        HmGui.SetStretch(1.0, 0.0)
        HmGui.SetAlign(1.0, 0.5)
        HmGui.EndGroup()
        HmGui.SetStretch(1.0, 0.0)
        HmGui.SetAlign(0.0, 0.0)
        HmGui.EndScroll(InputInstance)
        HmGui.EndGroup()
        --                    HmGui.SetStretch(0.95, 0.95)
        --                    HmGui.SetAlign(1.0, 0.05)
        HmGui.EndGroup()
        HmGui.SetStretch(0.95, 0.9)
        HmGui.SetAlign(0.1, 0.05)
        HmGui.EndGroup() -- end Commodity Window Panel
        --                HmGui.SetStretch(1.0, 1.0)
        --                HmGui.SetAlign(0.5, 1.0)
        HmGui.EndGroup() -- end Construction Materials Panel
        HmGui.SetStretch(1.0, 1.0)

        HmGui.BeginGroupStack() -- Order Book (offers/requests) Panel
        -- Create a nice border
        HmGui.Rect(1.0, 1.0, 0.7, 0.7, 0.7, 1.0);
        HmGui.SetStretch(1.0, 1.0)
        HmGui.Rect(1.0, 1.0, 0.1, 0.1, 0.1, 1.0);
        HmGui.SetStretch(0.992, 0.997)
        HmGui.SetAlign(0.5, 0.5)

        HmGui.BeginGroupY() -- Commodity Window Panel
        HmGui.BeginGroupX() -- Commodity Window Panel; search filter
        HmGui.TextEx(Cache.Font('Exo2', 20), 'Search:', 1.0, 1.0, 1.0, 1.0)
        HmGui.SetStretch(0.5, 0.0)
        HmGui.SetAlign(0.5, 0.49)
        HmGui.BeginGroupStack()
        HmGui.Rect(1.0, 1.0, 0.2, 0.2, 0.2, 1.0);
        HmGui.SetStretch(1.0, 0.6)
        HmGui.SetAlign(0.5, 0.5)
        HmGui.TextEx(Cache.Font('Exo2', 16), 'Current Station', 1.0, 1.0, 1.0, 1.0)
        HmGui.SetStretch(0.7, 0.0)
        HmGui.SetAlign(0.2, 0.5)
        HmGui.EndGroup()
        HmGui.SetStretch(1.2, 1.0)
        HmGui.SetAlign(0.0, 0.5)
        HmGui.PushFont(Cache.Font('Exo2Bold', 16))
        HmGui.Button("V")
        HmGui.SetStretch(0.0, 0.0)
        HmGui.SetAlign(1.0, 0.5)
        HmGui.PopStyle(1)
        HmGui.EndGroup()
        HmGui.SetStretch(0.965, 0.2)
        HmGui.SetAlign(0.5, 0.05)

        HmGui.Rect(1.0, 1.0, 0.7, 0.7, 0.7, 1.0);
        HmGui.SetStretch(0.995, 0.05)
        HmGui.SetAlign(0.5, 0.0)

        HmGui.BeginGroupX() -- Commodity Window Panel; commodity search textbox
        HmGui.BeginGroupStack()
        HmGui.Rect(1.0, 1.0, 0.2, 0.2, 0.2, 1.0);
        HmGui.SetStretch(0.98, 0.9)
        HmGui.SetAlign(0.5, 0.5)
        HmGui.TextEx(Cache.Font('Exo2', 16), 'Search Commodity', 1.0, 1.0, 1.0, 1.0)
        HmGui.SetStretch(0.7, 0.0)
        HmGui.SetAlign(0.2, 0.5)
        HmGui.EndGroup()
        HmGui.EndGroup()
        HmGui.SetStretch(0.965, 0.3)
        HmGui.SetAlign(0.5, 0.0)

        HmGui.Rect(1.0, 1.0, 0.7, 0.7, 0.7, 1.0);
        HmGui.SetStretch(0.995, 0.05)
        HmGui.SetAlign(0.5, 0.0)

        HmGui.BeginGroupStack()
        HmGui.Rect(1.0, 1.0, 0.15, 0.15, 0.15, 1.0);

        HmGui.BeginGroupStack() -- Commodity List Panel
        -- The BeginScroll() calculation is a hack to keep the length of the scrollable area
        -- inside the bound of its enclosing GroupStack. It glitches when the full game window
        -- is vertically resized too far (but not all the way to the full vertical extent).
        HmGui.BeginScroll(634 + (self.resY - 900))
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Agricultural Products', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Alloys', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Animals, Livestock', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Animals, Rare', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Art, Audio Recordings', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Art, Paintings', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Art, Sculptures', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Chemicals', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Clothing', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Computer Parts', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Computers', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Construction Materials', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Consumables', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Crystals, Gems, Non-Precious', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Crystals, Gems, Precious', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Crystals, Raw', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Currency, Coinage', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Currency, Paper', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Drones', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Electronics', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Elements', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Energy Systems', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Fertilizers', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Foods, Animal', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Foods, Gourmet', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Foods, Plants', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Fuels', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Furniture', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Gases', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Hydrocarbons, Raw', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Liquids', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Medical Equipment', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Metals', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Minerals', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Nitrogen Compounds', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Ores, Metal, Ferrous', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Ores, Metal, Non-Ferrous', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Ores, Non-Metal', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Ores, Radioactive', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Petrochemicals', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Pharmaceuticals', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Plants', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Plants, Rare', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Plastics', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Polymers', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Robots', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Ship Engines', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Ship Parts', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Ship Shields', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Software Components', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Software', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Special Alloys', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Spices', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Textiles', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Unit Prefabs', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Utility', 1.0, 1.0, 1.0, 1.0)
        HmGui.EndScroll(InputInstance)
        HmGui.EndGroup()
        HmGui.EndGroup()
        HmGui.EndGroup() -- end Commodity Window Panel
        HmGui.SetStretch(1.0, 1.0)
        HmGui.EndGroup() -- Order Book (offers/requests) Panel
        HmGui.SetStretch(1.0, 1.0)

        HmGui.BeginGroupStack() -- Place Order Panel
        -- Create a nice border
        HmGui.Rect(1.0, 1.0, 0.7, 0.7, 0.7, 1.0);
        HmGui.SetStretch(1.0, 1.0)
        HmGui.Rect(1.0, 1.0, 0.1, 0.1, 0.1, 1.0);
        HmGui.SetStretch(0.992, 0.997)
        HmGui.SetAlign(0.5, 0.5)

        HmGui.BeginGroupY() -- Commodity Window Panel
        HmGui.BeginGroupX() -- Commodity Window Panel; search filter
        HmGui.TextEx(Cache.Font('Exo2', 20), 'Search:', 1.0, 1.0, 1.0, 1.0)
        HmGui.SetStretch(0.5, 0.0)
        HmGui.SetAlign(0.5, 0.49)
        HmGui.BeginGroupStack()
        HmGui.Rect(1.0, 1.0, 0.2, 0.2, 0.2, 1.0);
        HmGui.SetStretch(1.0, 0.6)
        HmGui.SetAlign(0.5, 0.5)
        HmGui.TextEx(Cache.Font('Exo2', 16), 'Current Station', 1.0, 1.0, 1.0, 1.0)
        HmGui.SetStretch(0.7, 0.0)
        HmGui.SetAlign(0.2, 0.5)
        HmGui.EndGroup()
        HmGui.SetStretch(1.2, 1.0)
        HmGui.SetAlign(0.0, 0.5)
        HmGui.PushFont(Cache.Font('Exo2Bold', 16))
        HmGui.Button("V")
        HmGui.SetStretch(0.0, 0.0)
        HmGui.SetAlign(1.0, 0.5)
        HmGui.PopStyle(1)
        HmGui.EndGroup()
        HmGui.SetStretch(0.965, 0.2)
        HmGui.SetAlign(0.5, 0.05)

        HmGui.Rect(1.0, 1.0, 0.7, 0.7, 0.7, 1.0);
        HmGui.SetStretch(0.995, 0.05)
        HmGui.SetAlign(0.5, 0.0)

        HmGui.BeginGroupX() -- Commodity Window Panel; commodity search textbox
        HmGui.BeginGroupStack()
        HmGui.Rect(1.0, 1.0, 0.2, 0.2, 0.2, 1.0);
        HmGui.SetStretch(0.98, 0.9)
        HmGui.SetAlign(0.5, 0.5)
        HmGui.TextEx(Cache.Font('Exo2', 16), 'Search Commodity', 1.0, 1.0, 1.0, 1.0)
        HmGui.SetStretch(0.7, 0.0)
        HmGui.SetAlign(0.2, 0.5)
        HmGui.EndGroup()
        HmGui.EndGroup()
        HmGui.SetStretch(0.965, 0.3)
        HmGui.SetAlign(0.5, 0.0)

        HmGui.Rect(1.0, 1.0, 0.7, 0.7, 0.7, 1.0);
        HmGui.SetStretch(0.995, 0.05)
        HmGui.SetAlign(0.5, 0.0)

        HmGui.BeginGroupStack()
        HmGui.Rect(1.0, 1.0, 0.15, 0.15, 0.15, 1.0);

        HmGui.BeginGroupStack() -- Commodity List Panel
        -- The BeginScroll() calculation is a hack to keep the length of the scrollable area
        -- inside the bound of its enclosing GroupStack. It glitches when the full game window
        -- is vertically resized too far (but not all the way to the full vertical extent).
        HmGui.BeginScroll(634 + (self.resY - 900))
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Agricultural Products', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Alloys', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Animals, Livestock', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Animals, Rare', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Art, Audio Recordings', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Art, Paintings', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Art, Sculptures', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Chemicals', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Clothing', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Computer Parts', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Computers', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Construction Materials', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Consumables', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Crystals, Gems, Non-Precious', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Crystals, Gems, Precious', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Crystals, Raw', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Currency, Coinage', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Currency, Paper', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Drones', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Electronics', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Elements', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Energy Systems', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Fertilizers', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Foods, Animal', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Foods, Gourmet', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Foods, Plants', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Fuels', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Furniture', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Gases', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Hydrocarbons, Raw', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Liquids', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Medical Equipment', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Metals', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Minerals', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Nitrogen Compounds', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Ores, Metal, Ferrous', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Ores, Metal, Non-Ferrous', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Ores, Non-Metal', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Ores, Radioactive', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Petrochemicals', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Pharmaceuticals', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Plants', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Plants, Rare', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Plastics', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Polymers', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Robots', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Ship Engines', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Ship Parts', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Ship Shields', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Software Components', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Software', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Special Alloys', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Spices', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Textiles', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Unit Prefabs', 1.0, 1.0, 1.0, 1.0)
        HmGui.TextEx(Cache.Font('Exo2', 14), 'Utility', 1.0, 1.0, 1.0, 1.0)
        HmGui.EndScroll(InputInstance)
        HmGui.EndGroup()
        HmGui.EndGroup()
        HmGui.EndGroup() -- end Commodity Window Panel
        HmGui.SetStretch(1.0, 1.0)
        HmGui.EndGroup() -- end Place Order Panel
        HmGui.SetStretch(1.0, 1.0)

        HmGui.EndGroup()                    -- end Commodity Screen
        HmGui.SetStretch(1.0, 1.0)
    elseif station_options_market == 2 then -- Data/Intel Screen
        HmGui.BeginScroll(200)
        HmGui.Checkbox("Thing1", true)
        HmGui.Checkbox("Thing2", false)
        HmGui.EndScroll(InputInstance)                   -- end Data/Intel Screen
    elseif station_options_market == 3 then -- Exchange Orders Screen
        HmGui.BeginScroll(200)
        HmGui.Button("Parnellite")
        HmGui.Button("Glorboscite")
        HmGui.Button("Lonsdaleite")
        HmGui.Button("Ketracel White")
        HmGui.EndScroll(InputInstance)                   -- end Exchange Orders Screen
    elseif station_options_market == 4 then -- Blank Screen
        HmGui.BeginScroll(200)
        HmGui.EndScroll(InputInstance)                   -- end Blank Screen
    end
    HmGui.EndGroup()
    HmGui.SetStretch(1.0, 1.0)
    HmGui.EndGroup()
    HmGui.SetStretch(1.0, 1.0)
    HmGui.EndGroup()
    HmGui.SetStretch(1.0, 1.0)
    HmGui.EndGroup()
    HmGui.SetStretch(1.0, 1.0)
end

function Test:showGameCtrlInner()
    HmGui.BeginGroupY()
    HmGui.PushTextColor(1.0, 1.0, 1.0, 1.0)
    HmGui.PushFont(Cache.Font('Exo2Bold', 18))
    if HmGui.Button("Cancel") then
        drawExitMenu = false
    end
    HmGui.SetSpacing(8)
    if HmGui.Button("Quit") then
        Test:quit()
    end
    HmGui.EndGroup()
end

function Test:showCtrlMenu()
    HmGui.BeginWindow("Game Control", InputInstance)
    HmGui.TextEx(Cache.Font('Iceland', 20), 'Game Control', 0.3, 0.4, 0.5, 1.0)
    HmGui.SetAlign(0.5, 0.5)
    HmGui.SetSpacing(16)
    self:showGameCtrlInner()
    HmGui.EndWindow()
    HmGui.SetAlign(0.5, 0.5)
end

function Test:showMetrics()
    HmGui.BeginWindow("Metrics", InputInstance)
    HmGui.Text(format("fps: %.2f", 1.0 / self.dt))
    HmGui.EndWindow()
end

function Test:onUpdate(dt)
    Profiler.Begin('HmGui.Update')
    if InputInstance:isPressed(Button.KeyboardEscape) then
        drawExitMenu = not drawExitMenu
    end
    HmGui.Begin(self.resX, self.resY, InputInstance)
    if drawExitMenu then
        HmGui.BeginGroupStack()
        self:showCtrlMenu()
        HmGui.EndGroup()
    else
        --      self:showMetrics()
        self:showMenuDrop()
    end
    HmGui.End(InputInstance)
    Profiler.End()
end

function Test:onDraw()
    if useRenderer then
        self.renderer:start(self.resX, self.resY)
        Viewport.Push(0, 0, self.resX, self.resY, true)
        HmGui.Draw()
        Viewport.Pop()
        self.renderer:stop()
        self.renderer:present(0, 0, self.resX, self.resY)
    else
        HmGui.Draw()
    end
end

return Test
