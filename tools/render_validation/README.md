# Render validation

Each script launches `ltr` with one `script/States/App/Rendering/*` scene,
waits for it to exit, and checks the probe values the scene logs (pixel
readbacks) against expected thresholds.

```sh
python tools/render_validation/indexed_mrt_supervisor.py wgpu
```

The renderer has one backend, wgpu; the backend argument only labels the
artifacts, and anything but `wgpu` is rejected. Every run sets
`LTHEORY_WGPU_FATAL=1`, so the first wgpu validation error fails the scene.
`WGPU_BACKEND` (`vulkan`, `metal`, `dx12`) overrides the platform's native API.

Build `ltr` first. Logs go to `target/render_validation/`. Override the binary
with `LTHEORY_EXE` or `CARGO_TARGET_DIR`. Exit code 0 means the scene passed.
A run that times out is killed together with its process group
(`launch.py`), on every platform.

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
python tools/render_validation/capture.py wgpu [scene...]   # -> target/render_validation/captures/wgpu/
python tools/render_validation/capture.py --baseline        # copy wgpu captures into baseline/
python tools/render_validation/compare.py wgpu              # RMSE / max / % pixels over threshold, heatmaps in captures/wgpu/diff/
python tools/render_validation/run_all.py wgpu              # supervisors + capture + compare
```

The committed `baseline/` was recorded with the former OpenGL backend on
Windows. Captures differ per driver and platform, so for a change that must
not alter pixels, compare against captures of a reference build instead:
check out the earlier commit in a separate worktree, build it there, run
`capture.py` from that worktree with its own `LTHEORY_VALIDATION_ARTIFACT_DIR`,
then

```sh
python tools/render_validation/compare.py wgpu --reference <ref-artifacts>/captures/wgpu
```

Hot reload is covered by a separate probe (about two minutes): `python tools/render_validation/hot_reload_probe.py [wgpu]`
edits `material/planet.glsl` while `PlanetTest` runs (a tint, a revert, and a new first `MaterialParams` member) and
checks the pictures against an unedited run and the logs for errors.

`compare.py` needs Pillow. Baseline `.json` stats (fps, draw calls) are informational.

Auto-exposure (asynchronous readback, S8) has its own test, `python tools/render_validation/auto_exposure_test.py [backend]`: the `AutoExposure` scene
runs `RenderCoreSystem:tonemap` over a black and then a mid-gray frame and the script checks the exposure it converges to against
`baseline/auto_exposure.json` (recorded with the synchronous implementation). It is not part of `run_all.py` (whose supervisor count is 13).
