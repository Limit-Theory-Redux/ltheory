-- Static object type names and data
Config.objectInfo = {
    {
        ID = "object_types",
        name = "Object Types",
        elems = {
            -- NOTE: If you change these, you must also change autonavRanges!
            { 1,  "Unknown",     "" },
            { 2,  "Reserved",    "" },
            { 3,  "Star Sector", "" },
            { 4,  "Star System", "" },
            { 5,  "Zone",        "zone_subtypes" },
            { 6,  "Star",        "star_subtypes" },
            { 7,  "Planet",      "planet_subtypes" },
            { 8,  "Asteroid",    "asteroid_subtypes" },
            { 9,  "Jumpgate",    "jumpgate_subtypes" },
            { 10, "Station",     "station_subtypes" },
            { 11, "Ship",        "ship_subtypes" },
            { 12, "Colony",      "colony_subtypes" },
        }
    },
    {
        ID = "zone_subtypes",
        name = "Zone Subtypes",
        elems = {
            { 1, "Unknown" },
            { 2, "Reserved" },
            { 3, "Asteroid Field" },
            { 4, "Political Extent" },
            { 5, "Military Extent" },
            { 6, "Economic Extent" },
            { 7, "Cultural Extent" },
        }
    },
    {
        ID = "planet_subtypes",
        name = "Planet Types",
        elems = {
            { 1, "Unknown" },
            { 2, "Brown Dwarf" },
            { 3, "Gas giant" },
            { 4, "Rocky" },
        }
    },
    {
        ID = "planet_subtypes_size",
        name = "Planet Subtypes - Size",
        elems = {
            { 1, "(none)" },
            { 2, "Unknown" },
            { 3, "Large" },
            { 4, "Small" },
        }
    },
    {
        ID = "planet_subtypes_atm",
        name = "Planet Subtypes - Atmosphere",
        elems = {
            { 1,  "Unknown" },
            { 2,  "None (vacuum)" },
            { 3,  "Thin" },
            { 4,  "Thin, tainted" },
            { 5,  "Thin, exotic" },
            { 6,  "Normal" },
            { 7,  "Normal, tainted" },
            { 8,  "Dense" },
            { 9,  "Dense, tainted" },
            { 10, "Dense, exotic" },
        }
    },
    {
        ID = "planet_subtypes_hyd",
        name = "Planet Subtypes - Hydrosphere",
        elems = {
            { 1, "Unknown" },
            { 2, "None (vacuum)" },
            { 3, "Desert (1% - 9% water)" },
            { 4, "Dry (10% - 29% water)" },
            { 5, "Wet (30% - 69% water)" },
            { 6, "Water (70% - 89% water)" },
            { 7, "Ocean (90% - 100% water)" },
        }
    },
    {
        ID = "asteroid_subtypes", -- if you change these, also change massAsteroidExp
        name = "Asteroid Types",  -- until reference functions access the values from here
        elems = {
            { 1, "Unknown",      0.0 },
            { 2, "Reserved",     0.0 },
            { 3, "Carbonaceous", 5.5 },
            { 4, "Metallic",     6.0 },
            { 5, "Silicaceous",  5.0 },
        }
    },
    {
        ID = "jumpgate_subtypes",
        name = "Jumpgate Types",
        elems = {
            { 1, "Unknown" },
            { 2, "Reserved" },
            { 3, "Neighbor" },
            { 4, "Wild" },
        }
    },
    {
        ID = "station_subtypes",
        name = "Station Types",
        elems = {
            { 1, "Unknown" },
            { 2, "Reserved" },
            { 3, "Solar Energy Array" },
            { 4, "Nuclear Reactor" },
            { 5, "Pirate" },
        }
    },
    {
        ID = "station_hulls",
        name = "Station Hulls",
        elems = {
            { 1,  "Unknown" },
            { 2,  "Reserved" },
            { 3,  "Small" },
            { 4,  "Medium" },
            { 5,  "Large" },
            { 6,  "Trade" },
            { 7,  "Market" },
            { 8,  "Depot" },
            { 9,  "Outpost" },
            { 10, "Base" },
            { 11, "Citadel" },
        }
    },
    {
        ID = "ship_subtypes",
        name = "Ship Types",
        elems = {
            { 1,  "Unknown" },
            { 2,  "Reserved" },
            { 3,  "Solo" },
            { 4,  "Small" },
            { 5,  "Compact" },
            { 6,  "Medium" },
            { 7,  "Large" },
            { 8,  "Very Large" },
            { 9,  "Fighter" },
            { 10, "Corvette" },
            { 11, "Frigate" },
            { 12, "Destroyer" },
            { 13, "Cruiser" },
            { 14, "Battleship" },
            { 15, "Courier" },
            { 16, "Trader" },
            { 17, "Merchanter" },
            { 18, "Freighter" },
            { 19, "Bulk Freighter" },
            { 20, "FreighterMax" },
            { 21, "Miner" },
            { 22, "Prospector" },
            { 23, "Digger" },
            { 24, "Driller" },
            { 25, "Dredger" },
            { 26, "Excavator" },
            { 27, "Scout" },
            { 28, "Ranger" },
            { 29, "Seeker" },
            { 30, "Explorer" },
            { 31, "Wayfinder" },
            { 32, "Surveyor" },
            { 33, "Boat" },
            { 34, "Runabout" },
            { 35, "Cabin Cruiser" },
            { 36, "Sloop" },
            { 37, "Yacht" },
            { 38, "Liner" },
            { 39, "Marauder" },
            { 40, "Security" }
        }
    },
    {
        ID = "colony_subtypes",
        name = "Colony Types",
        elems = {
            { 1, "Unknown" },
            { 2, "Reserved" },
            { 3, "Outpost" },
            { 4, "Military Base" },
            { 5, "Manufacturing" },
            { 6, "Trading" },
            { 7, "Research" },
            { 8, "Breeding" },
            { 9, "Consulate" },
        }
    },
    {
        ID = "reserved_subtypes",
        name = "Reserved Types",
        elems = {
            { 1, "Unknown" },
            { 2, "Reserved" },
            { 3, "?" },
            { 4, "!" },
        }
    },
    {
        ID = "1_subtypes",
        name = "1 Subtypes",
        elems = {
            { 1, "Unknown" },
            { 2, "" },
            { 3, "" },
            { 4, "" },
            { 5, "" },
            { 6, "" },
        }
    },
}

--* i think we can organize this better (instead of being part of Config) ~@IllustrisJack
function Config:getObjectTypeByName(objIDname, objtypename)
    -- For a given kind of object (by ID), find the index of the object type provided
    local objIDnum = Config:getObjectTypeIndex(objIDname)

    return Config:getObjectTypeByIDVal(objIDnum, objtypename)
end

function Config:getObjectTypeIndex(objIDname)
    -- For a given kind of object (by ID name), find the index of the object type provided
    local objIDnum = 1 -- default is "Unknown"

    -- Find index number of given object ID in the object types table
    for i = 1, #Config.objectInfo do
        if string.match(objIDname, Config.objectInfo[i].ID) then
            objIDnum = i
            break
        end
    end

    return objIDnum
end

function Config:getObjectTypeByIDVal(objIDnum, objtypename)
    -- For a given kind of object (by ID number), find the index of the object type provided
    local objtype = 1 -- default is "Unknown"

    if objIDnum > 0 then
        -- Scan object types table for match against provided object's type
        -- Return number of object type if found
        for i = 1, #Config.objectInfo[objIDnum]["elems"] do
            if string.match(objtypename, Config.objectInfo[objIDnum]["elems"][i][2]) then
                objtype = Config.objectInfo[objIDnum]["elems"][i][1]
                break
            end
        end
    end

    return objtype
end

function Config:getObjectInfo(objIDname, objtypenum)
    if Config.objectInfo[Config:getObjectTypeIndex(objIDname)] then
        if Config.objectInfo[Config:getObjectTypeIndex(objIDname)]["elems"][objtypenum] then
            return Config.objectInfo[Config:getObjectTypeIndex(objIDname)]["elems"][objtypenum][2]
        end
    end
end

function Config:getObjectSubInfo(objIDname, objtypenum, objsubtypenum)
    if Config.objectInfo[Config:getObjectTypeIndex(objIDname)] then
        if Config.objectInfo[Config:getObjectTypeIndex(objIDname)]["elems"][objtypenum] then
            local subtypename = Config.objectInfo[Config:getObjectTypeIndex(objIDname)]["elems"][objtypenum][3]
            return Config.objectInfo[Config:getObjectTypeIndex(subtypename)]["elems"][objsubtypenum][2]
        end
    end
end