Config.credits = {
    ["BASED ON THE IDEAS & WORKS OF"] = {
        { name = "Josh Parnell", teams = "Original Limit Theory Development" },
    },
    ["MAINTAINERS"] = {
        { name = "Haron",        teams = "Engine Development & Design" },
        { name = "dga",           teams = "Engine Development & Design" },
        { name = "Flatfingers",   teams = "Scripting, UI/UX & Game Design" },
        { name = "IllustrisJack", teams = "Scripting, UI/UX & Game Design" },
        { name = "BFett",         teams = "UI/UX, Concept Art & Game Design" }
    },
    ["CONTRIBUTORS"] = {
        { name = "Your Name Here", teams = "Pull list from GitHub" },
    }
}

local header_order = {
    "BASED ON THE IDEAS & WORKS OF",
    "MAINTAINERS",
    "CONTRIBUTORS"
}

local styleHeader = { font = { size = 18, weight = 750, family = "Unageo" }, brush = Color(1, 1, 1, 1) }
local styleName = { font = { size = 14, family = "Unageo" }, brush = Color(1, 1, 1, 1) }
local styleTeams = { font = { size = 10, family = "Unageo" }, brush = Color(1, 1, 1, 0.5) }

function Config.credits:formatted()
    local formattedTbl = {}
    for _, creditorType in ipairs(header_order) do
        table.insert(formattedTbl, { creditorType .. "\n", styleHeader })
        for _, creditor in ipairs(Config.credits[creditorType]) do
            table.insert(formattedTbl, { creditor.name .. "\n", styleName })
            table.insert(formattedTbl, { creditor.teams .. "\n\n", styleTeams })
        end
    end
    return formattedTbl
end
