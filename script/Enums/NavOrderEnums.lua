---@alias NavOrderStatusId integer An `Enums.NavOrderStatus` value
---@alias NavOrderFailureId integer An `Enums.NavOrderFailure` value
---@alias NavOrderKindId integer An `Enums.NavOrderKind` value
---@alias NavOrderPriorityId integer An `Enums.NavOrderPriority` value

-- Where a navigation order is in its life. A queued order waits behind the ones before it;
-- `Planning` is the head order with its path still being planned; `Suspended` is an order that
-- had begun and was displaced by one of higher priority, and resumes when it is the head again;
-- `Done`, `Failed` and `Cancelled` are the three ends, and an ended order never changes again.
Enums.NavOrderStatus = {
    Queued = 1,
    Planning = 2,
    Active = 3,
    Done = 4,
    Failed = 5,
    Cancelled = 6,
    Suspended = 7,
}

-- How urgent an order is: a higher level runs before a lower one, and displaces it.
Enums.NavOrderPriority = {
    Routine = 1,
    Urgent = 2,
    Combat = 3,
}

-- Why an order failed.
Enums.NavOrderFailure = {
    TargetLost = 1, -- the entity the order flies to is gone
    ShipLost = 2,   -- the ship is gone
}

Enums.NavOrderKind = {
    MoveTo = 1,
    Patrol = 2,
    Hold = 3,
    Orbit = 4,
    Follow = 5,
}
