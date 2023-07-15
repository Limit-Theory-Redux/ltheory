Enums.Jobs = {
  Mining = 1,
  Transport = 2
}

Enums.JobNames = {
  [1] = "Mining",
  [2] = "Transport",
}

Enums.BlackMarketJobs = {
  Marauding = 1
}

Enums.BlackMarketJobNames = {
  [1] = "Marauding"
}

Enums.JobStateTransport = {
    None             = 0,
    DockingAtSrc     = 1,
    BuyingItems      = 2,
    UndockingFromSrc = 3,
    DockingAtDst     = 4,
    SellingItems     = 5,
    UndockingFromDst = 6,
    JobFinished      = 7,
}

Enums.JobStateMine = {
    None             = 0,
    MovingToAsteroid = 1,
    MiningAsteroid   = 2,
    DockingAtDst     = 3,
    SellingItems     = 4,
    UndockingFromDst = 5,
    JobFinished      = 6,
}

Enums.JobStateMarauding = {
  None             = 0,
  SelectArea       = 1,
  MovingToArea     = 2,
  Marauding        = 3,
  FindBlackMarket  = 4,
  DockingAtStation = 5,
  SellingItems     = 6,
  Undocking        = 7,
  JobFinished      = 8,
}
