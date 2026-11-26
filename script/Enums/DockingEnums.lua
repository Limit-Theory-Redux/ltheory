---@alias DockingPhase integer An `Enums.DockingPhase` value

-- Where a ship is in docking at a station: flying to the port's approach point, gliding into the
-- port, held in it, gliding back out. `Failed` is a docking whose flight to the approach point
-- failed or whose station is gone; the ship flies on its own again.
Enums.DockingPhase = {
    Approaching = 1,
    Entering = 2,
    Docked = 3,
    Leaving = 4,
    Failed = 5,
}
