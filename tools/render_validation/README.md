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

Real scenes (PlanetTest, Benchmark, BenchmarkPhases, Benchmark1080, SolarSystemPlayable, MoonTest, plus the variants PlanetTestRing = PlanetTest with seed 27, which rolls a planet ring, and the WeaponSystem testbed with deferred point lights) are captured
deterministically via `LTHEORY_CAPTURE=<out.png>` (+ `LTHEORY_CAPTURE_FRAME=<n>`,
default 120): the engine uses a fixed 60 Hz delta time, a 1280x720 window, saves
the backbuffer after frame n and exits, logging a `CAPTURE ...` stats line.

More capture options (all only under `LTHEORY_CAPTURE`): `LTHEORY_CAPTURE_SIZE=WxH` (window size; re-requested
during the first frames, since a size set before the OS window exists is lost),
`LTHEORY_CAPTURE_EXTRA_FRAMES=a,b,...` (also save `<out>_f<a>.png`, ... in the same run; each logs a
`CAPTURE_CULL` line with the scene list's submitted/culled counts) and `LTHEORY_CAPTURE_GC_FRAME=g` (full Lua GC at
frame g). `capture.py` always forces the GC halfway to the first captured frame and fails a scene whose log reports a
texture bound after it was destroyed (`texture ResourceId(n) not found`): a GPU resource kept alive only by a Lua object
that something still draws with shows up there instead of after the first natural GC, minutes into a session. Two
Benchmark variants cover what frame 120 at 1280x720 does not: `BenchmarkPhases` (frames 600, 1000, 1300, 1600, 1850 and
2100: ring fly-through, asteroid zoom and close-up, return, moon zoom and close-up) and `Benchmark1080` (1920x1080). A
minimized window saves nothing (`CAPTURE_SKIPPED`), so the scene fails as missing rather than diffing a 1x1 image.

```sh
python tools/render_validation/capture.py gl [scene...]   # -> target/render_validation/captures/gl/
python tools/render_validation/capture.py --baseline      # copy gl captures into baseline/
python tools/render_validation/compare.py gl              # RMSE / max / % pixels over threshold, heatmaps in captures/gl/diff/
python tools/render_validation/run_all.py gl              # supervisors + capture + compare
```

Hot reload is covered by a separate probe (about two minutes): `python tools/render_validation/hot_reload_probe.py`
edits `material/planet.glsl` while `PlanetTest` runs (a tint, a revert, and a new first `MaterialParams` member) and
checks the pictures against an unedited run and the logs for errors.

`compare.py` needs Pillow. Baseline `.json` stats (fps, draw calls) are informational.

Auto-exposure (asynchronous readback, S8) has its own test, `python tools/render_validation/auto_exposure_test.py [backend]`: the `AutoExposure` scene
runs `RenderCoreSystem:tonemap` over a black and then a mid-gray frame and the script checks the exposure it converges to against
`baseline/auto_exposure.json` (recorded with the synchronous implementation). It is not part of `run_all.py` (whose supervisor count is 13).
