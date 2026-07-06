---
name: ecs
description: The Lua-side Entity Component System — Registry, entities, components, systems, and the Modules folder layout. Use when adding gameplay features, components, or systems to the game layer.
---

# Entity Component System (Lua)

LTR's gameplay code follows an ECS pattern implemented in Lua. Core classes live in `script/Core/ECS/` (`Entity`, `Component`, `Registry`); gameplay content lives in `script/Modules/`. Docs: `doc/script/ecs.md` and `doc/script/modules.md`.

- **Entities** are identifiers/containers with no behavior; created from *definition functions* that compose components.
- **Components** are plain data holders with only getters/setters — no game logic.
- **Systems** hold the logic; they query the **Registry** for entities with specific component types and subscribe to engine events.
- The **Registry** (`script/Core/ECS/Registry.lua`) is the single storage/access point for all entities and components. Query components by type directly (e.g. iterate all `MarketplaceComponent`s) instead of scanning entities.

## Module layout

Each module in `script/Modules/<Name>/` may contain `Entities/`, `Components/`, `Systems/`, each with an `__init__.lua` exporting its contents:

```lua
-- script/Modules/Physics/Components/__init__.lua
return {
    Mass = require('Modules.Physics.Components.MassComponent'),
    RigidBody = require('Modules.Physics.Components.RigidBodyComponent'),
    Transform = require('Modules.Physics.Components.TransformComponent'),
}
```

Existing modules: `Cameras`, `CelestialObjects`, `Constructs`, `Core`, `Economy`, `Physics`, `Rendering`, `Spatial`, `UI` (see `script/Modules/README.md`).

## Patterns

Entity definition (a function returning `Entity.Create(...)` with components):

```lua
local Entity = require("Core.ECS.Entity")
local Physics = require("Modules.Physics.Components")

return function(definition, quantity)
    return Entity.Create(definition.name, Physics.Mass(definition.mass), ...)
end
```

Component (subclass of `Component`, data + accessors only):

```lua
local Component = require("Core.ECS.Component")

---@class NameComponent: Component
local NameComponent = Subclass("NameComponent", Component, function(self, name)
    self:setComponentName("NameComponent")
    self:setName(name)
end)
```

System (a `Class` that registers vars and subscribes to events in its constructor):

```lua
local MarketplaceSystem = Class("MarketplaceSystem", function(self)
    self:registerVars()      -- profiler (Shared.Tools.QuickProfiler), rng, rates...
    self:registerEvents()    -- EventBus:subscribe(Event.PreRender, self, self.onPreRender)
end)
```

Good reference implementations: `script/Modules/Economy/Systems/MarketplaceSystem.lua`, `script/Modules/Physics/Components/*.lua`.

## Guidelines

- Keep components dumb; put behavior in systems.
- Spread periodic work across frames (randomize next-update times) rather than updating everything in one frame.
- Use `QuickProfiler` in systems so performance can be measured.
- Item/data registries live in `script/Shared/Registries/`.
