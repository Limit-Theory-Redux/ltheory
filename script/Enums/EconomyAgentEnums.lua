---@alias EconomyAgentRole integer An `Enums.EconomyAgentRole` value
---@alias EconomyAgentState integer An `Enums.EconomyAgentState` value

-- What an economic agent ship does for a living: a trader buys where an item is cheap and sells
-- where it is dear; a miner takes ore from an asteroid and sells it.
Enums.EconomyAgentRole = {
    Trader = 1,
    Miner = 2,
}

-- Where an agent is in its job. `Idle` looks for a job; `Waiting` sits until a time, docked or not,
-- and then looks again.
Enums.EconomyAgentState = {
    Idle = 1,
    Waiting = 2,
    DockingToBuy = 3,  -- flying to and docking at the station it buys from
    Buying = 4,        -- docked there, trading before it leaves
    FlyingToMine = 5,  -- flying to the asteroid it mines
    Mining = 6,        -- at the asteroid, filling its hold
    DockingToSell = 7, -- flying to and docking at the station it sells to
    Selling = 8,       -- docked there, trading before it leaves
    Undocking = 9,     -- leaving a port
}
