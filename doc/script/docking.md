# Docking

The `Docking` module (`script/Modules/Docking`) docks ships at stations: a ship takes a port, flies to
it, glides in, stays with the station while docked, and glides out again.

- **Ports.** A station's `DockingPortsComponent` (every `SpaceStationEntity` has one) holds its ports,
  spread around its equator: each has the place a docked ship sits (`offsetRadii`, just outside the
  hull) and an approach point further out (`approachRadii`), both in station radii along the station's
  own axes. A port is held by one ship from its request until it has left.
- **Phases.** A docking ship has a `DockingComponent` with its station, port and phase
  (`Enums.DockingPhase`):
  1. `Approaching`: a navigation `MoveTo` to the station with a `StandOff` of `approachGapM` (1.5 km)
     off its hull. A loose stand-off is what navigation arrives at reliably; the precise part is the
     glide's.
  2. `Entering`: the order is done; `DockingSystem` claims the ship's controls (`Nav.claim`), makes its
     body kinematic and glides it along a traffic path: out or in to the approach ring (the ports'
     approach points' distance), round the ring to the port's side, and into the port, turning its
     nose out of the port.
  3. `Docked`: the ship is held in the port as the station moves and turns.
  4. `Leaving`: the ship glides back to the approach point; then its body is dynamic again, its
     controls go back to its orders (`Nav.unclaim`), the port is freed and the component removed.
  5. `Failed`: the flight to the approach point failed or the station is gone. `release` ends it.
- **Use.** `DockingSystem:requestDock(ship, station)` (false when no port is free),
  `undock(ship)`, `phaseOf(ship)`, `isDocked(ship)`, `release(ship)`, `countPorts(station)`.
- **Timing.** The system runs at `PreSim`, before the frame's physics step. A glide takes its path's
  length at `glideSpeed`, within `minGlideS` and `maxGlideS`. A held or gliding ship's
  kinematic target leads the station by its velocity over the step, so it keeps its place on a
  moving station.
