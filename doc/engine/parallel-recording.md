# Parallel Pass Recording: Profile and Design

Status: analysis and design, nothing implemented. The request was to "add parallel pass recording across
worker threads". This document measures where frame time goes after the render API v2 refactor
(`render-api-v2.md`, S2 to S11) and ranks the parallel and non-parallel options against that data.

**Summary.**
- In release builds, none of the measured scenes is limited by recording work: SceneList, the pass
  encoder, ImmBatcher, the FFI and the channel together cost under 0.3 ms per frame on the main thread.
- The render thread's CPU cost for passes is 0.2 to 1.7 ms per frame. Most of its time is spent
  waiting for the GPU.
- The scenes are limited by three other things, and parallel recording speeds up none of them:
  - **the GPU**: PlanetTest, the LTheoryRedux menu, Benchmark at 1080p;
  - **gameplay Lua that is not render work**: WeaponSystem 16-17 ms, most of SolarSystemPlayable's
    main-thread time;
  - **one serial Lua loop**: the Benchmark asteroid belt cull, 2.9-3.5 ms. Today the GPU wait hides it.
- Parallel recording is **not the right lever now** (section 3). Recommended instead:
  1. GPU timing per pass.
  2. Profile the gameplay Lua and move its hot paths into Rust.
  3. Move the belt cull to a Rust bulk path. That loop is the one data-parallel CPU job, so give it an
     optional worker split.
- Parallel wgpu pass encoding stays a documented plan (section 5) with a measurable trigger for when to
  build it.

Two findings came up while profiling that are not about performance: a texture is destroyed while it
is still bound (section 2.5), and the stats dashboard has dead fields (section 2.6).

---

## 1. Method

All numbers come from this machine (i7-6700K 4C/8T, GTX 1070, Windows 10) on HEAD `9df9c832`. They were
taken with an instrumented copy of the tree, built outside `target/`, which is not part of the repo:

- **Build.** `git archive HEAD` was extracted to `target-analysis/snap`, with `res` as a junction. It was
  built into `target-analysis/tgt-rel` (release: LTO, codegen-units 1, `--features stats-server`) and into
  `target-analysis/tgt-dbg` (dev profile; dependencies at opt-level 3, as the workspace sets).
  - Building there needs `NoDefaultCurrentDirectoryInExePath` unset: LuaJIT's `msvcbuild.bat` runs
    `minilua` from its working directory.
- **Main-thread Lua profile.**
  - A wrapper entry script (`-e prof_entry.lua`) turns the engine `Profiler` on for a frame window.
  - It adds the scopes `X.EventLoop` (all Lua of the frame), `RC.BuildPassLists`, and one `Fn:<file>:<line>`
    scope per render function inside `Render.Fns`.
  - A `package.path` override of `ffi_ext/SceneList.lua` adds `SL.Prepare`/`SL.PerDraw`/`SL.Emit`/`SL.Reset`.
  - Profiler scopes are **self time**: a child scope's time is excluded from its parent.
  - The printed table drops scopes under 1% of the window. A scope missing from a table below is
    therefore under about 1% of the frame (≈ 0.02-0.06 ms).
- **Render-thread and main-thread Rust timing.** The snapshot has an added `rtprof` module, enabled by
  `LTHEORY_RT_PROF=start:stop`. It records wall time per command around `executor.execute(cmd)`,
  keyed as follows:
  - `pass.begin|cmds|end:<label>` for pass commands;
  - `SwapBuffers`, `BeginFrame`, `recv_wait`;
  - on wgpu, `submit_frame` (encoder finish plus `queue.submit`), acquire+blit+present, and `present`;
  - on the main thread, `channel_send` and `end_frame` (pacing wait plus swap submit).
- **Settings.** Capture mode (1280x720, fixed dt), `PresentMode.NoVsync`, profile window of frames 300-900
  (WeaponSystem 200-500). Benchmark ran outside capture mode at 1920x1080 borderless with NoVsync, over
  frames 600-8600 (all phases, about 1.3 orbit/zoom/moon cycles).
- **What was not measured.**
  - GPU time per pass: no timestamp queries exist.
  - The `immediate` build.
  - Debug wgpu: the runs were stopped on request to avoid opening more windows. Use the S11 table in
    render-api-v2.md for debug wgpu.
  - Debug WeaponSystem: the run exited at frame 239, before the window.
  - The stats dashboard at runtime (section 2.6 is from code).
- **Noise.** One run each. Another agent's `ltr.exe` and its builds were running at times, so treat the
  debug menu and debug Benchmark rows as noisy (marked).

Abbreviations below: **frame** = main-thread wall time per frame; **wait** = main thread blocked in
`end_frame` (pacing for frames in flight); **Lua** = the Lua scopes that matter; **RT CPU** = render-thread
pass work (`pass.*` sums); **RT GPU wait** = render-thread time in `SwapBuffers`/`BeginFrame` that is the
driver or fence waiting for the GPU.

## 2. Profile (measured)

### 2.1 Release, GL

| scene | frame ms | wait ms | main busy ms | largest main-thread items (ms/frame) | RT CPU ms | RT GPU wait ms | bound by |
|---|---|---|---|---|---|---|---|
| PlanetTest | 2.72 | 2.07 | 0.65 | X.EventLoop 0.13, Bloom setup 0.08, RenderPass_Begin/End 0.08 | ≈0.56 | SwapBuffers 1.89 | **GPU** |
| SolarSystemPlayable | 1.77 | 0.30 | 1.47 | X.EventLoop 0.74 (non-render Lua), GC.Step 0.14, BuildPassLists 0.10, Font_Draw 0.04 | ≈0.79 | 0.74 | **main (Lua, not render)** |
| WeaponSystem | 19.24 | 0.00 | 19.2 | X.EventLoop **16.42** (sim and gameplay Lua outside any render scope), GC.Step 1.38, RenderCore.render 0.22 | ≈1.70 | 0.58 (`recv_wait` 16.8) | **main (gameplay Lua)** |
| Menu, Main view | 5.82 | 4.82 | 1.0 | GC.Step 0.21, UIRouter.Update 0.10 | ≈1.53 | BeginFrame **3.53** + Swap 0.52 | **GPU** |
| Menu, Newgame view | 6.13 | 4.75 | 1.4 | UIRouter.Update 0.35, GC.Step 0.19, Font_Draw 0.07 | ≈1.60 | 3.59 + 0.74 | **GPU** |
| Benchmark 1080p | 8.35 | 4.26 | 4.1 | **AsteroidBeltRenderer fn 2.88**, X.EventLoop 0.50, BuildPassLists 0.14 | ≈0.87 | Swap **5.59** + BeginFrame 1.71 | **GPU** |

Benchmark per phase (release GL, `BENCH` lines): orbit 9.3-10.2 ms, asteroid look 12-13 ms, moon look
4.8-5.5 ms.

### 2.2 Release, wgpu

| scene | frame ms | wait ms | belt fn / X.EventLoop | RT pass encode ms | RT `submit_frame` ms | RT present ms | RT GPU wait (BeginFrame) ms | bound by |
|---|---|---|---|---|---|---|---|---|
| PlanetTest | 2.81 | 2.21 | – / 0.14 | ≈0.20 | 0.54 | 0.34 | 1.60 | **GPU** |
| SolarSystemPlayable | 1.77 | 0.19 | – / 0.81 | ≈0.22 | 0.54 | 0.34 | 0.44 | **main** |
| WeaponSystem | 19.89 | 0.00 | – / **16.98** (+GC 1.22) | ≈0.52 | 0.78 | 0.31 | – (`recv_wait` 18.1) | **main** |
| Menu, Main view | 5.27 | 4.11 | – / 0.11 | ≈0.32 | 0.73 | 0.32 | **3.57** | **GPU** |
| Menu, Newgame view | 5.18 | 3.80 | – / 0.10 | ≈0.33 | 0.71 | 0.31 | 3.53 | **GPU** |
| Benchmark 1080p | 8.19 | 3.28 | **3.50** / 0.53 | ≈0.36 | 0.83 | 0.36 | **6.28** | **GPU** |

What the wgpu executor's CPU work costs per frame:
- recording the passes, about 0.2-0.5 ms;
- `encoder.finish()` plus `queue.submit` at `SwapBuffers`, 0.5-0.8 ms;
- the present blit plus present, about 0.35 ms.

That is 1.1-1.7 ms in all. Splitting that work across threads is the most parallel pass encoding could
save (section 3, option R1).

### 2.3 Debug, GL (no wgpu debug runs; see S11's debug table)

| scene | frame ms | wait ms | largest main-thread items (ms/frame) | bound by |
|---|---|---|---|---|
| PlanetTest | 2.73 | 1.20 | X.EventLoop 0.23, RenderPass_Begin/End 0.20/0.19, Bloom 0.15, SL.Emit 0.03 | GPU |
| SolarSystemPlayable | 3.71 | 0.05 | X.EventLoop 1.27, Font_Draw 0.38, RenderPass_End/Begin 0.35/0.23, BuildPassLists 0.18, Font_GetSize 0.16, GC 0.14 | main |
| Menu, Newgame view | 6.42 | 2.49 | HmGui_Draw 0.70, UIRouter.Update 0.68, Font_Draw 0.37, HmGui_End 0.22, Font_GetSize2 0.17 (about 2.1 ms of UI) | GPU |
| Menu, Main view (noisy) | 14.1 | 11.1 | the render thread's SwapBuffers took 11.5 ms (GPU contention from another process likely) | – |
| Benchmark 1080p (noisy) | 14.2 | 6.66 | **belt fn 4.91**, X.EventLoop 0.85, BuildPassLists 0.32, RenderPass_End/Begin 0.30/0.22, SL.Prepare 0.06 | GPU |
| WeaponSystem | – | – | no data (the run exited early); S11 debug numbers: 34.8 ms frame, main-bound | main |

The difference between debug and release matters for the main-thread part only:
- Rust in debug is slower: `RenderPass_Begin/End` take 0.4-0.6 ms, against 0.08 in release.
- `Font_Draw` takes 0.38 ms in debug against 0.04 in release.
- Lua costs about the same in both builds, because LuaJIT is the same.

So "main-thread bound", as the S6/S11 notes call these scenes, is partly a debug-build effect. In release,
PlanetTest and the menu are GPU-bound.

### 2.4 Against the pre-refactor Benchmark recording (vsync on)

| metric | before (dashboard) | now, release GL 1080p NoVsync (measured) |
|---|---|---|
| Render.Fns | 3.61 ms | the belt fn, 2.88 ms (GL) / 3.50 ms (wgpu) / 4.91 ms (debug). It is the whole of Render.Fns: the other fns are under 1% |
| Render.Opaque | 1.17 ms | under 1% (≤ 0.08 ms; SceneList `SL.*` under 1% in release, `SL.Prepare` 0.06 in debug) |
| Shader_Start / RenderTarget_Push | 0.39 / 0.10 ms | gone (the APIs are deleted) |
| commands/frame | 1,056 | 84 |
| draw calls | 184 (mesh 107 / imm 69 / inst 8) | 57-73 averaged over phases (see 2.5: check culling) |
| render-thread frame time | 2.25 ms | 2.76 ms busy, of which 1.7 ms waits for the GPU fence; pass work ≈ 0.87 ms |
| frame | 9.22 ms (vsync-capped, present 8.66) | 8.35 ms (GPU-bound) |

The CPU recording overhead the old API had (Shader_Start, per-draw uniforms, 1k commands) is gone. What
is left of "Render.Fns" is the belt's per-asteroid Lua loop, which is CPU work that has nothing to do with
recording.

### 2.5 Observations relevant to the reported Benchmark regressions (not investigated)

- **A texture is destroyed while it is still bound.**
  - The GL logs show `WARN bind view: texture ResourceId(N) not found` thousands of times per run:
    - Benchmark: id 45, 7.7k warnings;
    - SolarSystemPlayable: id 119, 19k;
    - WeaponSystem: ids 66 and 133, 11k;
    - menu: id 45, 114.
  - In Benchmark the warnings start at frame ≈ 900. That is exactly where the heap drops from 178 MB to
    54 MB: the first full GC.
  - Likely cause: a Lua `Tex2D` (or similar) that only a bind group or material references by
    `ResourceId` gets collected. Its `ResourceHandle` then destroys the GPU texture while a bind group
    still names it, and GL binds nothing on that unit.
  - That fits "black pixelated hole in the planet" (a planet material texture or a group-0/3 input
    sampling nothing).
  - wgpu logs show no such warning: the executor substitutes its white/zero fallback silently or under
    another message.
  - Bind entries probably need to hold the texture alive: a handle clone, or an `Rf` to the shared cell.
- **Draw count.** Benchmark draws 57-73 per frame now against 184 before. Most of that drop is the imm
  batching of S6 (69 imm draws). Mesh draws were not counted separately (the dashboard was not run).
- **Culling.** SceneList cull stats were not logged. `SceneList::prepare` culls a sphere of radius
  `local_radius * t.scale` at the camera-relative centre against `Renderer:setCamera`'s frustum.
  Benchmark scales the planet after creation (`setScale(earthRadius)`), and the cull uses the rigid
  body's scale. Comparing `getSubmitted/getVisible/getCulled` against the draw count would show
  over-culling quickly.

### 2.6 Stats dashboard (from code at HEAD; not run)

- `RenderStats` fields that nothing writes any more show a constant 0 on the dashboard:
  - both executors: `uniform_cache_hits`, `uniform_cache_misses`, `texture_cache_invalidations`,
    `texture_invalidations_on_shader_bind`, `texture_invalidations_on_shader_unbind`. The legacy
    uniform/shader-bind paths that set them were removed in S3/S6, and `uniform_dedup_skips` went
    entirely.
  - wgpu only: `texture_bind_calls`, `texture_binds_skipped(_cumulative)`, `category_counts`,
    `category_time_us` (gap 17).
- The newer counters `passes`, `pipeline_switches`, `bind_group_switches`, `imm_vertices` and the cache
  sizes have Lua getters but no dashboard panel.
- The other agent is editing the stats right now, so re-check after that lands.

## 3. Options, ranked by expected gain for the measured bottlenecks

Gains are estimates unless stated otherwise. They are derived from the tables above, in release.

| # | option | targets | expected frame-time gain now | cost | verdict |
|---|---|---|---|---|---|
| G1 | **GPU timing per pass** (GL `glQueryCounter`, wgpu `TIMESTAMP_QUERY`), then fix the heaviest GPU pass | PlanetTest, menu, Benchmark (GPU-bound: 2-6 ms of GPU wait) | unknown until measured; it is the only lever for 4 of 6 scenes | S-M | **do first** |
| L1 | **Profile the gameplay Lua** (scopes per event handler, see 6.1) and move hot systems into Rust/ECS bulk paths | WeaponSystem (16.4 ms of unscoped Lua, plus 1.4 ms GC), SSP (0.74 ms) | up to 10+ ms on WeaponSystem (estimated: the work is entirely Lua) | M-L, per system | **do** |
| D1 | **Belt cull as a Rust bulk path** (`InstanceField`), optional worker split over chunks | Benchmark main thread: 2.9 ms (GL) / 3.5 ms (wgpu) / 4.9 ms (debug) | 2.5-3.3 ms of main-thread time (estimated: Rust scalar 0.3-0.5 ms for 100k, about 0.1 ms on 3 workers). Frame time improves only once the GPU stops being the limit; until then it gives latency and headroom | S-M | **do (the one parallel piece)** |
| P1 | Overlap: the wgpu frame is submitted once at `SwapBuffers`; submit after the scene passes too, so the GPU starts earlier | wgpu GPU-bound scenes | latency, not throughput; maybe 0-0.5 ms when the queue runs dry. Measure with G1 | S | try with G1 |
| R1 | wgpu parallel pass encoding (an encoder per pass on workers, submitted in order) | render-thread CPU: 1.1-1.7 ms total, of which ≈0.5-1 ms could be split | **≈0 now**: the render thread is never the limit (it waits for the GPU 1.6-6.3 ms per frame, or for the main thread 16-18 ms). It would save up to ≈0.8 ms of render-thread time if that became the limit | L | **defer**; plan in 5 |
| R2 | RenderBundles for static or repeated lists | the post chain (≈20 one-draw passes), scene passes | ≈0: each post pass records in ≤ 0.06 ms, and bundles do not apply to group-2 ring offsets that change per frame | M | no |
| M1 | Parallel SceneList prepare (cull/sort/`mWorldIT`/DrawBlock) on workers | main thread | ≈0: `SL.*` is under 1% in release (≤ 0.03 ms); 26-312 draws per frame | M (per-worker ring arenas) | **no** |
| M2 | Parallel ImmBatcher, glyph layout or text | menu, SSP UI | ≤ 0.07 ms release (Font_Draw); in debug ≈0.4 ms. The Lua UI (UIRouter 0.35 ms, HmGui) stays serial | M | no |
| M3 | Move `buildPassLists` into Rust/ECS | SSP, Benchmark | 0.10-0.17 ms release (0.18-0.32 debug) | M | later, low priority |
| GL | Parallel GL execution | – | none: one context; shared-context multithreading on GL 3.3 is a driver lottery | – | GL stays serial (documented) |

**Why the main thread does not benefit from parallel recording.** The work that can be parallelised
(cull, sort, DrawBlock writes, PassCmd encoding) costs microseconds at this scene size. The expensive main-thread work is
Lua (gameplay, UI, the belt loop), and Lua cannot be split across threads: there is one VM, and
`TaskQueue` workers have separate VMs without game state and exchange serialised `Payload`s. The way to
speed up Lua is to move the work to Rust, and only a data-parallel Rust kernel (the belt) then gains
from threads.

**Why the render thread does not benefit.** In release it does 0.2-1.7 ms of pass work per frame and
otherwise waits for the GPU or for the main thread. Encoding in parallel shortens a phase that is not on
the critical path.

**Pipelining (c).** Recording already overlaps execution:
- passes stream as Begin, chunks of 512 commands, End;
- the render thread executes frame N while the main thread records it;
- up to 3 frames are in flight (`MAX_FRAMES_IN_FLIGHT`).

When the main thread is the limit, the render thread idles (WeaponSystem: `recv_wait` 16.8 of 19.2 ms).
When the GPU is the limit, a deeper queue only adds latency. The one gap in the overlap is P1: wgpu's
GPU work starts at the end of the frame, since there is one submit at `SwapBuffers`.

## 4. Recommended work, with a plan

### 4.1 G1, GPU timing per pass (both executors)

1. **(S)** wgpu: request `TIMESTAMP_QUERY` (and `TIMESTAMP_QUERY_INSIDE_ENCODERS` where present). Write
   timestamps at each `BeginRenderPass`/`EndRenderPass` (`RenderPassTimestampWrites`) into a per-slot
   query set. Resolve at `SwapBuffers` into a mappable buffer, read it back when the slot's fence
   completes (the existing `BeginFrame` wait), and publish `gpu_pass_us[label]` in `RenderStats`.
2. **(S)** GL: `glQueryCounter(GL_TIMESTAMP)` pairs per pass, one ring of query objects per frame slot,
   read at `BeginFrame` after the slot fence.
3. **(S)** Put them in the dashboard and in the capture's `CAPTURE` line (`gpu_ms`, top 3 passes).

Gate: no pixel changes (RMSE 0 on six captures, GL and wgpu). The overhead with timing on stays under 2%.

### 4.2 L1, gameplay Lua

1. **(S)** Add permanent scopes per event handler (wrap the tunnel in `EventBus:subscribe` with a name
   from the subscriber, behind `Config.debug.profileEvents`). Add per-system scopes in the ECS update
   loop.
2. **(M)** Profile WeaponSystem (16-17 ms) and SSP. Move the top systems (likely the projectile,
   turret and physics sync loops) to Rust bulk calls over component arrays.
3. **(S)** GC: 1.2-1.4 ms per frame in WeaponSystem; reduce per-frame allocation in the same hot paths.

### 4.3 D1, belt cull in Rust with an optional worker split

**API.**
- `InstanceField.Create(posX, posY, posZ, scales, chunkOffsets, chunkIndices, chunkCentroids)`. The SoA
  arrays are copied once, at belt generation.
- `field:cull(eye, forward, origin, pxPerUnitSq, lodPxMinSq[8], renderDistSq, maxDrawn, spawnedMask)`.
- After the cull, per-LOD `(count, RingOffset)` results come back from `field:emit(pass, meshes[8],
  pipeline, groups)`. That call allocates the vertex ring and records `DrawInstancedIndices` per LOD,
  like the Lua code does now.

**Threading model.**
- The `Renderer` (main thread) owns a small `rayon::ThreadPool`. rayon is already in `Cargo.lock`
  through dependencies. The pool has `min(3, cores-1)` threads, is named `phx-cull`, is created lazily,
  and is dropped with the renderer.
- Work splits over the existing angular chunks. Each worker fills per-chunk, per-LOD `Vec<u32>` (reused
  across frames).
- No ring access happens on workers. After `join`, the main thread computes prefix sums, allocates the
  ring once per LOD, and copies the chunk lists in chunk order (≤ 400 KB, ≈0.05 ms).
- This avoids per-worker ring sub-arenas entirely. `UniformRing`/`VertexRing` stay main-thread-only
  allocators.
- Below about 20k instances, or with the pool disabled (`Config.render.parallelCull = false`), it runs
  inline on the main thread.

**Ordering and determinism.**
- The output order equals the Lua loop's order: chunks in order, indices in order within each chunk.
- The `maxDrawn` cap applies in that order. With prefix sums, the first `maxDrawn` survivors are taken
  exactly as the serial loop takes them.
- Float math is f32 in Rust against f64 in Lua. Use `f64` for the distance and LOD comparisons to keep
  LOD choices bit-identical to the current Lua path. Verify with a test that compares Rust against the
  Lua reference on the Benchmark seed.

**Backends.**
- Threaded and immediate renderers: identical. It is main-thread work that records the same `PassCmd`s.
- GL and wgpu executors: untouched.

**Verification.**
- `cargo test`: the culler against a scalar reference on random fields and the Benchmark seed, plus
  the cap and order tests.
- `tools/render_validation/run_all.py gl` (default and `--features immediate`) and `run_all.py wgpu`.
- Six captures RMSE 0, Benchmark included.
- The Benchmark belt fn time is measured before and after with the profiling method of section 1.

**Steps.**
1. **(S)** `InstanceField` in Rust, inline only, plus tests.
2. **(S)** `AsteroidBeltRenderer` calls it. The Lua loop is deleted (no fallback).
3. **(S)** Add the pool and the chunk split; config switch; tests for identical output at 1, 2 and 4
   workers.
4. **(S)** Profile and record the result in this document.

### 4.4 P1, earlier wgpu submit (with G1)

**(S)** Submit the frame encoder at `EndRenderPass` of the last scene pass (or every N passes, or after
about 0.5 ms of recorded work), not only at `SwapBuffers`. `flush_for_write` already handles ordering, so
splitting the frame's submissions is semantically safe. Keep the change only if G1 shows the GPU idling
at the start of the frame.

## 5. Deferred plan: parallel wgpu pass encoding (R1)

**Trigger.** Build this only when render-thread CPU time (`pass.*` + `submit_frame`, excluding the
`BeginFrame`/present waits) exceeds about 70% of the frame in a release build of a shipping scene. Today
the figure is 7-25%.

**Model.**
- *Resolve phase* (render thread, serial, as commands arrive). For each pass it resolves everything that
  touches mutable caches, so that recording needs only immutable data:
  - pipeline variants (`variants`, lazily built);
  - bind groups (`resolve_group`, content cache plus sweep);
  - texture views;
  - the vertex/index buffers of meshes.

  The output is a `ResolvedPass { desc, attachments, ops: Vec<ResolvedOp> }` holding
  `wgpu::RenderPipeline`/`BindGroup`/`Buffer` clones. These are cheap to clone, and they keep the
  objects alive even if a cache sweep runs.
- *Encode phase.* Each finished pass (at `EndRenderPass`) goes to a worker in a small render-thread-owned
  pool. The worker creates its own `CommandEncoder`, records the `RenderPass`, and returns a
  `CommandBuffer` tagged with the pass's sequence number. `Device`/`Queue` are `Send + Sync`, and
  encoders are independent.
- Copies and `GenerateMips` between passes become their own sequence entries (encoded on the render
  thread).
- *Submit phase.* At `SwapBuffers`, `Flush`, a readback, or `flush_for_write`, the render thread joins
  the outstanding sequence numbers and calls `queue.submit` with the command buffers **in sequence
  order**. This preserves today's ordering rules:
  - ring `write_buffer`s are issued before the submission that reads them, as now;
  - texture writes that must not overtake earlier passes still force a submit of everything recorded
    before them.
- *Splitting large passes* (by chunk) is possible with `RenderBundle`s per chunk, executed in order in
  one pass. It is not needed at 26-312 draws per frame.

**Determinism.** The GPU sees the same command sequence: pass order is fixed by sequence numbers, and
within a pass the ops are recorded in stream order by one worker. Captures must stay pixel-identical,
which the gate verifies.

**Stats.** Per-worker `FrameCounters`, merged at submit.

**Backends.**
- GL executor: unchanged and serial (one context).
- Immediate renderer: GL-only today. If wgpu ever runs inline, the same pool works there; the main
  thread would then join at `SwapBuffers`.
- Threaded renderer: the main thread does not change. Passes still stream.

**Main-thread side** (only if main-thread recording ever became the limit; not recommended now): give
each worker that records `PassCmd`s a `StagingRing` that owns whole chunks.
- `RingOffset.buffer` is a chunk index within the frame slot. If the slot's chunk index space is
  partitioned (worker k takes chunks from a shared atomic counter), offsets stay valid without copying.
- Each worker's `PassCmd` vector is concatenated in pass/chunk order before `PassCommands` is sent.
- Material commits and other resource writes stay on the main thread, before the parallel section,
  because writes inside a pass are forbidden.

**Verification.** Same gate as 4.3, plus one `run_all.py wgpu` with a pool of 1 against a pool of N
(bit-identical captures), plus a stress run with readbacks and hot reload while the pool runs.

**Steps.**
1. **(M)** Split `cmd_pass_commands` into resolve and record.
2. **(M)** `ResolvedPass` plus per-pass encoders, still serial. Check that output is identical and
   overhead small.
3. **(M)** The worker pool, sequence-ordered submit, flush points.
4. **(S)** Stats merge and dashboard.
5. **(S)** Profile and gate.

---

## 6. Reproducing the profile

### 6.1 Harness pieces (kept out of the repo)

- `prof_entry.lua`: a wrapper entry script.
  - It runs `script/Main.lua`.
  - It wraps `InitSystem` to force NoVsync.
  - It wraps `RenderCoreSystem.buildPassLists`/`renderInOrder` and `AppEventLoop`, enabling the
    `Profiler` for frames `PROF_START`..`PROF_STOP`.
  - It logs `PROFSTATS` (render-thread busy/recv/present, `main_wait`, commands, draws) from the
    `Renderer:stats*` getters.
- `override/ffi_ext/*`: a full copy of `ffi_ext`, put first on `package.path` before `Main.lua` runs.
  A full copy is needed because `requireAll('ffi_ext')` enumerates the first matching directory.
  - `SceneList.lua` adds the `SL.*` scopes.
  - `EventBus.lua` wraps each subscriber in a scope `Ev<event>:<file>:<line>` with `PROF_EVSPLIT=1`.
    This is the tool for L1; it was not run before the stop.
- `rtprof.rs` plus hooks in `render_thread.rs`, `renderer_threaded.rs`, `renderer_immediate.rs` and the
  wgpu `frame.rs`: per-command render-thread timing and main-thread send/end-frame timing, gated by
  `LTHEORY_RT_PROF`.

### 6.2 Making it permanent

Worth upstreaming in some form: per-pass CPU timing on the render thread as `RenderStats` fields (G1
needs the same plumbing). A dashboard table of `Profiler` scopes per frame already exists.
