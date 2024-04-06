Config.econ = {
    pStartCredits          = 10000,   -- player starting credits
    eStartCredits          = 1000000, -- NPC player starting credits

    eInventory             = 100,     -- starting number of inventory slots

    jobIterations          = 4000,    -- how many randomly-chosen jobs an asset will consider before picking

    inputBacklog           = 1,       -- multiplier of number of units a factory can bid for on each input

    pickupDistWeightMine   = 1.0,     -- importance of pickup distance for a Mine job (smaller = more important)
    pickupDistWeightTran   = 3.0,     -- importance of pickup distance for a Transport job (smaller = more important)
    markup                 = 1.2,     -- change to base value when calculating ask price for selling an item
    markdown               = 0.8,     -- change to base value when calculating bid price for buying an item

    lowAttentionUpdateRate = 5,
}
