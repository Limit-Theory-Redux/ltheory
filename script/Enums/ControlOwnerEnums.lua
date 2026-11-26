---@alias ControlOwnerId integer An `Enums.ControlOwner` value

-- Who writes a ship's control input. A higher value wins: manual input over a direct-control claim,
-- a claim over navigation orders.
---@enum ControlOwner
Enums.ControlOwner = {
    None   = 0, -- nobody: the ship drifts
    Orders = 1, -- the navigation order system
    Claim  = 2, -- a direct-control claim: the claimer writes the input itself
    Manual = 3, -- the player's input
}
