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
close get a signature too. The sensor HUD (below) shows what the sensor makes
of it all; the text in the top left shows the settings and the keys.

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

## The sensor HUD

`UIComponent.SensorHud` (`script/UI/HmGui/Components/Hud/SensorHud.lua`)
draws what a sensor shows with shapes, and words only where nothing else can
say it: a target's name and distance, and what a full scan read.

- **The radar**, at the bottom centre: a disc tilted away from you, your ship
  in the middle with its nose up, distance on a log scale (50 m at the centre,
  100 km at the rim). A contact sits at its bearing and distance.
- **The target panel**, at the bottom right: a hologram circle that fills in
  as the sensor learns more about the target. Static without a contact, a blob
  at T1, its icon at T2, a wireframe of it at T3 (a stand-in for the
  hologram) with its power draw (bars), what it's doing (an icon) and its
  shields (how strongly the ring glows), brighter at T4 with what the full
  scan read beside it. A scan's progress runs round the rim, and its echo
  fills a meter under the panel with a notch where it reads T3 and one for T4;
  chevrons mean the echo is too weak to read more from here.

How a contact looks:

| | |
|---|---|
| Shape: what it is | ships and stations by class: ○ I, Δ II, ▼ III, ⬢ IV; an asteroid a dot; a planet or moon ◎; a star ✦; a wreck its class icon, broken |
| Fill: how much is known | T1 a soft blob (no shape yet), T2 an outline, T3 filled, T4 filled with a bright core |
| Colour: whose side | pale until T3, then red for hostile |

Amber means "look here" and nothing else: your own scan and pulse, your
thermal bloom. On the radar, rings show where the target would turn T1 to T4
in open space, each marked with that tier's icon, and the ring round the disc
bulges where signals come from. A dashed ring shows how far your power draw
is heard (an amber one further out while you scan or just after a pulse), an
arc round your ship your heat, and ripples over a contact that it heard your
last pulse.

Feed it a `SensorPicture` every frame:

```lua
local hud = UIComponent.SensorHud {}
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
  activity, and what a T3 read (name, shields, standing) and a T4 read show.
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
