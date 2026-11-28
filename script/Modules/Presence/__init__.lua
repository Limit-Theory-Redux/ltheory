-- Presence: entities kept as data while nothing needs them in the physics scene, materialized as
-- constructs where they are seen. Design: doc/script/presence.md
return {
    Components = require("Modules.Presence.Components"),
    System = require("Modules.Presence.Systems.PresenceSystem"),
}
