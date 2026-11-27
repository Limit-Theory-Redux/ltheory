---The kinds of AI ship there are, by what they carry: the size of their hold, a mining laser,
---weapons. A ship's equipment decides which jobs its AI can choose from (a hold: trade; a mining
---laser: mining; weapons: security and raids); the AI chooses among them as it goes.
---`SpawnerSystem:spawnAgent` builds one; `credits` is what an unowned one starts with.

---@class ShipKindDefinition
---@field name string
---@field capacity number The hold's mass capacity
---@field miningLaser boolean
---@field armed boolean
---@field credits number

---@type table<string, ShipKindDefinition>
local ShipKinds = {
    Hauler = { name = "Hauler", capacity = 600, miningLaser = false, armed = false, credits = 8000 },
    Miner = { name = "Miner", capacity = 480, miningLaser = true, armed = false, credits = 500 },
    Fighter = { name = "Fighter", capacity = 120, miningLaser = false, armed = true, credits = 2000 },
}

return ShipKinds
