---
name: hmgui
description: Building UI with HmGui (the hybrid immediate/retained GUI in Rust) and the Lua-side UI Router (Pages/Views/Components). Use when creating or modifying any in-game UI.
---

# HmGui UI

Two layers:

1. **HmGui core** (Rust, `engine/lib/phx/src/ui/hmgui/`, README there) — the widget/layout engine, driven from Lua via the global `Gui`.
2. **UI Router** (Lua, `script/UI/HmGui/`, README at `script/UI/HmGui/README.md`) — a Vue.js-inspired structure of Pages → Views → Components built on top of raw HmGui calls.

## HmGui core concepts

Three fundamental widgets — **Text**, **Rect**, **Container** — compose everything else (Button, Checkbox, Menu...). Containers lay out children by one of three models: **Stack** (overlapping), **Horizontal**, **Vertical**. Positioning is controlled by:

- alignment per axis: Center, Left/Top (defaults), Right/Bottom, Expand, Stretch (alignment stretch beats fixed size)
- size: fixed pixels or percent of parent
- decorations: padding, spacing (between children), border, margin

Raw usage from Lua (a frame is bracketed by `Gui:beginGui(resX, resY)` ... `Gui:endGui()`):

```lua
Gui:beginHorizontalContainer()
Gui:setFixedSize(100, 100)
Gui:rect(); Gui:setFixedHeight(10); Gui:setFixedWidth(20)
Gui:endContainer()
```

The full `Gui` API is annotated in `engine/lib/phx/script/meta/HmGui.lua`.

## UI Router (preferred for game UI)

- **Pages** (`script/UI/HmGui/Pages/`) — top level, contain Views. Defined via `UICore.Page { name = "..." }` + `addViewToPage(view)`.
- **Views** (`script/UI/HmGui/Views/`) — contain components and UI logic; defined via `UICore.View { name = "..." }` with `onInput()`/`onUpdate(dt)` hooks; add content with `addContent(component)` (render order = add order).
- **Components** (`script/UI/HmGui/Components/`) — reusable parsed HmGui objects with own state, e.g. `UIComponent.Text { ... }`, `UIComponent.Button { title = "...", callback = fn }`; `UIComponent.RawInput` embeds raw HmGui code.

Wiring in an app state:

```lua
local UIRouter = require('UI.HmGui.UICore.UIRouter')
function AppState:onInit()
    UIPageExample:setView("Main")
    UIRouter:addPage(UIPageExample)
    UIRouter:setCurrentPage("Example")
end
function AppState:onInput(dt) UIRouter:input(dt) end
function AppState:onUpdate(dt)
    Gui:beginGui(self.resX, self.resY)
    UIRouter:update(dt)
    Gui:endGui()
end
```

Dynamic state: pass a getter function instead of a value (`UIComponent.Text { text = getSomeState }`); the component re-reads it each update.

## Reference examples

- Live game UI: `script/UI/HmGui/Pages/MainMenu.lua`, `Gameplay.lua`, `LoadingScreen.lua` with matching folders under `Views/`.
- Tutorial example: `UI/HmGui/Pages/Example.lua` + `UI/HmGui/Views/Example/` (the `UIRouterTest` app referenced in the README now lives in `script/States/App/Tests/_outdated/` — use it for reading, not running).
- Default components: `UI/HmGui/Components/Default`.

Namespaces `UICore`/`UIComponent` etc. are preloaded at boot by `Main.lua` (`Namespace.LoadInline('UI.HmGui.*')`).
