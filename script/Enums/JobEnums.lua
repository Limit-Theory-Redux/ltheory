-- Enumerations for job states
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
