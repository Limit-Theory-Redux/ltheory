# Performance Stats Server & Dashboard

The engine ships a built-in HTTP stats server that exposes the render
pipeline's per-frame counters and the Lua producer's profiler scopes in
real time. It is a **diagnostics tool only** — it adds a small per-frame
overhead (one mutex-guarded snapshot copy per frame end) and should be
used for profiling sessions, not shipped builds.

## Enabling

The feature is compiled in with the `stats-server` cargo feature and
activated at runtime with `--stats-server <port>`:

```bash
cargo run -p ltr --features stats-server -- --stats-server 8777
```

The flag sets `PHX_STATS_PORT`, which the engine reads at startup and
spawns the server on (see `engine/lib/phx/src/engine/engine.rs`). The
server is `start_stats_server(port)` in
`engine/lib/phx/src/render/thread/stats_server.rs` — it returns the
shared snapshot sink (`Arc<Mutex<StatsSnapshot>>`) that the render
thread publishes into each frame end.

**Building without the feature** (release builds):

```bash
cargo build -p ltr --release
```

## Endpoints

| Endpoint          | Method | Description                                                      |
|-------------------|--------|------------------------------------------------------------------|
| `/`               | GET    | The dashboard HTML (`stats_dashboard.html`, embedded via include_str!) |
| `/stats.json`     | GET    | Latest `StatsSnapshot` (render-thread counters) as JSON          |
| `/profile.json`   | GET    | Lua producer profiler scopes (with nesting parent) as JSON       |
| `/profile/toggle` | GET    | Enable/disable the producer profiler (picked up on the main thread's next safe point) |

Anything else returns `404`.

### `/profile/toggle`

The producer profiler is a global scope-stack recorder. Toggling is
**deferred**: the request handler only sets a flag; the actual
enable/disable happens on the main thread's next safe point
(`Application:onPreRender`). Never toggle from the request thread —
disabling mid-scope could panic the stack unwinder.

## `/stats.json` — render-thread counters

All times are microseconds (`*_us`). "Last frame" means the frame the
render thread most recently finished executing. The JSON nests the
render-thread counters under a `"render"` object (the executor's
`RenderStats` verbatim); main-thread producer fields and
`server_time_us` sit at the top level. The bundled dashboard merges
the two back into a flat view after fetching. Keys:

| Key | Meaning |
|-----|---------|
| `render.last_frame_time_us` | Total wall time of the last render-thread frame (recv → execute → present) |
| `render.present_wait_us` | Time blocked in the GL buffer swap (vsync/vblank wait) |
| `render.recv_wait_us` / `render.recv_wait_count` | Producer starvation: render thread blocked waiting for commands |
| `main_thread_wait_us` | Time the producer blocked in `end_frame_triple_buffered` (fence throttling) |
| `send_blocked_us_last_frame` / `send_block_count_last_frame` | Producer blocked pushing commands onto the channel |
| `channel_high_water` | Peak command-queue depth this frame (buffering health) |
| `frames_in_flight` | Current triple-buffer slot occupancy (0–3) |
| `render.commands_processed` / `render.draw_calls_cumulative` / `render.state_changes_cumulative` | Cumulative totals since launch |
| `render.commands` / `render.draw_calls` / `render.state_changes` | Per-frame values |
| `render.draw_mesh_calls` | DrawMeshByResource calls (mesh entities) |
| `render.draw_immediate_calls` | DrawImmediate calls (UI/overlay quads) |
| `render.draw_instanced_calls` | DrawInstancedWithData calls (asteroid groups) |
| `render.immediate_vertices` | Vertices submitted via DrawImmediate |
| `render.instanced_data_items` | Per-instance matrix/scale entries submitted |
| `render.texture_bind_calls` / `render.texture_binds_skipped` | Texture binds vs deduped (same texture already bound) |
| `render.texture_cache_invalidations` | Cached texture evicted (by shader bind/unbind) |
| `render.texture_binds_skipped_cumulative` | Cumulative deduped texture binds |
| `render.uniform_cache_hits` / `render.uniform_cache_misses` | Uniform-location cache: hits avoid driver round-trips |
| `uniform_dedup_skips_last_frame` | Uniforms skipped because the value didn't change (main-thread producer cost, top level) |
| `render.shader_bind_commands` | BindShader commands executed |
| `render.shader_redundant_binds` | Binds where the program was already current (deduped) |
| `render.shader_distinct_programs` | Distinct GL programs used this frame |
| `render.category_counts` | Command counts per category (`[u64; 12]`, `CommandCategory` order) |
| `render.category_time_us` | Executor time per category (µs; all zero unless the dashboard is open — timing is opt-in) |
| `server_time_us` | Publication timestamp (µs since the UNIX epoch), used by the dashboard for wall-clock FPS averaging |

**Reading the frame-time budget:** the render thread's frame time is
roughly `recv_wait + execute + present`. If `recv_wait` dominates, the
producer (Lua) is the bottleneck — look at `/profile.json`. If
`present_wait` is large, the frame end blocks in the buffer swap. If
`main_thread_wait` is large, the producer is being throttled by the
triple-buffer fence, which is the healthy state.

## `/profile.json` — producer (Lua) scopes

The Lua side wraps systems in `Profiler.Begin('Name')` / `Profiler.End()`
calls. The server returns:

```json
{
  "enabled": true,
  "scopes": [
    { "name": "DrawScene.ECS", "parent": "Opaque.DrawScene",
      "scope_pct": 38.2, "cumul_pct": 90.0,
      "total_ms": 41230, "min_ms": 3.10, "max_ms": 15.32, "mean_ms": 4.57 }
  ]
}
```

| Field | Meaning |
|-------|---------|
| `name` | Scope name (matches `Profiler.Begin` argument) |
| `parent` | Scope that was active when this one began (`""` = top-level). Drives flame-graph rendering |
| `scope_pct` | Percent of the recording window spent in this scope |
| `cumul_pct` | Percent of *all* recorded time (recording window) |
| `total_ms` | Total time in scope across the recording |
| `min_ms` / `max_ms` | Min/max per-frame time |
| `mean_ms` | Mean per-frame time — the number to compare against other scopes |

**The frame-time flame graph** (dashboard "Frame time explanation"
section) combines `/stats.json` (frame total, starve, present) with
`/profile.json` (producer scopes) into one budget view: each scope is a
bar whose width = `mean_ms` relative to the widest scope, nested under
its `parent`, color-hashed per name. This is the fastest way to see
*which system contains which* (e.g. `Physics Update` inside
`App.onPreRender`, the three render passes inside `Canvas.Draw`, `GC.Step`
inside `App.onPostRender`).

### Recording a profile

1. Open `http://127.0.0.1:8777/` in a browser.
2. Hit `/profile/toggle`.
3. Let it record 10+ seconds (the scopes are rolling means; single-frame
   reads are noise).
4. Read `/profile.json` while it's still recording — the snapshot is
   cumulative from the toggle.
5. Toggle again to stop. The counters keep their last values until the
   next enable resets them.

## Dashboard

`GET /` serves the self-contained dashboard. It polls `/stats.json` and
`/profile.json` every second and renders:

- live frame time + starve/present rows
- per-frame counters (commands, draws, shader/texture binds)
- category breakdown table
- **Frame time explanation**: flame graph of the producer scopes overlaid
  with the render-thread frame budget (the "where does the ms go" view)

No external JS/CSS — it's a single embedded HTML file
(`engine/lib/phx/src/render/thread/stats_dashboard.html`). Because it's
`include_str!`'d, **edits require rebuilding the binary**; a Lua-only
change won't reach the served page.

## Source map

| File | Role |
|------|------|
| `engine/lib/phx/src/render/thread/stats_server.rs` | tiny_http server: endpoints, JSON serialization |
| `engine/lib/phx/src/render/thread/stats_snapshot.rs` | `StatsSnapshot` struct + sink attachment |
| `engine/lib/phx/src/render/thread/stats_dashboard.html` | Embedded dashboard + flame graph |
| `engine/lib/phx/src/render/thread/command_executor_gl.rs` | Per-frame counter collection (render thread) |
| `engine/lib/phx/src/render/thread/renderer_threaded.rs` | `end_frame_triple_buffered`, fence throttling, `main_thread_wait_us` |
| `engine/lib/phx/src/system/profiler.rs` | Scope stack, parent tracking, snapshot |
| `script/States/Application.lua` | Lua `Profiler.Begin/End` for frame systems (onPreRender/onRender/onPostRender) |
| `script/Legacy/Systems/Overlay/GameView.lua` | `Opaque.BuildLists`, `DrawScene.ECS`, `DrawScene.Recursive` scopes |

## Adding a new scope

1. In Lua: `Profiler.Begin('My.System')` ... `Profiler.End()` — the name
   shows up in `/profile.json` and the flame graph automatically.
2. Keep scope names hierarchical (dot-separated) so the flame graph's
   parent/child rendering is meaningful.
3. Nested Begin/End pairs are fine (stack-based), but the pairs must
   balance — an unbalanced `End` panics the scope stack.
4. Avoid adding scopes inside per-mesh/per-entity hot loops for
   long-term profiling; each Begin/End is a mutex + hash lookup. Use them
   around whole passes, not individual draws (temporary micro-scopes are
   fine for a diagnosis run, remove them after).
