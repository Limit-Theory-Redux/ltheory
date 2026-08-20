return {
    ConstructManager = require("Modules.Constructs.Managers.ConstructManager"),
    ConstructionScope = require("Modules.Constructs.Managers.ConstructionScope"),
    ShipArmamentManager = require("Modules.Constructs.Managers.ShipArmamentManager"),
    Generators = {
        ShipGenerator = require("Modules.Constructs.Managers.Generators.ShipGenerator"),
        CapitalHullGenerator = require("Modules.Constructs.Managers.Generators.CapitalHullGenerator"),
        HullMountDiscovery = require("Modules.Constructs.Managers.Generators.HullMountDiscovery"),
        StationGenerator = require("Modules.Constructs.Managers.Generators.StationGenerator"),
        TargetGenerator = require("Modules.Constructs.Managers.Generators.TargetGenerator"),
        TurretLoadoutGenerator = require("Modules.Constructs.Managers.Generators.TurretLoadoutGenerator"),
        LoadoutGenerator = require("Modules.Constructs.Managers.Generators.LoadoutGenerator"),
    },
}
