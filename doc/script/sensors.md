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

or pick **Sensor Testbed** under New Game. The player's ship carries a
frigate's sensor; around it are a station (12 km), a freighter (4 km), a
fighter parked next to an asteroid (11 km), a nearer asteroid and a cold wreck
(400 m), and the star system's star, planet and moon. Belt rocks that come
close get a signature too. The text in the top left lists the contacts, the
target and the scan.

| Key | Action |
|---|---|
| X (hold) | Scan the target |
| Z | Pulse |
| Tab | Next target (nearest first) |
| 1 2 3 | Hull: fighter, frigate, capital ship |
| 4 | Surroundings: open space, nebula, debris field, radiation belt |
| 5 | The fighter's activity: idle, thrusting, firing, cloaked |
| 6 | Resolution rule: background in the cell, dims the target |
| 7 | Merge unresolved signals in one cell on or off |
| B | Put everything back around the ship; clear what the sensor learned |

Things to try: with the frigate the fighter hides in the asteroid's
background, with the capital ship it shows; in a nebula the wreck only shows
after a pulse; holding X on the freighter with the capital ship reads its T3.

## How it fits together

- `SensorModel.lua`: the maths, on plain tables. Units are metres, megawatts,
  seconds and degrees; positions are relative to the sensor. It holds no engine
  types, so it runs without a scene.
- `SensorText.lua`: distances, classes and contact lines as the HUD writes them.
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
