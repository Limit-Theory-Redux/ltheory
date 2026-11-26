# Sensors

A ship's sensor system: what it makes of the objects around it. One system
with two modes:

- **Passive**: all round, always on, costs nothing. An object is heard (T1)
  once its strongest signal beats the noise, classed (T2) once it fills one
  resolution cell, and identified by sight (T3) once it fills five cells and
  its reflected light alone beats the noise.
- **Active**: the aimed scan reads T3 and then T4 from its echo (it takes the
  scan time per tier, heats the ship up and the target hears it); the
  all-round pulse finds whatever echoes back strongly enough, for a while, and
  every ship and station in earshot hears where you are.

Every object comes down to three facts: what it gives off (EM from its power
draw, and heat), what it reflects (light, and a share of a scan or pulse) and
its size. A sensor has a few dials: sensitivity, resolution, heat band, power,
scan time, and the ship's own power draw. Hull presets: fighter, frigate,
capital ship. Planets, asteroids, a star's glare, nebulae, debris fields and
radiation belts block or smear signals.

The code is in `script/Modules/Sensors/`. The sensor model only runs in the
Sensor Testbed so far; ships in the normal game have no sensors yet.

## Trying it: the Sensor Testbed

```sh
cargo run -p ltr -- Testbeds/Sensor
```

or pick **Sensor Testbed** under New Game. The player's ship carries a sensor
and a signature of its own (its power draw, louder and hotter while it scans).
Around it is one setup at a time, each showing one thing; B builds the next
one around the ship, R the same one again. The star system's star, planet and
moon have signatures, and belt rocks that come close get one too. The sensor
HUD (below) shows what the sensor makes of it all, with the setup's name and
what to try; the text in the top left shows the settings and the keys.

| Setup | What it shows |
|---|---|
| Reference scene | The sensor mock's scene: a station, a freighter, a fighter parked next to an asteroid, a second asteroid and a cold wreck. |
| Fighter, frigate, capital ship | The same ships (class I, II, III and a station) at 1, 3, 10, 30 and 100 km: how far each hull hears, classes and identifies them. |
| Hiding | Fighters in the open, next to a rock, behind one, cloaked, running cold, firing (in the open and in the star's glare), in front of the planet and behind it. |
| Passive against active | A nebula: listening only classes ships by their EM, a scan identifies them, a pulse finds the cold wreck. |
| Resolution and merging | Five fighters 60 m apart at 15 km: one blob for a fighter's sensor, five for a frigate's, five classed for a capital ship's. |
| Being heard | Five frigates with sensors of their own, 20 to 400 km away; the text says which hear you, while you listen, scan or pulse. |

| Key | Action |
|---|---|
| X (hold) | Scan the target |
| Z | Pulse |
| Tab | Next target (nearest first) |
| 1 2 3 | Hull: fighter, frigate, capital ship |
| 4 | Surroundings: open space, nebula, debris field, radiation belt |
| 5 | The target's activity, if it is a ship: idle, thrusting, firing, cloaked |
| 6 | Resolution rule: background in the cell, dims the target |
| 7 | Merge unresolved signals in one cell on or off |
| B | Next setup (rebuilt around the ship; clears what the sensor learned) |
| R | Start this setup again |

The setups are in `script/Config/Scenarios/SensorTestbedSetups.lua`: plain
tables of objects, each with its sensor values and its place (distance and
bearing from the ship, or on the line to the planet or the star). Objects
smaller than a pixel or so at their distance get no mesh, only a signature.

## The sensor HUD

`UIComponent.SensorHud` (`script/UI/HmGui/Components/Hud/SensorHud.lua`)
draws what a sensor shows, in two panels:

- **Passive**, at the bottom: a radar disc tilted away from you, your ship in
  the middle with its nose up, distance on a log scale (50 m at the centre,
  100 km at the rim). A contact sits at its bearing and distance: a soft blob
  at T1, a class icon at T2 (outline) and T3 (filled), amber at T4 (○ class I,
  Δ II, ▼ III, ⬢ IV station; circles for planets, a star, diamonds for
  asteroids). Rings show where the target would turn T1 to T4 in open space,
  and the ring round the disc bulges where signals come from. Beside it the
  contact list, and on its left your ship as others hear it: power draw,
  heat, and who heard your last pulse.
- **Active**, on the right: the scan window. The target's tier and echo,
  whether a scan reads it from here, the scan's progress, and what scans
  have read (T3 identity, T4 contents).

Feed it a `SensorPicture` every frame:

```lua
local hud = UIComponent.SensorHud { scanHint = "HOLD X TO SCAN" }
-- every frame
local body = ship:get(PhysicsComponents.RigidBody)
hud:update(SensorPicture.build(sensor, body:getForward(), body:getRight(), "FRIGATE"), dt)
```

A scenario shows its own HUD components by listing them in its `hud` field
(see `script/Config/Scenarios/__init__.lua`); the In_Game view draws them
with the game's HUD while the scenario runs.

## How it fits together

- `SensorModel.lua`: the maths, on plain tables. Units are metres, megawatts,
  seconds and degrees; positions are relative to the sensor. It holds no engine
  types, so it runs without a scene.
- `SensorText.lua`: distances, classes and contact lines as the HUD writes them.
- `SensorPicture.lua`: what a sensor shows, ready to draw: every object with
  its bearing and distance in the ship's frame, the contacts in list order,
  the target and its scan.
- `Components/SensorSignatureComponent.lua`: on anything that can be sensed:
  kind, size (the entity's bounding radius unless given), power draw,
  activity, and what a T3 and a T4 read show.
- `Components/SensorComponent.lua`: on a ship: the dials and the sensor's
  state (scan target and progress, reads, pulse echoes, its own heat), and the
  last readings.
- `Systems/SensorSystem.lua`: `SensorSystem:update(dt)` places every signature
  relative to every sensor (in real metres), fires requested pulses, evaluates
  the model and moves each sensor's state on.

Giving an entity a signature and a ship a sensor:

```lua
local SensorComponents = require("Modules.Sensors.Components")
local SensorModel = require("Modules.Sensors.SensorModel")

station:add(SensorComponents.SensorSignature { kind = "station", label = "Station", mw = 500 })
ship:add(SensorComponents.Sensor(SensorModel.hulls.frigate))

-- each simulation step
SensorSystem:update(dt)
local sensor = ship:get(SensorComponents.Sensor)
for _, o in ipairs(sensor:getObjects()) do
    local reading = sensor:getReadings()[o] -- tier, why, count
end
```

## Tuning

The constants (`SensorModel.constants`), object kinds and hull presets are at
the top of `SensorModel.lua`. They are tuned so a frigate against a 20 m ship
drawing 50 MW, in open space, hears it at 50 km (T1), classes it at 10 km (T2),
sees who it is at 2 km (T3) and reads it with a scan at 200 m (T4); the other
values are placeholders.

`SensorModelTest` checks the model against those distances and against
reference readings of a fixed scene for every hull and surrounding:

```sh
cargo run -p ltr -- SensorModelTest
```

It exits 0 and logs `SensorModelTest: passed`. After changing a constant on
purpose, update the reference values in the test with it.

`SensorSetupsTest` goes through every setup of the testbed with every hull and
checks the readings that make each one's point (and a pulse, a scan and who
hears you):

```sh
cargo run -p ltr -- SensorSetupsTest
```

It exits 0 and logs `SensorSetupsTest: passed`. When a setup or the model
changes on purpose, update its expectations at the top of the test.
