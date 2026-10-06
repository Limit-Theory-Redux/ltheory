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
