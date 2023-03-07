-- This is a placeholder file for directly adding game content that would
-- typically be added in mod format

-- Items -----------------------------------------------------------------------

local Item = require('Systems.Economy.Item')

Item.T1 = {} -- nonsolid
Item.T2 = {} -- solid, raw, inanimate
Item.T3 = {} -- solid, raw, animate
Item.T4 = {} -- solid, processed, elemental
Item.T5 = {} -- solid, processed, constructed

--                       NAME               UNIT MASS   ENERGY-DENSITY
Item.Energy      = Item("Energy Cell",              1,   1.00)
Item.Data        = Item("Data Cube",                1,   0.10)
Item.Information = Item("Info Wafer",               1,   0.12)
Item.WaterIce    = Item("Water Ice",                4,   1.50)
Item.WaterLiquid = Item("Liquid Water",            10,   1.00)
Item.Isotopes    = Item("Radioactive Isotopes",    18,   2.00)
Item.Silicates   = Item("Silicate Ore",             6,   0.02)
Item.AluminumOre = Item("Aluminum Ore",             7,   0.10)
Item.IronOre     = Item("Iron Ore",                10,   0.18)
Item.CopperOre   = Item("Copper Ore",              11,   0.24)
Item.SilverOre   = Item("Silver Ore",              12,   0.71)
Item.PlatinumOre = Item("Platinum Ore",            15,   1.13)
Item.Animals     = Item("Animals",                  8,   5.00)
Item.Plants      = Item("Plants",                   6,   3.00)
Item.Hydrogen    = Item("Hydrogen",                 1,   2.00)
Item.Helium      = Item("Helium",                   1,   0.01)
Item.Lithium     = Item("Lithium",                  1,   0.20)
Item.Beryllium   = Item("Beryllium",                2,   0.30)
Item.Boron       = Item("Boron",                    2,   0.20)
Item.Carbon      = Item("Carbon",                   2,   0.30)
Item.Nitrogen    = Item("Nitrogen",                 1,   0.05)
Item.Oxygen      = Item("Oxygen",                   1,   2.00)
Item.Aluminum    = Item("Aluminum",                 3,   3.00)
Item.Silicon     = Item("Silicon",                  2,   1.00)
Item.Iron        = Item("Iron",                     5,   3.00)
Item.Copper      = Item("Copper",                   5,   3.50)
Item.Silver      = Item("Silver",                   5,   5.00)
Item.Platinum    = Item("Platinum",                 6,   7.00)
Item.Gold        = Item("Gold",                     8,   6.00)
Item.Thorium     = Item("Thorium",                  9,   9.00)
Item.HullShip    = Item("Ship Hull",            15000,   1.00)
Item.EngineShip  = Item("Ship Engine",            500,   1.00)
Item.WeaponShip  = Item("Ship Weapon",            100,   1.00)
Item.Waste       = Item("Radioactive Waste",        5,   0.50)

insert(Item.T1, Item.Energy)
insert(Item.T1, Item.Data)
insert(Item.T1, Item.Information)
insert(Item.T2, Item.WaterIce)
insert(Item.T2, Item.Isotopes)
insert(Item.T2, Item.AluminumOre)
insert(Item.T2, Item.IronOre)
insert(Item.T2, Item.CopperOre)
insert(Item.T2, Item.SilverOre)
insert(Item.T2, Item.Silicates)
insert(Item.T3, Item.Animals)
insert(Item.T3, Item.Plants)
insert(Item.T4, Item.Hydrogen)
insert(Item.T4, Item.Helium)
insert(Item.T4, Item.Lithium)
insert(Item.T4, Item.Beryllium)
insert(Item.T4, Item.Boron)
insert(Item.T4, Item.Carbon)
insert(Item.T4, Item.Nitrogen)
insert(Item.T4, Item.Oxygen)
insert(Item.T4, Item.Aluminum)
insert(Item.T4, Item.Silicon)
insert(Item.T4, Item.Iron)
insert(Item.T4, Item.Silver)
insert(Item.T4, Item.Platinum)
insert(Item.T4, Item.Gold)
insert(Item.T4, Item.Thorium)
insert(Item.T5, Item.EngineShip)
insert(Item.T5, Item.WeaponShip)

-- Production ------------------------------------------------------------------

local Production = require('Systems.Economy.Production')

Production.EnergySolar = Production('Solar Energy Array')
  :addInput(Item.Waste, 1)
  :addOutput(Item.Energy, 5)
  :setDuration(0.5)

Production.EnergyNuclear = Production('Nuclear Reactor')
  :addInput(Item.Isotopes, 1)
  :addOutput(Item.Energy, 10)
  :addOutput(Item.Waste, 2)
  :setDuration(0.1)

--------------------------------------------------------------------------------
