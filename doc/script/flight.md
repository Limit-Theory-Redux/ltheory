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
- `AutoPilotSystem` flies the ship through a `NavWorld` with one agent and sensing on for every collision group, so it steers around the collidable bodies near it (the target body too: the stand-off point lies outside its hull). While the travel drive runs the ship has no lateral thrust and turns at a tenth of its torque, so avoidance acts only in normal flight; a body in the way of a drive flight isn't avoided. It tracks the target (with a lead for a moving one), sets a goal on the line from the ship to the target at a stand-off distance from the target's centre, steps the `NavWorld`, and copies the six axes into the ship's component. It arrives, and disengages, when the `NavWorld` reports the goal reached; the arrival radius is a share of the gap between the stand-off distance and the target's radius, so a ship that coasts in after disengaging stays outside the hull. It requests the drive when the target is more than 100 stand-off distances away, and sets `driveSpeedCap` from a braking law: the wanted speed is the distance to the stand-off point divided by a braking time, so the cap is `distance / (brakingTime · maxSpeed)`, limited by the zone's cap. The braking time is 4 times the sum of the drive multiplier's lag (`1 / rampDownSpeed`) and the velocity's lag (`1 / linearDrag`); at that margin the loop from distance to speed has real roots, so the ship decays onto the stand-off point without overshooting, and a margin of 2 oscillates. Below a drive multiplier of 2 the drive is released and the `NavWorld` flies the rest at normal speed. The ETA comes from the same law: a cruise at the zone's cap while the law wants more, then an exponential decay, then the flight at `maxSpeed`. Because the drive multiplier and the velocity lag the falling cap, the distance decays faster than `1 / brakingTime`; the ETA uses the rate that solves `r = 1 / (T · (1 - r·τ₁) · (1 - r·τ₂))`. The ETA uses the zone cap at the ship's position and leaves out the drive's charging and ramp-up time, so it runs short. While the autopilot is engaged the ship's body must stay in the physics world, and a moving target's velocity is not fed to the `NavWorld`.

## Actuation

A frame writes control input at `PreSim` and actuates it at `Sim`. The writers (`PlayerController:update(dt)`, and through it `AutoPilotSystem`, which runs after the manual axes are written and replaces them while it is active) run in the app's `PreSim` handler. The app's `Sim` handler then moves the bodies (orbits, asteroid follow), applies the gravity wells' carry force (`GravityWellSystem:update(dt, substeps)`), runs `TravelDriveSystem:update(dt)`, then `ShipFlightSystem:update()`, then the physics step. The `Sim` handler calls them in this order because `EventBus:subscribe` gives no ordering inside a stage, and the bodies must have moved before the ships are actuated. Both actuation systems process every ship that has the components they need.

```mermaid
flowchart TD
    subgraph PreSim
        PC["PlayerController:update<br/>writes the manual axes"] --> AP["AutoPilotSystem<br/>replaces them while it is active"]
        AP --> CI["ControlInputComponent"]
    end
    subgraph Sim
        OR["Orbits and asteroid follow:<br/>move the bodies"] --> GW["GravityWellSystem:update<br/>carry force"]
        GW --> TD["TravelDriveSystem:update"]
        TD --> SF["ShipFlightSystem:update"]
        SF --> PH["Physics step"]
    end
    CI -.-> TD
    CI -.-> SF
```

- `ShipFlightSystem` clamps the axes and applies forces and torques. While the travel drive is active it ignores thrust and boost, because the drive provides the forward force, and applies a tenth of the torque.
- `TravelDriveSystem` runs the drive's states: idle, charging, active, decelerating. An engage request starts charging an idle drive; after `chargeRequired` seconds the drive is active. A disengage request moves a charging or active drive to decelerating. While active, the multiplier approaches `min(zone limit, driveSpeedCap)`: upward linearly at `rampUpSpeed`, downward exponentially with time constant `1 / rampDownSpeed`. All ramps are functions of elapsed time, so the result does not depend on the frame rate.

```mermaid
stateDiagram-v2
    [*] --> Idle
    Idle --> Charging: engage request
    Charging --> Active: charged for chargeRequired seconds
    Charging --> Decelerating: disengage request
    Active --> Decelerating: disengage request
    Decelerating --> Idle: multiplier falls to the idle threshold
```

The zone limit is `GravityWellSystem:getMaxDriveSpeedAt(position)`, evaluated at the position of each ship, so every ship has its own limit. See [Zones and gravity wells](zones.md).
