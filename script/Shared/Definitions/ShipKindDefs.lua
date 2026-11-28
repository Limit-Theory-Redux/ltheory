---The kinds of AI ship there are, by what they carry: the size of their hold, a mining laser,
---weapons. A ship's equipment decides which jobs its AI can choose from (a hold: trade; a mining
---laser: mining; weapons: security and raids); the AI chooses among them as it goes.
---`SpawnerSystem:spawnAgent` builds one; `credits` is what an unowned one starts with. `crew` is the
---roles aboard (`CrewRoleDefs`), the commanding one first: every ship is manned, and its commander
---decides what it does. A lost ship leaves a wreck (`WreckSystem`) with its `scrap` and cargo.

---@class ShipKindDefinition
---@field name string
---@field capacity number The hold's mass capacity
---@field miningLaser boolean
---@field armed boolean
---@field credits number
---@field crew string[] Role ids, one person each
---@field scrap number What its wreck yields to salvage, in units of scrap
---@field wreckIntegrity number The damage its wreck takes before it breaks up

---@type table<string, ShipKindDefinition>
local ShipKinds = {
    Hauler = { name = "Hauler", capacity = 600, miningLaser = false, armed = false, credits = 8000, crew = { "Captain", "Mate", "Hand" }, scrap = 60, wreckIntegrity = 300 },
    Miner = { name = "Miner", capacity = 480, miningLaser = true, armed = false, credits = 500, crew = { "Captain", "Hand" }, scrap = 45, wreckIntegrity = 250 },
    Fighter = { name = "Fighter", capacity = 120, miningLaser = false, armed = true, credits = 2000, crew = { "Captain" }, scrap = 25, wreckIntegrity = 150 },
}

return ShipKinds
