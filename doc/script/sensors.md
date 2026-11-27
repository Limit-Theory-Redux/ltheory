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
and a signature of its own (its power draw, louder and hotter while it scans),
and takes its hull's size: 8 m for the fighter, 30 m for the frigate, 150 m
for the capital ship.
Around it is one setup at a time, each showing one thing; B builds the next
one around the ship, R the same one again. The star system's star, planet and
moon have signatures, and belt rocks that come close get one too. The sensor
HUD (below) shows what the sensor makes of it all; the text in the top left
shows the setup's name, what to try, the settings and the keys.

| Setup | What it shows |
|---|---|
| Reference scene | The sensor mock's scene: a station, a freighter, a fighter parked next to an asteroid, a second asteroid and a cold wreck. |
| Fighter, frigate, capital ship | The same ships (class I, II, III and a station) at 1, 3, 10, 30 and 100 km: how far each hull hears, classes and identifies them. |
| Hiding | Fighters in the open, next to a rock, behind one, cloaked, running cold, firing (in the open and in the star's glare), in front of the planet and behind it. The tuning panel says why one is no contact. |
| Passive against active | A nebula: listening only classes ships by their EM, a scan identifies them, a pulse finds the cold wreck. |
| Resolution and merging | Five fighters 60 m apart at 15 km: one blob for a fighter's sensor, five for a frigate's, five classed for a capital ship's. |
| Being heard | Five frigates with sensors of their own, 20 to 400 km away; the text says which hear you, while you listen, scan or pulse. |

| Key | Action |
|---|---|
| X (hold) | Scan the target |
| Z | Pulse |
| Tab | Next target (nearest first) |
| ` (above Tab) | Focus the radar (below), or put it back |
| 1 2 3 | Hull: fighter, frigate, capital ship |
| 4 | Surroundings: open space, nebula, debris field, radiation belt |
| 5 | The target's activity, if it is a ship: idle, thrusting, firing, cloaked |
| 6 | Resolution rule: background in the cell, dims the target |
| 7 | Merge unresolved signals in one cell on or off |
| B | Next setup (rebuilt around the ship; clears what the sensor learned) |
| R | Start this setup again |
| 8 | Tuning panel (below) on or off |
| Up, Down | Tuning panel: pick a dial |
| Left, Right | Tuning panel: change the dial (hold to keep changing it) |
| Backspace | Tuning panel: put the hull's dial values back |

The setups are in `script/Config/Scenarios/SensorTestbedSetups.lua`: plain
tables of objects, each with its sensor values and its place (distance and
bearing from the ship, or on the line to the planet or the star). Objects
smaller than a pixel or so at their distance get no mesh, only a signature,
until the ship comes close enough to see them.

## The sensor HUD

`UIComponent.SensorHud` (`script/UI/HmGui/Components/Hud/SensorHud.lua`)
draws what a sensor shows with shapes, and words only where nothing else can
say it: a target's name and distance, and what a full scan read.

- **The radar**, at the bottom centre: a small disc tilted well away from
  you, your ship in the middle with its nose up, distance on a log scale (50 m
  at the centre, 100 km at the rim), and a faint wedge for about what's in
  your view. A contact's foot sits on the disc at its bearing, with a stick up
  to it when it's above your plane and a dashed one down when it's below. On
  the radar a contact is a dot: a ship's as big as its class, a station a ring
  round a dot, a planet a ring, a yellow blob while it's only heard.
  Focused (`hud:toggleFocus()`, or `hud:setFocused(on)`), the radar grows
  and tilts toward top-down, with the middle ring's distance beside it, and
  its contacts show as their icons. A small button right of the rim shows the
  key that does it (`focusKey` when the HUD is made), lit while it's focused.
- **Your view**: each contact's icon where it is on the screen (except
  planets and stars, and anything already big on screen), and the target in
  brackets with its icon, its distance under them and, once it's identified,
  its name over them. Off screen, the screen's edge glows toward contacts that
  are unknown (yellow, softly) or hostile (red, with a quicker pulse), a
  chevron with its icon points to every classed contact, and a white arrow
  with its distance points to the target.
- **The target panel**, at the bottom right: a hologram circle that fills in
  as the sensor learns more about the target. Static without a contact, a
  yellow blob at T1, its icon at T2, a wireframe of it at T3 (a stand-in for
  the hologram) with its power draw (bars), what it's doing (an icon) and its
  shields (how strongly the ring glows), brighter at T4 with what the full
  scan read beside it. A scan's progress runs round the rim, and its echo
  fills a meter under the panel with a notch where it reads T3 and one for T4;
  chevrons mean the echo is too weak to read more from here.

How a contact looks:

| | |
|---|---|
| Icon: what it is | the game map icons (`res/tex2d/map-icon`): ships by class (I a fighter, II a corvette, III a capital ship), a station, an asteroid, a planet, a star; a wreck its class's icon |
| Colour: whose side | yellow while it's only heard (T1: unknown, possibly a threat); from T2 on, when it's classed, its standing: green owned, cyan friendly, grey neutral and wrecks, red hostile |

White is yours: your scan and pulse, the ring round your target on the radar,
the arrow toward it, your thermal bloom. The HUD itself is blue. On the radar,
dashed rings show where the target would turn T2, T3 and T4 in open space (T1
is as far as the signal ring hears), and the ring round the disc bulges where
signals come from. A dashed ring shows how far your power draw is heard (a
white one further out while you scan or just after a pulse), an arc round your
ship your heat, and ripples over a contact that it heard your last pulse.

The icons are made from the game map icons' SVG by `tools/ui-icons` (see its
README); the HUD draws them as tinted images (`Gui:setImageColor`).

### What the sensor remembers

A sensor keeps the best tier it has had of every object, whether it saw it or
a scan read it (`SensorState.memory`). What it knows stays when the object
fades: a fighter it identified stays a red fighter on the radar and in the
target panel, even once it is only heard. While it senses less than it knows,
the details are as last seen: its dot and icon fade, an arc round the dot
closes over two minutes as the knowledge ages, and the target panel greys the
shields, power and activity. Once the object is not heard at all, a ghost (a
dashed ring) stays where it was last seen, fading for a ship, and the target
panel shows the memory grey with scan lines. Objects it has classed don't
merge into anonymous blobs.

Feed it a `SensorPicture` every frame:

```lua
local hud = UIComponent.SensorHud {}
-- every frame
local body = ship:get(PhysicsComponents.RigidBody)
hud:update(SensorPicture.build(sensor, body:getForward(), body:getRight(), body:getUp()), dt)
```

A scenario shows its own HUD components by listing them in its `hud` field
(see `script/Config/Scenarios/__init__.lua`); the In_Game view draws them
with the game's HUD while the scenario runs.

## How it fits together

- `SensorModel.lua`: the maths, on plain tables. Units are metres, megawatts,
  seconds and degrees; positions are relative to the sensor. It holds no engine
  types, so it runs without a scene.
- `SensorText.lua`: distances, power draws and SNRs as the HUD and the tuning
  panel write them.
- `SensorTuning.lua`: the dials the tuning panel changes, with their ranges
  and steps.
- `SensorPicture.lua`: what a sensor shows, ready to draw: every object with
  its bearing, height and distance in the ship's frame and what the sensor
  remembers of it (lost ones where they were last seen), the target and its
  scan, and how far the ship and its pulse are heard.
- `Components/SensorSignatureComponent.lua`: on anything that can be sensed:
  kind, size (the entity's bounding radius unless given), power draw,
  activity, standing (the HUD shows it from T2), and what a T3 read (name,
  shields) and a T4 read show.
- `Components/SensorComponent.lua`: on a ship: the dials and the sensor's
  state (scan target and progress, reads, pulse echoes, its own heat), and the
  last readings.
- `Systems/SensorSystem.lua`: `SensorSystem:update(dt)` places every signature
  relative to every sensor (in real metres), fires requested pulses, evaluates
  the model, moves each sensor's state on, and keeps where each contact was
  last seen.

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

In the testbed, 8 shows the tuning panel
(`script/UI/HmGui/Components/Hud/SensorTuningPanel.lua`), at the bottom left.
It is a developer's view of the numbers behind the target's reading:

- EM, heat and light: what reaches the sensor of each, as an SNR against the
  "heard" line (SNR 1), the strongest one bright.
- Noise: how much the background in the target's cell raises it, and what the
  background is (with the "dim" rule: how much of the signal counts).
- Cells: how many resolution cells the target fills, against the T2 and T3
  marks.
- Scan and pulse: the echo a scan would get, against the T3 and T4 marks, and
  a pulse's echo against "found" (SNR 1).
- The tiers the target has reached, and why.

Under them are the dials. Up and Down pick one, Left and Right change it,
Backspace puts the hull's values back; each slider has ticks where the
fighter, frigate and capital ship have it. Their ranges and steps are in
`script/Modules/Sensors/SensorTuning.lua`. Changes last until the next hull
key or setup.

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
