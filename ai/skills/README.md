# AI Skills for Limit Theory Redux

Skills for AI coding assistants working on this repository. Each skill is a folder containing a `SKILL.md` with `name` and `description` frontmatter (standard Agent Skills format). Read `project-overview` first to orient.

| Skill | Use when |
|-------|----------|
| [project-overview](project-overview/SKILL.md) | Orienting in the codebase: workspace layout, Rust↔Lua architecture, doc index. |
| [build-and-run](build-and-run/SKILL.md) | Building, launching the game or test apps, running tests, formatting, lint rules. |
| [luajit-ffi-bindings](luajit-ffi-bindings/SKILL.md) | Exposing/changing Rust APIs callable from Lua; regenerating `ffi_gen`/`meta` files. |
| [lua-scripting](lua-scripting/SKILL.md) | Writing game-side Lua: boot flow, directory roles, app states, conventions. |
| [ecs](ecs/SKILL.md) | Adding gameplay entities, components, or systems in `script/Modules/`. |
| [rendering](rendering/SKILL.md) | Working on `engine/lib/phx/src/render/`, incl. the multithreaded render thread. |
| [lua-workers](lua-workers/SKILL.md) | Offloading Lua computation to background threads via TaskQueue/Payload. |
| [hmgui](hmgui/SKILL.md) | Building in-game UI with HmGui and the Pages/Views/Components UI Router. |
