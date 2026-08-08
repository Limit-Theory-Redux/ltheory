# Replacing `&mut Renderer` parameters with a cloneable command-queue handle

Companion doc: `ai/multithreaded_rendering.md` (the render-thread architecture this builds
on) and `ai/skills/rendering` (day-to-day conventions for `Renderer`/`RenderCommand`).

## Problem

Every GL-touching type (`Tex1D/2D/3D`, `TexCube`, `Mesh`, `Shader`, `Font`, `LodMesh`, `Draw`,
`RenderState`, ...) takes `r: &mut Renderer` as an explicit parameter, because that is the
only handle to the command channel. Lua has no way to supply that argument, so every affected
type needs a hand-written shim in `engine/lib/phx/script/ffi_ext/*.lua` that injects the
global `Renderer` set by `SetEngine`. As of 2026-08-07 that is **22 files, ~240 injection
sites** (worst offenders: `Shader.lua` 37, `Draw.lua` 27, `Tex2D.lua` 21, `ShaderState.lua` 18,
`ShaderVar.lua` 16).

Proposed fix: give each resource a `Sender<RenderCommand>` (or a small handle wrapping one) at
construction so it can submit commands itself, and drop the `Renderer` parameter from methods
that only ever use it to call `submit`.

## Verdict

**Sound, but only solves part of the API — and only that part should move.** `Renderer`
currently plays two distinct roles:

| Role | Replaceable by a channel handle? |
|---|---|
| **Command sink** — `submit(RenderCommand)` into `command_tx` | Yes, cleanly |
| **Owner of `RendererData`** — main-thread CPU state (`imm`, `render_state`, `clip_rect`, `viewport`, `render_target`, `shader_vars`, cached shaders) | **No** — this is mutable CPU state, not commands |

Precedent already in the codebase: `ResourceHandle` (`render/thread/resource_handle.rs:26`)
holds a `Sender<ResourceId>` for exactly this reason — a destructor can never reach
`&mut Renderer`. This proposal generalizes that trick to the command channel itself.

## Which types actually get clean

Measured (2026-08-07) by counting `RendererData` field accesses (`.data.*`) against
`r: &mut Renderer` parameters per file:

| File | `RendererData` uses | `&mut Renderer` params | Verdict |
|---|---|---|---|
| `render/tex2d.rs` | 0 | 21 | fully convertible |
| `render/tex3d.rs` | 0 | 12 | fully convertible |
| `render/tex1d.rs` | 0 | 10 | fully convertible |
| `render/font.rs` | 0 | 5 | fully convertible |
| `render/lod_mesh.rs` | 0 | 1 | fully convertible |
| `render/texcube.rs` | 1 (`irmap_shader` cache, `texcube.rs:443`) | 14 | convertible after moving the cache |
| `render/mesh.rs` | 4 (`ao_shader`/`occlusion_shader` caches, `mesh.rs:735-794`) | 7 | convertible after moving the caches |
| `render/shader.rs` | 1 (`shader_vars` stack read, `shader.rs:512`) | 44 | **blocked** |
| `render/draw.rs` | 124 (`data.imm`) | many | **not convertible** |
| `render_state.rs` / `clip_rect.rs` / `viewport.rs` / `render_target.rs` / `shader_var.rs` | 15 / 6 / 4 / 4 / 3 | — | **not convertible** — they *are* the CPU state |

~60 of ~240 Lua injection sites convert with no design compromise (`Tex1D/2D/3D`, `TexCube`,
`Font`, `LodMesh`). `Mesh` and `TexCube` need their three lazily-created shader caches
(`RendererData.ao_shader`, `occlusion_shader`, `irmap_shader`) rehomed first — pure
memoization, fine as a `OnceCell` on the resource or a thread-local.

`Shader` stays blocked: `Shader::start` reads the auto-var stack at `shader.rs:512`
(`r.data.shader_vars.get(...)`), which `ShaderVar::push_*` mutates from elsewhere — genuine
shared main-thread state, not a command.

`Draw` is not convertible and shouldn't be: `data.imm` is the immediate-mode vertex
accumulator, touched 124 times in `draw.rs`. It is a CPU-side buffer by definition.

## Pros

1. **Deletes most of the FFI shims** — the whole motivation. All of `Tex1D/2D/3D`, `TexCube`,
   `Font`, `LodMesh` shims go away; `Mesh`/`TexCube` mostly.
2. **Removes borrow-checker friction.** A method holding a borrow derived from `Renderer`
   currently can't also call another method taking `&mut Renderer`. `Font::draw`
   (`render/font.rs:190`) already has to clone its `Shader` handle out before its loop for
   exactly this reason.
3. **Blocking readbacks keep working unchanged.** `read_texture_2d_data`,
   `sample_pixel_2d_by_resource`, `get_uniform_location_by_resource`, `create_shader` each
   already build their own `bounded(1)` reply channel (`renderer_threaded.rs:735-830,
   940-965`) — they need a sender, not `&mut Renderer`; only `&mut self` → `&self` changes.
4. **Ordering is preserved for free.** MPSC is FIFO regardless of how many `Sender` clones
   exist, as long as there is exactly one channel.

## Cons

1. **Doesn't eliminate the `Renderer` parameter, only narrows it** — two idioms end up
   coexisting (self-submitting resources vs. `&mut Renderer` state types). Only worth it if
   the split is easy to state: *"resources self-submit; state stacks take `&mut Renderer`."*
2. **`create_resource` needs rework.** It mints ids from a `&mut self` counter
   (`renderer_threaded.rs:199`: `self.data.next_resource_id += 1`). To make constructors
   (`Tex2D.Create`/`Load`/`ScreenCapture`) shim-free too, the counter needs to become an
   `Arc<AtomicU64>` carried in the handle.
3. **Loses the borrow checker as a correctness proof.** `&mut Renderer` today statically
   guarantees a single submitter; afterwards correctness rests on the convention "there is
   only one channel" — still true, but no longer compiler-enforced.
4. **Resources can silently no-op after `Renderer` shutdown** instead of being
   unrepresentable. Already accepted for `ResourceHandle::drop`
   (`resource_handle.rs:44-47`), but would become the norm — wrap the sender rather than
   exposing a bare one, so it can carry the same `running`/disconnect handling `submit` does.
5. **Do not wrap `RendererData` in `Rc<RefCell<_>>` to "finish the job."** That is the one
   change that would cost real performance — see below.

## Memory and performance

**Memory: negligible.** A `crossbeam::channel::Sender<T>` is an `Arc<Channel<T>>` — one
pointer plus a discriminant, ~16 bytes — and it would live once per resource in the shared
`XShared` struct behind the existing `Rf`, not once per clone. 10k live resources ≈ 160 KB.
Cloning a `Sender` is a single atomic increment, paid once at construction — `ResourceHandle`
already pays exactly this cost today.

**Throughput: unchanged.** `Renderer::submit` sends one command per call straight into the
bounded `command_tx`; there is no local batching buffer a per-resource sender would bypass.
Same channel, same `send`, just a different place the `Sender` is reached from.

**The real regression risk** is wrapping `RendererData` in `Rc<RefCell<_>>` so the remaining
CPU state (in particular `data.imm`) stays reachable without `&mut Renderer`. `data.imm` is
touched 124 times inside the immediate-mode vertex loop in `draw.rs` — adding a `RefCell`
borrow-flag check and `Ref` drop to each of those is a measurable cost on the hottest CPU path
in the renderer, and converts a compile error into a runtime `BorrowMutError` panic. Keep
`RendererData` owned by `Renderer`; keep passing `&mut Renderer` to the types that need it.

## Recommended approach (staged, each step independently shippable)

Introduce a small cheap-to-clone handle rather than a bare `Sender`, so the `running` check
and disconnect logging in `submit` aren't duplicated at every call site:

```rust
// render/thread/render_queue.rs
#[derive(Clone)]
pub struct RenderQueue {
    command_tx: Sender<RenderCommand>,
    destroy_tx: Sender<ResourceId>,
    next_resource_id: Arc<AtomicU64>,
    running: Arc<AtomicBool>,
}
```

with `submit(&self, cmd)` mirroring `Renderer::submit`, `create_resource(&self) ->
ResourceHandle` moved off `Renderer` (`renderer_threaded.rs:199`), and `&self`
blocking-readback helpers that build their own `bounded(1)` reply channel exactly as
`renderer_threaded.rs:735+` does now.

(Note: the upstream fork this branch was ported from already has a type named
`RenderQueue` — a global FFI singleton, see `ai/multithreaded_rendering.md` §0. This proposal
is a different, non-global design: an owned handle stored per-resource. Pick a non-colliding
name if porting further fork code later.)

1. **Add `RenderQueue`**; have `Renderer` own one and delegate `submit`/`create_resource` to
   it. Pure refactor — no API change, everything still compiles.
2. **Convert the zero-`RendererData` types**: `Tex1D`, `Tex2D`, `Tex3D`, `Font`, `LodMesh`.
   Store a `RenderQueue` in each `XShared` next to the existing `ResourceHandle`; drop the
   `r: &mut Renderer` parameter from their methods. Delete the corresponding `ffi_ext` shims
   and regenerate bindings.
3. **Rehome the three shader caches** (`RendererData.ao_shader`, `occlusion_shader`,
   `irmap_shader`) so `Mesh` and `TexCube` can convert too.
4. **Stop there.** Leave `Shader` (auto-var stack), `Draw`, `RenderState`, `ClipRect`,
   `Viewport`, `RenderTarget`, `ShaderVar` on `&mut Renderer`. Document the split rule in
   `ai/multithreaded_rendering.md` so the two idioms read as intentional, not inconsistent.

Expected outcome: ~60 of ~240 Lua injection sites gone after step 2, ~85 after step 3, no
change to the immediate-mode hot path, no new interior mutability.

## Verification (when this is implemented)

- `cargo check` / `cargo clippy` per step; check both backends —
  `cargo check -p phx --features immediate` too.
- Regenerate FFI bindings and confirm the deleted `ffi_ext` shims are genuinely unnecessary —
  generated `script/ffi_gen/Tex2D.lua` etc. should no longer take a `Renderer` first arg.
- Run the game (`ai/skills/build-and-run`) and exercise: text rendering (`Font`, hits the
  shader auto-var path that stays on `&mut Renderer`), `Tex2D.Load` + `deepClone`, and an
  HmGui-heavy screen — covers both sides of the split.
- Compare `Renderer::get_stats()` / `get_main_thread_wait_us()` before/after to confirm no
  regression in commands-per-frame or main-thread blocking time.
