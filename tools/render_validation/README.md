# Render validation

Each script launches `ltr` with one `script/States/App/Rendering/*` scene on a
given backend, waits for it to exit, and checks the probe values the scene
logs (pixel readbacks) against expected thresholds.

```sh
python tools/render_validation/indexed_mrt_supervisor.py gl
python tools/render_validation/indexed_mrt_supervisor.py wgpu   # sets LTHEORY_WGPU=1
```

Build `ltr` first. Logs go to `target/render_validation/`. Override the binary
with `LTHEORY_EXE` or `CARGO_TARGET_DIR`. Exit code 0 means the scene passed.

## Capture baseline / regression diff

Real scenes (PlanetTest, Benchmark, SolarSystemPlayable, MoonTest, plus the variants PlanetTestRing = PlanetTest with seed 27, which rolls a planet ring, and the WeaponSystem testbed with deferred point lights) are captured
deterministically via `LTHEORY_CAPTURE=<out.png>` (+ `LTHEORY_CAPTURE_FRAME=<n>`,
default 120): the engine uses a fixed 60 Hz delta time, a 1280x720 window, saves
the backbuffer after frame n and exits, logging a `CAPTURE ...` stats line.

```sh
python tools/render_validation/capture.py gl [scene...]   # -> target/render_validation/captures/gl/
python tools/render_validation/capture.py --baseline      # copy gl captures into baseline/
python tools/render_validation/compare.py gl              # RMSE / max / % pixels over threshold, heatmaps in captures/gl/diff/
python tools/render_validation/run_all.py gl              # supervisors + capture + compare
```

`compare.py` needs Pillow. Baseline `.json` stats (fps, draw calls) are informational.
