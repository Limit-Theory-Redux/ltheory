---Tags for zones that route planning weighs: what a region is, so that a cost profile can weigh it.
---Registered before any tag component is created, since a component sizes itself on the tag count.
---Loaded from Config/General/GameConfig.lua during config load.

local Tags = require("Shared.Registries.Tags")

Tags:new("ZoneKind", { "AsteroidField", "HostileSpace", "GravityWell" })
