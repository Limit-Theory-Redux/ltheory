-- Factions: organisations of players that own ships and stations, with relations to each other
-- that decide whom their weapons engage. Design: doc/script/factions.md
return {
    Components = require("Modules.Factions.Components"),
    Manager = require("Modules.Factions.Managers.FactionManager"),
    System = require("Modules.Factions.Systems.FactionSystem"),
}
