# Ship flight control

Every ship flies from one data path: whoever steers it writes a `ControlInputComponent` on the ship, and the actuation systems turn that component into forces. The player, an autopilot and AI are all writers; no actuation system reads an input device.

## Control input

`ControlInputComponent` (`script/Modules/Constructs/Components/`) holds:

| Field | Range | Meaning |
|---|---|---|
| `thrustForward`, `thrustRight`, `thrustUp` | `ControlAxis`, [-1, 1] | Fraction of the ship's profile thrust along its axes |
| `pitch`, `yaw`, `roll` | `ControlAxis`, [-1, 1] | Fraction of the ship's profile torque about its axes |
| `boost` | [0, 1] | Forward thrust is multiplied by `1 + 2·boost` |
| `driveRequest` | `Enums.TravelDriveRequest` | `Engage` or `Disengage` the travel drive; `None` when nothing is pending |
| `driveSpeedCap` | multiplier ≥ 1, or nil | Highest drive multiplier the ship asks for; nil is no cap |

Axes are levels: the writer sets them every frame (`clearAxes()` zeroes them). The drive request is an edge: `TravelDriveSystem` clears it once it has applied it. A writer that has no manual input this frame clears the axes, so a ship never keeps flying on stale input.

## Flight profiles

A ship's flight limits are data, not configuration: a `FlightProfile` (`Shared/Types/FlightProfile.lua`) per ship class, registered in `FlightProfileRegistry` by an `Enums.FlightProfile` id from `Shared/Definitions/FlightProfileDefs.lua`. The ship holds its profile in a `FlightProfileComponent`; the flight systems read the limits from it and fail loudly on a ship that flies without one.

| Field | Unit | Meaning |
|---|---|---|
| `mass` | kg | Body mass |
| `linearDrag`, `angularDrag` | 1/s | Damping of velocity and spin |
| `thrustForward`, `thrustRight`, `thrustUp` | force, game units | Largest force along each ship axis; these are the effective limits, with no further gain applied |
| `torquePitch`, `torqueYaw`, `torqueRoll` | torque, game units | Largest torque about each ship axis |
| `maxSpeed` | game units/s | Speed that full forward thrust settles at |

Definitions are authored in real metres and compressed to game units once. The forward thrust is derived as `maxSpeed · mass · linearDrag`, so full thrust against the drag settles at `maxSpeed`. Boost and the steering torque during a travel drive are flight-model gains in `ShipFlightSystem`, not profile data.

A ship gets its profile by passing `flightProfile` in the config of `ShipGenerator` (and so of `ConstructManager:createShip`): the generator adds the component and applies the profile to the rigid body through `FlightProfileHelper.applyToRigidBody`. That sets the mass, the drag, no friction, no sleeping and the soft-CCD distance (one frame of travel at `maxSpeed`). Hull size stays in `Config.game.shipHulls`.

## Writers

- `PlayerController:update(dt)` reads `ShipActions` and writes the player ship's component. Chase mode steers by the cursor's position on screen, first person by cursor movement (an aim target accumulates and decays back to centre), and an active drive by cursor movement in either mode. Axes are cleared while input is blocked, the camera is not a piloting one, or the autopilot is flying. The travel-drive key engages an idle drive and stops a running one; firing stops it.
- `AutoPilotSystem` requests the drive when the target is far away, sets `driveSpeedCap` from the distance, and disengages on the final approach.

## Actuation

An app's simulation step runs the writers, then `TravelDriveSystem:update(dt)`, then `ShipFlightSystem:update()`, then the physics step. Both systems process every ship that has the components they need.

- `ShipFlightSystem` clamps the axes and applies forces and torques. While the travel drive is active it ignores thrust and boost, because the drive provides the forward force, and applies a tenth of the torque.
- `TravelDriveSystem` runs the drive's states: idle, charging, active, decelerating. An engage request starts charging an idle drive; after `chargeRequired` seconds the drive is active. A disengage request moves a charging or active drive to decelerating. While active, the multiplier approaches `min(zone limit, driveSpeedCap)`: upward linearly at `rampUpSpeed`, downward exponentially with time constant `1 / rampDownSpeed`. All ramps are functions of elapsed time, so the result does not depend on the frame rate.

The zone limit comes from `GravityWellSystem:getMaxDriveSpeed()`, which describes the zone of the player's ship only.
