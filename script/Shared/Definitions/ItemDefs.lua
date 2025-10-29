-- Types --
---@type ItemGroup
local ItemGroup = require("Shared.Types.ItemGroup")
---@type ItemDefinition
local ItemDefinition = require("Shared.Types.ItemDefinition")

ItemGroup {
    name = "Virtual",
    items = {
        ItemDefinition { name = "Credit", mass = 0, energyDensity = 0 }
    }
}

ItemGroup {
    name = "Data",
    items = {
        ItemDefinition { name = "Data Cube", mass = 1, energyDensity = 2.1, startEquilibriumPrice = 50 },
        ItemDefinition { name = "Info Wafer", mass = 1, energyDensity = 3.82, startEquilibriumPrice = 75 },
        ItemDefinition { name = "Infolytic Chip", mass = 3, energyDensity = 1.35, startEquilibriumPrice = 100 }
    }
}

ItemGroup {
    name = "Raw Materials",
    items = {
        ItemDefinition { name = "Silicate Ore", mass = 4, energyDensity = 1.3, startEquilibriumPrice = 20 },
        ItemDefinition { name = "Iron Ore", mass = 6, energyDensity = 1.68, startEquilibriumPrice = 30 },
        ItemDefinition { name = "Copper Ore", mass = 7, energyDensity = 1.74, startEquilibriumPrice = 35 },
        ItemDefinition { name = "Aluminum Ore", mass = 4, energyDensity = 1.6, startEquilibriumPrice = 25 },
        ItemDefinition { name = "Beryllium Ore", mass = 4, energyDensity = 1.45, startEquilibriumPrice = 40 },
        ItemDefinition { name = "Thorium Ore", mass = 7, energyDensity = 7.5, startEquilibriumPrice = 100 }
    }
}

ItemGroup {
    name = "Refined Materials",
    items = {
        ItemDefinition { name = "Iron", mass = 5, energyDensity = 1, startEquilibriumPrice = 50 },
        ItemDefinition { name = "Copper", mass = 5, energyDensity = 1, startEquilibriumPrice = 60 },
        ItemDefinition { name = "Aluminum", mass = 3, energyDensity = 1.8, startEquilibriumPrice = 55 },
        ItemDefinition { name = "Silver", mass = 5, energyDensity = 1.5, startEquilibriumPrice = 150 },
        ItemDefinition { name = "Gold", mass = 6, energyDensity = 2, startEquilibriumPrice = 200 },
        ItemDefinition { name = "Platinum", mass = 8, energyDensity = 3, startEquilibriumPrice = 250 },
        ItemDefinition { name = "Steel", mass = 7, energyDensity = 1, startEquilibriumPrice = 70 },
        ItemDefinition { name = "Transparent Aluminum", mass = 5, energyDensity = 1.7, startEquilibriumPrice = 80 },
        ItemDefinition { name = "Glassiron", mass = 11, energyDensity = 1, startEquilibriumPrice = 90 },
        ItemDefinition { name = "Plastic", mass = 3, energyDensity = 1.9, startEquilibriumPrice = 45 },
        ItemDefinition { name = "Thorium", mass = 9, energyDensity = 27, startEquilibriumPrice = 300 }
    }
}

ItemGroup {
    name = "Gases",
    items = {
        ItemDefinition { name = "Hydrogen", mass = 1, energyDensity = 2, startEquilibriumPrice = 15 },
        ItemDefinition { name = "Helium", mass = 1, energyDensity = 1.01, startEquilibriumPrice = 10 },
        ItemDefinition { name = "Nitrogen", mass = 1, energyDensity = 1.05, startEquilibriumPrice = 12 },
        ItemDefinition { name = "Oxygen", mass = 1, energyDensity = 1.5, startEquilibriumPrice = 18 }
    }
}

ItemGroup {
    name = "Consumables",
    items = {
        ItemDefinition { name = "Biomass", mass = 4, energyDensity = 4, startEquilibriumPrice = 60 },
        ItemDefinition { name = "Water Ice", mass = 5, energyDensity = 1, startEquilibriumPrice = 20 },
        ItemDefinition { name = "Liquid Water", mass = 4, energyDensity = 1, startEquilibriumPrice = 25 }
    }
}

ItemGroup {
    name = "Equipment",
    items = {
        ItemDefinition { name = "Ship Computer", mass = 110, energyDensity = 22, startEquilibriumPrice = 500 },
        ItemDefinition { name = "Ship Sensor", mass = 80, energyDensity = 19, startEquilibriumPrice = 400 },
        ItemDefinition { name = "Ship Engine, Thruster", mass = 600, energyDensity = 20, startEquilibriumPrice = 2000 },
        ItemDefinition { name = "Ship Engine, Impeller", mass = 1500, energyDensity = 18, startEquilibriumPrice = 4000 },
        ItemDefinition { name = "Ship Weapon, Pulse Turret", mass = 100, energyDensity = 10, startEquilibriumPrice = 600 },
        ItemDefinition { name = "Ship Weapon, Beam Turret", mass = 120, energyDensity = 10, startEquilibriumPrice = 650 },
        ItemDefinition { name = "Ship Weapon, Launcher Turret", mass = 90, energyDensity = 8, startEquilibriumPrice = 550 },
        ItemDefinition { name = "Ship Weapon, Pulse Bay", mass = 1000, energyDensity = 9, startEquilibriumPrice = 3000 },
        ItemDefinition { name = "Ship Weapon, Beam Bay", mass = 1200, energyDensity = 9, startEquilibriumPrice = 3500 },
        ItemDefinition { name = "Ship Weapon, Cannon Bay", mass = 2500, energyDensity = 10, startEquilibriumPrice = 5000 },
        ItemDefinition { name = "Ship Weapon, Launcher Bay", mass = 800, energyDensity = 7, startEquilibriumPrice = 2500 }
    }
}

ItemGroup {
    name = "Constructs",
    items = {
        ItemDefinition { name = "Ship Hull, Solo", mass = 15000, energyDensity = 50, startEquilibriumPrice = 15000 },
        ItemDefinition { name = "Ship Hull, Small", mass = 32000, energyDensity = 49, startEquilibriumPrice = 30000 },
        ItemDefinition { name = "Ship Hull, Compact", mass = 75000, energyDensity = 47, startEquilibriumPrice = 70000 },
        ItemDefinition { name = "Ship Hull, Medium", mass = 102500, energyDensity = 46, startEquilibriumPrice = 100000 },
        ItemDefinition { name = "Ship Hull, Large", mass = 237250, energyDensity = 45, startEquilibriumPrice = 200000 },
        ItemDefinition { name = "Ship Hull, Very Large", mass = 518000, energyDensity = 42, startEquilibriumPrice = 500000 },
        ItemDefinition { name = "Space Station, Small", mass = 1550000, energyDensity = 40, startEquilibriumPrice = 1000000 }
    }
}

ItemGroup {
    name = "Waste",
    items = {
        ItemDefinition { name = "Waste", mass = 1, energyDensity = 1, startEquilibriumPrice = 5 },
        ItemDefinition { name = "Radioactive Waste", mass = 10, energyDensity = 2.5, startEquilibriumPrice = 30 },
        ItemDefinition { name = "Anode Sludge", mass = 9, energyDensity = 1, startEquilibriumPrice = 15 }
    }
}

ItemGroup {
    name = "Miscellaneous",
    items = {
        ItemDefinition { name = "Energy Cell", mass = 1, energyDensity = 1, startEquilibriumPrice = 20 },
        ItemDefinition { name = "Radioactive Isotopes", mass = 13, energyDensity = 50, startEquilibriumPrice = 500 }
    }
}

-- Annotations
---@class Items
---@field Virtual ItemsVirtual
---@field Data ItemsData
---@field RawMaterials ItemsRawMaterials
---@field RefinedMaterials ItemsRefinedMaterials
---@field Gases ItemsGases
---@field Consumables ItemsConsumables
---@field Equipment ItemsEquipment
---@field Constructs ItemsConstructs
---@field Waste ItemsWaste
---@field Miscellaneous ItemsMiscellaneous

---@class ItemsVirtual
---@field Credit ItemDefinition

---@class ItemsData
---@field DataCube ItemDefinition
---@field InfoWafer ItemDefinition
---@field InfolyticChip ItemDefinition

---@class ItemsRawMaterials
---@field SilicateOre ItemDefinition
---@field IronOre ItemDefinition
---@field CopperOre ItemDefinition
---@field AluminumOre ItemDefinition
---@field BerylliumOre ItemDefinition
---@field ThoriumOre ItemDefinition

---@class ItemsRefinedMaterials
---@field Iron ItemDefinition
---@field Copper ItemDefinition
---@field Aluminum ItemDefinition
---@field Silver ItemDefinition
---@field Gold ItemDefinition
---@field Platinum ItemDefinition
---@field Steel ItemDefinition
---@field TransparentAluminum ItemDefinition
---@field Glassiron ItemDefinition
---@field Plastic ItemDefinition
---@field Thorium ItemDefinition

---@class ItemsGases
---@field Hydrogen ItemDefinition
---@field Helium ItemDefinition
---@field Nitrogen ItemDefinition
---@field Oxygen ItemDefinition

---@class ItemsConsumables
---@field Biomass ItemDefinition
---@field WaterIce ItemDefinition
---@field LiquidWater ItemDefinition

---@class ItemsEquipment
---@field ShipComputer ItemDefinition
---@field ShipSensor ItemDefinition
---@field ShipEngineThruster ItemDefinition
---@field ShipEngineImpeller ItemDefinition
---@field ShipWeaponPulseTurret ItemDefinition
---@field ShipWeaponBeamTurret ItemDefinition
---@field ShipWeaponLauncherTurret ItemDefinition
---@field ShipWeaponPulseBay ItemDefinition
---@field ShipWeaponBeamBay ItemDefinition
---@field ShipWeaponCannonBay ItemDefinition
---@field ShipWeaponLauncherBay ItemDefinition

---@class ItemsConstructs
---@field ShipHullSolo ItemDefinition
---@field ShipHullSmall ItemDefinition
---@field ShipHullCompact ItemDefinition
---@field ShipHullMedium ItemDefinition
---@field ShipHullLarge ItemDefinition
---@field ShipHullVeryLarge ItemDefinition
---@field SpaceStationSmall ItemDefinition

---@class ItemsWaste
---@field Waste ItemDefinition
---@field RadioactiveWaste ItemDefinition
---@field AnodeSludge ItemDefinition

---@class ItemsMiscellaneous
---@field EnergyCell ItemDefinition
---@field RadioactiveIsotopes ItemDefinition
