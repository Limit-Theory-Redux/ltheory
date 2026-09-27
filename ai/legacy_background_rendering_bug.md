# Legacy background rendering bug — investigation plan

## Context

Reported symptom: running the default `Main` entry point, the main menu's
background (nebula/stars) renders correctly. After starting a new game, the
background becomes completely black with oversaturated red/blue/etc regions.

This was found while validating the batch-rendering port
(`doc/engine/batch-rendering.md`), but is **confirmed unrelated to it**:
`RenderCoreSystem.lua` (the only file touched by that work) is never invoked
by `Main`'s actual gameplay path. Traced the real path:

- `cargo run` (no app arg) → `script/States/App/LTheoryRedux.lua` →
  `GameView(...)` (`LTheoryRedux.lua:58,81-83`) → `GameView:draw`
  (`script/Legacy/Systems/Overlay/GameView.lua:14`), which owns its own
  `RenderPipeline()` (`GameView.lua:306`, `script/Render/RenderPipeline.lua`)
  — completely separate from `RenderCoreSystem`/`Renderer:addEntity`.
- `GameView:drawScene` (`GameView.lua:323-345`) draws every
  `RigidBodyComponent`+`RenderComponent` entity directly
  (`mesh.material:start()` → `:updateState(...)` → `mesh.mesh:draw()` →
  `:stop()`, `GameView.lua:329-333`), then triggers
  `GameState.world.currentSystem:render(...)` → `StarSystem:render`
  (`script/Legacy/GameObjects/Entities/StarSystem.lua:293-297`) →
  `Nebula:render` (`script/Legacy/GameObjects/Entities/Objects/Nebula.lua`).
- **Ordering within one blend pass**: entity meshes (ships/stations/planets)
  draw *first*, nebula/stars draw *second*, same pass, same call
  (`GameView.lua:324-338` then `:341-344`). The main menu has few or no
  `RenderComponent` entities; a fresh game spawns many (ships, stations,
  planets, asteroids). This is the most concrete structural difference
  between "menu: correct" and "in game: broken" — nebula's draw is now
  preceded by a large amount of entity rendering it previously wasn't.

Also relevant: `91401ab0 "feat(render): migrate camera and point-light
uniforms to UBOs"` is the most recent commit (predating this session) to
touch both the nebula shaders (`res/shader/fragment/gen/nebula*.glsl`) and
the shared `camera_ubo.glsl`/`light_ubo.glsl` mechanism that now `#define`s
`mView`/`mProj`/`eye`/`starDir` as macros over UBO block members
(`res/shader/include/camera_ubo.glsl:24-29`) for *every* shader that
includes it — a wide blast radius, and the nearest suspicious change in time
to whenever this regression actually started (unknown — not yet bisected).

**Confirmed NOT the cause:**
- `genStarDir` (added in `91401ab0`) *is* set —
  `script/Legacy/Systems/Gen/Nebula/Nebula1.lua:35`
  (`ss:setFloat3('genStarDir', ...)`), called from `System:beginRender()`
  (`StarSystem.lua`) before `System:render()`.
- Camera/light UBOs are created once at boot (`script/Main.lua:22-23`) and
  updated by both pipelines: legacy `Camera.lua:45`
  (`Renderer:updateCameraUbo`) and `GameView.lua:72`
  (`Renderer:updateLightUbo`), alongside the new `CameraManager.lua:266`. Not
  an obviously-missing call.
- The batch-rendering port's Rust changes
  (`engine/lib/phx/src/render/thread/*`, Phases 1-4): the shared
  `get_uniform_location_cached`/`get_uniform_location_for_program` signature
  change is behaviorally identical to before (same cache, same lookup
  semantics), and the new `SetUniformMat4ByGenericName` command has no
  producer outside `process_batch_intern`, which the legacy pipeline never
  calls.

## Hypotheses (ranked by how well they fit "menu OK, in-game broken")

1. **State bleed from entity-mesh drawing into nebula's draw, same pass.**
   `GameView:drawScene` draws all `RenderComponent` meshes immediately before
   `StarSystem:render`/`Nebula:render`, no explicit state reset in between.
   If any in-game-only material/shader (ship/station/planet — legacy
   material system under `script/Legacy/GameObjects/Elements/Material/`,
   *not* `MaterialDefs.lua`) leaves texture units, blend mode, depth state,
   or the currently-bound shader program in an unexpected state, nebula's
   subsequent shaders (`farplane/skybox`, `farplane/starbg`, `gen/nebula*`)
   inherit it. Fits the menu-vs-new-game contrast exactly: the menu has few
   or zero `RenderComponent` entities to draw first.
2. **`camera_ubo.glsl`/`light_ubo.glsl` macro collision.** If any
   legacy-only shader still declares its own `uniform mat4 mView;` /
   `uniform vec3 eye;` / `uniform vec3 starDir;` (pre-migration style)
   *and* also `#include`s `camera_ubo`/`light_ubo`, the `#define` macros
   (`camera_ubo.glsl:24-29`) would collide with or shadow that declaration,
   silently reading UBO data where the shader's author intended a
   Lua-pushed `ShaderVar` value (or vice versa). Needs a direct read of every
   shader reachable from the in-game (not menu) path.
3. **G-buffer/MRT mismatch specific to legacy in-game materials.** Same
   class of bug already ruled out for `RenderCoreSystem`'s ship/station
   materials (`material/metal.glsl` etc. correctly write all deferred
   outputs) — but the *legacy* pipeline may use a different material/shader
   set entirely (`script/Legacy/GameObjects/Elements/Material/`), not yet
   checked for deferred-output completeness against `RenderPipeline.lua`'s
   own MRT config.
4. **Unbalanced `RenderTarget`/`RenderState` push/pop from an uncaught Lua
   error mid-pass**, leaving an FBO or state stack unbalanced across frames
   (same mechanism as the analogous risk found in `RenderCoreSystem`/
   `RenderingPass.lua`, structurally possible in `RenderPipeline.lua` too if
   it has similar start/stop pass boundaries without a `pcall`). Would
   produce a *persistent* black/garbled result matching "still broken",
   unlike a single-frame glitch.
5. **`envMap`/`irMap` genuinely invalid in this scenario.**
   `System:beginRender()` pushes `self.nebula.envMap`/`irMap` via
   `ShaderVar.PushTexCube` (`StarSystem.lua:288-291`) — if
   `self.nebula:forceLoad()` hasn't finished generating these textures yet
   (e.g. first frame(s) after a fresh game start, async/lazy generation),
   nebula's shaders would sample invalid textures. Would explain a
   transient black frame; less likely to explain a *persistent* broken
   background unless generation never completes.

## Investigation steps

1. **Reproduce with full logging.** `cargo run` (Main), through the menu,
   then start a new game; capture stdout/stderr across the transition. Grep
   for shader compile errors/warnings, "does not have uniform", GL errors
   (the `glcheck!` macro), and any Lua traceback at the moment of
   transition.
2. **Read every legacy-only shader for macro collisions** (hypothesis 2):
   `res/shader/fragment/gen/nebula*.glsl`, `farplane/skybox.glsl`,
   `farplane/starbg.glsl`, and whatever fragment/vertex shaders the legacy
   ship/station/planet materials use — check for any `uniform` declaration
   named `mView`/`mProj`/`mViewInv`/`mProjInv`/`eye`/`starDir` that would
   collide with `camera_ubo.glsl`'s `#define`s, or an equivalent collision
   with `light_ubo.glsl`.
3. **Check the legacy material `start()`/`stop()` implementation**
   (`script/Legacy/GameObjects/Elements/Material/VisibleMesh.lua` and
   whatever `mesh.material:start/:updateState/:stop` resolve to) for
   anything left bound/enabled that isn't explicitly undone in `:stop()` —
   texture units, blend mode, depth write, wireframe, scissor — that
   `Nebula:render`'s shaders could inherit (hypothesis 1). Compare against
   what state `RenderState.PushAllDefaults()`/pass boundaries in
   `RenderPipeline.lua` are actually expected to reset between the two draw
   calls.
4. **Check MRT/deferred-output completeness for legacy in-game materials**
   (hypothesis 3): read `RenderPipeline.lua`'s Opaque-equivalent pass config
   (color attachment list) and confirm every legacy ship/station/planet
   fragment shader writes all of them, the same way this was already
   verified for `RenderCoreSystem`'s `material/metal.glsl` etc.
5. **Audit `RenderPipeline.lua`'s pass boundaries for unguarded errors**
   (hypothesis 4): read its render-loop structure fully (it's marked
   `-- TODO JP: Refactor all of this monolithic nonsense`, i.e. known-messy
   legacy code) and check whether an error thrown while drawing one
   `RenderComponent` entity (e.g. a spawned ship/station missing an expected
   component — a plausible gap, since entity content differs between menu
   and a freshly generated game) would leave a `RenderTarget`/`RenderState`
   push unbalanced for the rest of the session, matching a *persistent*
   broken result rather than a one-frame glitch.
6. **Confirm nebula/envMap readiness at the moment of first broken frame**
   (hypothesis 5): check `Nebula:forceLoad()`/generation completion timing
   relative to when `GameView` starts drawing after a new game begins.
7. **If steps 1-6 don't converge**, add temporary Rust-side tracing (same
   technique used validating the batch port: an `AtomicU32`-gated counter in
   `command_executor_gl.rs` around `BindShaderByResource`,
   `SetUniformMat4`/`SetUniformMat4ByName`, and draw commands) to capture the
   actual GL command sequence spanning the menu→new-game transition, since
   screenshot-based visual debugging proved unreliable in this environment
   when this same technique was used to validate the batch rendering port
   (see `doc/engine/batch-rendering.md`) — remove before landing any fix, per
   that same session's precedent.

## Out of scope

- `engine/lib/phx/src/render/thread/*` (the batch-rendering port,
  Phases 1-4) — confirmed not implicated.
- `RenderCoreSystem.lua` / the `Tests/*` app states — not exercised by
  `Main`'s actual gameplay flow; a separate, working pipeline.
