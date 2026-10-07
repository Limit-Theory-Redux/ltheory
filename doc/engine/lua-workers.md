# Lua Workers (TaskQueue): What They Can Do and Where They Fit

Status: analysis only, nothing implemented. The question was whether the TaskQueue's Lua workers make
sense anywhere in the active code. The answer: **in one place (procedural ship/station mesh generation),
and only after the worker plumbing is fixed**. Everything else that costs time is either GPU work,
needs live game state, runs every frame, or is better done in Rust (with rayon where it is
data-parallel). Frame-time numbers come from `parallel-recording.md` (release, i7-6700K/GTX 1070).
Numbers marked *(est.)* are estimates from reading the code and were not measured.

## 1. How the worker system works today

**Structure.**
- `Engine` owns one `TaskQueue` (`engine/lib/phx/src/engine/engine.rs:28,175`). Lua reaches it as the
  global `TaskQueue` (`script/Main.lua:15`).
- A worker is N OS threads (`LTR-<name>`, `worker_instance.rs:290`) that share one unbounded crossbeam
  input channel and one output channel (`worker.rs:49-68`). That gives a fixed pool per worker type. A
  task goes to whichever instance is free, so instances cannot hold per-task state (no affinity).
  Ordering of results is not guaranteed with more than one instance.
- Engine (Rust) workers: only `Echo` exists (`worker_id.rs:431-437`, `task_queue.rs:33`). Adding a Rust
  worker means a new `WorkerId` variant plus a `Worker::new_native(name, n, Fn(IN)->OUT)`
  (`worker.rs:86-129`). Its IN/OUT can be any `Send` type, so it needs no `Payload`.
- Lua workers: `startWorker(name, scriptPath, n)` (`task_queue.rs:63-154`). Each instance creates a
  fresh LuaJIT state (`Lua::unsafe_new`, `:93`), runs the script (`:95`) and calls its global `Run`
  once per task (`:98,117`). The scripts start with `require('Init')`
  (`script/States/App/Tests/Workers/TestWorkerFunction.lua:4`), which loads all of `Core.*`, the
  structure and util namespaces and the libphx cdefs. So each instance has a startup cost *(est.
  tens of ms)* and is meant to live for the whole session.

**Data passing.**
- Only `Payload` values cross the boundary (`payload/payload.rs:5-36`): scalars, strings, typed
  arrays, and `PayloadTable`, an ordered string-keyed map (`payload_table.rs`). `Payload::Lua` (a
  VM-local cache index) is rejected (`task_queue.rs:231,258`).
- Send path: `PayloadConverter:valueToPayload` (`script/Core/Util/PayloadConverter.lua:26-81`) makes
  one FFI call per scalar or table field. An array is copied twice: Lua table → `ffi.new` → Rust `Vec`.
  The Payload is moved through the channel, boxed, and handed to `Run` as an integer pointer
  (`task_queue.rs:112-118`). `WorkerFunction.Create` (`script/Core/Util/WorkerFunction.lua:15-27`)
  converts it into a Lua value, calls the user function, converts the result back and returns a raw
  pointer that Rust re-boxes (`task_queue.rs:121-123`).
- Receive path: `nextTaskResult` → `TaskResult` → `payloadToValue`. **Arrays come back through
  `Payload_ForEach*` with a Lua closure as the C callback** (`PayloadConverter.lua:124-179`). That is a
  C→Lua callback per element (slow, and it stops JIT traces). LuaJIT also never frees callbacks that
  are created implicitly from a Lua function, and their slots are limited, so every array result
  leaks a callback slot. In practice, results bigger than a few thousand numbers are impractical.

**What a worker may call.** `libphx.lib = ffi.C` (`engine/lib/phx/script/libphx.lua:253`), so a
worker can call the whole engine FFI. Which calls are thread-safe:
- Safe: pure value and object APIs, used on objects the worker owns. That covers `RNG`,
  `Vec*`/`Quat`/`Matrix` math, CPU-side `Mesh` building (`addVertex`, `addQuad`, `computeNormals`; the
  GPU resource is created lazily on first draw, `render/mesh.rs:168-201`), `SDF` from data, and
  `Log`. `Profiler` is safe too (`Mutex`, `system/profiler.rs:68`) but mixes the worker's scopes into
  the main thread's.
- Unsafe: anything that touches `Engine`, `Window`, `Input`, `Renderer`, `TexGen`, textures and
  shaders, the physics world, `EventBus` or HmGui. Fonts use `static mut FT` (`render/font.rs:138`).
- Objects cannot be handed back. `Mesh` wraps `Rf` = `Rc<RefCell<…>>` (`rf.rs:16`) and is `!Send`.
  `Mesh::new` takes its cache uuid from wall-clock nanoseconds (`mesh.rs:216-224`), so two threads can
  get the same uuid.

**Current users.** Outside the three test states in `script/States/App/Tests/Workers/` and
`script/States/App/Workers/Echo.lua`, **nothing in active code uses the TaskQueue** (a grep for
`TaskQueue`, `startWorker`, `sendTask` finds only `Main.lua:15` and `ffi_ext`).

**Known bugs and limitations** (from reading the code):
1. **`nextTaskResult` blocks for up to 500 ms when no result is ready.** `Worker::recv` uses
   `recv_timeout(RECEIVE_TIMEOUT = 500 ms)` (`worker.rs:9,152-169`). Polling it once per frame stalls
   the frame. Guard every poll with `tasksReady() > 0`, or switch to `try_recv`.
2. **The worker tests are broken.** `WorkerTest.lua:6-7` and `WorkerBench.lua:13` point to
   `script/States/App/Tests/TestWorkerFunction*.lua`; the files moved to `Tests/Workers/` in
   `0fc7efd5` (2025-01). `startWorker` logs an error but still returns an id
   (`ffi_ext/TaskQueue.lua:12-15`). The following `sendTask` gets a NULL `uint64*` and indexes it
   (`:22-23`), which crashes.
3. **A Lua error in `Run` kills that instance silently.** The `?` at `task_queue.rs:117` ends the
   thread, the task never gets a result (`TaskResult::new_error` is only used for `Payload::Lua`), and
   a caller waiting for that task id spins forever. A worker function that returns `nil` hits the same
   path through `ffi.gc(nil, nil)`.
4. `tasks_in_progress = in_work - waiting - ready` reads two channel lengths without
   synchronisation and can underflow (`worker.rs:185-187`; a panic in debug).
5. Lua-side value mapping: integers always become `I64` and floats `F64`. A table counts as an array
   only if `value[1] ~= nil`, and the element type comes from element 1 (`PayloadConverter.lua:34-62`).
6. Dropping the queue joins every instance. A worker stuck in a long task blocks shutdown
   (`worker_instance.rs:316-347`).

**Summary.** Lua workers are a coarse-grained job channel. They fit tasks that run for 10 ms or more,
are pure Lua, start from a small input such as a seed, and return a modest result. They cannot see
game state, cannot touch the GPU, and cannot return engine objects.

## 2. Candidate workloads, ranked

Ranked by (time saved × frequency) against cost and risk. "Frame" means it affects steady-state frame
time; "hitch" means a one-off stall at load or spawn.

| # | Workload (where) | Cost | Data in / out | Pure, deterministic? | Best fit | User-visible effect |
|---|---|---|---|---|---|---|
| 1 | **WeaponSystem testbed gameplay Lua**: per-mount target-point sampling `WeaponTrackingSystem.lua:500-531` → `getTargetPoint` `:417-441` → `WeaponSystem:sampleTargetPoint` `WeaponSystem.lua:142-212`; per-mount `getTrackingConfig` merge `WeaponTrackingSystem.lua:84-103` | **16.4-17 ms/frame Lua + 1.2-1.4 ms GC** (measured, whole testbed). Main suspect *(est.)*: ~26 mounts (13 socket pairs, `Testbeds/WeaponSystem.lua:47-330`) × a full scan of every target triangle (thousands per capital hull, ×2-3 passes) with a new `facingCandidates` table and a new `RNG` per call | Needs live bodies, tracks and surfaces | Deterministic, but tied to game state; results are needed in the same frame (`mount.body:setRot`) | **Neither workers nor threads. Fix the algorithm, then move to Rust if still hot** | Frame: up to ~10+ ms (est.) |
| 2 | **Belt cull** `AsteroidBeltRenderer.lua:339-475` | 2.9-3.5 ms/frame (measured, Benchmark) | 100k instances per frame | Pure | **Rust + optional rayon split** (plan D1 in `parallel-recording.md` §4.3, already scheduled) | Frame headroom (GPU-bound today) |
| 3 | **Procedural ship/station meshes**: `ShipGenerator.lua` → `Legacy/Systems/Gen/ShipCapital.lua`, `ShipBasic`, `ShipFighter` (ShapeLib, `Shape:finalize` `ShapeLib/Shape.lua:768`), `CapitalHullGenerator.lua`, plus `HullMountDiscovery:discover` `:459` and `WeaponSystem:buildTargetSurface` `WeaponSystem.lua:102-135` | Unmeasured. *(est.)* 10-100+ ms per hull: thousands of lines of table-based geometry in Lua, then a per-vertex FFI scan. Runs at game start (stations, ships) and **at runtime on every testbed respawn** (`Testbeds/WeaponSystem.lua:1094-1146`) | In: seed, type, hull index (bytes). Out: vertex and index arrays (~0.2-2 MB *(est.)*) plus sockets | Pure given the seed, deterministic, no GPU | **Lua worker** (porting the shape grammar to Rust costs too much), once a mesh handoff exists (§3) | Hitch: removes spawn and respawn stalls; overlaps load |
| 4 | **Belt generation**: `generateBeltAsteroids` `AsteroidBeltRenderer.lua:60-104` and the static-texture build `:110-150` (an `RNG.Create`, `Quat`, `Matrix` per asteroid) | *(est.)* 1-3 µs per asteroid → 20-60 ms for a 20k game belt (`SolarSystemVisualizer.lua:308`), 100-300 ms for Benchmark's 100k (`Benchmark.lua:41`) | In: params. Out: SoA floats (16 per asteroid) | Pure, deterministic per seed | **Rust** inside `InstanceField.Create` (D1), rayon over chunks if needed. A Lua worker would pay the array-return cost and still feed Rust | Load hitch |
| 5 | **Starfield** `Shared/Generation/Starfield.lua` (30-45k stars, `GenConfig.lua:30`; ~15 FFI calls plus several cdata allocations per star) | *(est.)* 20-60 ms at skybox creation (`LTheoryRedux.lua:385-399`) | In: an RNG state shared with the nebula stream. Out: a mesh (~4 vertices per star, ~1.5 MB) | Pure, but depends on the order of the shared RNG stream | **Rust** `Starfield.Generate(seed, count)`, which reseeds deliberately (a one-time visual change) | Load hitch |
| 6 | Asteroid LOD meshes `Core/ECS/Mesh/CelestialObjects/AsteroidMesh.lua:38-47` | GPU density (`TexGen.Volume`) plus Rust SDF meshing; **cached on disk** (`AsteroidMeshPool.lua:47-90`) | – | The CPU part (`SDF:toMesh`, `computeOcclusion`) is pure Rust | Leave it; a rayon split per LOD only if first-run time matters | First run only |
| 7 | Planet cube (~365 ms), nebula, IR map `Nebula1.lua`, `GenUtil.ShaderToTexCube` | **GPU** | – | Uses the GPU | Not workers. Optimise on the GPU (G1) or spread over frames | Load hitch |
| 8 | Universe/system generation `UniverseManager.lua:43-215`, `UniverseGenerationSystem.lua:24-150` | Dozens of entities *(est. ≤ a few ms)*; creates ECS entities (game state) | – | Logic is pure, but its output *is* the registry | No | None |
| 9 | Economy and AI | **Not active**: `LTheoryRedux.lua` loads no economy or AI system; only `Tests/Economy/*` and Legacy | – | – | Future: Rust, or snapshot-based worker jobs with coarse ticks | None today |
| 10 | Pathfinding/autopilot `AutoPilotSystem.lua:136` | Direct steering, no search | – | – | No | None |
| 11 | Save/load | Does not exist | – | – | Rust (serde) when it does | – |
| 12 | Planet terrain chunks (the "landable planets" plan; no such doc is in this repo) | Future, per-chunk meshing with heavy noise | In: chunk key and seed. Out: vertex buffers | Pure | **Rust workers** (rayon or a dedicated pool), output written straight into GPU-upload buffers. Lua workers are wrong here: the marshalling cost and no shared noise code | Streaming without frame drops |
| – | SolarSystemPlayable gameplay Lua | 0.74 ms/frame (measured) | – | – | Nothing worth offloading | – |

## 3. Recommendation

**Where Lua workers make sense:** only for coarse, seed-driven procedural generation that is already
written in Lua and too large to port. Ship, station and hull generation (#3) is the one such case in
active code. Universe logic is too cheap, nebula and planet generation are GPU work, and belt and
starfield generation are small loops that belong in Rust.

**Where Rust is better:**
- The WeaponSystem hot path (#1). It is per-frame, tied to game state and needed in the same frame.
  Fix the algorithm first; move it to a Rust `TargetSurface` if it is still hot. This is L1 from
  `parallel-recording.md`.
- The belt cull and belt generation (#2, #4): Rust `InstanceField`, plus rayon for the cull (D1).
- The starfield (#5) and future terrain chunks (#12): Rust functions, with rayon for terrain.
- For any new data-parallel Rust job, use rayon or `Worker::new_native` with typed `Send` IN/OUT, not
  `Payload`.

**What not to do:**
- No per-frame tasks. Each round trip costs a channel hop, conversion of the payload in both VMs, a
  boxed allocation, and up to a frame of latency. For anything under ~1 ms of work the overhead
  exceeds the gain.
- Never call `nextTaskResult` without checking `tasksReady()` (it blocks for up to 500 ms).
- No jobs that need ECS entities, rigid bodies, the physics world, the renderer or `TexGen`.
- Do not return large arrays through today's `PayloadConverter` (per-element callbacks that leak a
  slot each). Do not pass `Mesh` or other `Rf` objects between threads.
- Do not start workers on demand (VM plus `Init` per instance). Start them once at boot, sized
  `min(2, cores-2)`.

### Step plan A: WeaponSystem hot path (biggest win, no threads)

1. **(S)** Confirm the suspects. Run the `PROF_EVSPLIT` subscriber split from `parallel-recording.md`
   §6.1, plus scopes around `WeaponTrackingSystem:update`, `sampleTargetPoint`, `getTrackingConfig`
   and `hasLineOfSight`.
2. **(S)** In `buildTargetSurface`, precompute per surface: FFI float arrays for the normals and a
   cumulative-area CDF. Sample by binary search on the CDF; reject back-facing triangles with a
   bounded number of retries, then fall back to the full scan. That makes each sample O(log T) instead
   of 3×O(T), with no per-call tables. Results stay deterministic per seed but differ from today's.
3. **(S)** Cache `getTrackingConfig` per mount (invalidate it on a weapon or module change). Create
   the per-call `RNG` once per frame instead of once per sample.
4. **(M)** If the testbed's Lua is still above ~5 ms: a Rust `TargetSurface:sample(seed, viewDir,
   minDot, time, amp, freq)` and a batched per-battery tracking or line-of-sight call. Then re-profile
   and record the numbers in `parallel-recording.md`.

### Step plan B: worker-based ship generation (only if the measurement justifies it)

1. **(S)** Measure first. `ShipGenerator.lua:41ff` already has `Gen.Ship*` profiler scopes; add scopes
   for `HullMountDiscovery:discover` and `buildTargetSurface`. Continue only if a spawn costs more
   than ~20 ms.
2. **(S)** Fix the plumbing: a non-blocking `next_task_result` (`try_recv`); NULL checks in
   `ffi_ext/TaskQueue.lua` `startWorker`/`sendTask`; `pcall` inside `WorkerFunction` so that a failure
   comes back as a `TaskResult` error and the thread survives; the stale test paths; bulk array access
   (`Payload_*ArrayPtr/Len` + `ffi.copy`) instead of `ForEach` callbacks.
3. **(M)** Add a `Send` mesh payload (`Payload::MeshData { vertices: Vec<Vertex>, indices: Vec<u32> }`)
   and `Mesh.FromPayload` on the main thread. Only plain `Vec`s cross threads, never `Rf`.
4. **(M)** Add the worker script `script/Workers/ShipGen.lua`: `require('Init')` plus the Legacy
   generator modules, `Run({seed, type, hull}) → {mesh, sockets, surfaceCDF}`. Doing surface and mount
   preprocessing in the worker as well removes the second per-vertex scan on the main thread.
5. **(S)** Add `ConstructManager:createTargetAsync`. The testbed's 3-4 s respawn delay
   (`targetRespawnDelay`) hides the latency; at game start, send all station and ship jobs before
   planet and nebula GPU generation so the two overlap.
6. **(S)** Determinism gate: for 50 seeds, a hash of the worker-generated vertices must equal the
   main-thread result, and a test must exercise the worker restart/error path.
