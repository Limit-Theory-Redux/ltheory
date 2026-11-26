---Tags of market orders: what kind of order it is, the legality of the goods, and the conditions of
---the order's market that `MarketplaceSystem` keeps on its orders. Registered before any tag
---component is created, since a component sizes itself on the tag count.
---Loaded from Config/General/GameConfig.lua during config load.

local Tags = require("Shared.Registries.Tags")

Tags:new("TradeType", { "BidOrder", "AskOrder", "Contract", "Auction", "Brokered" })
Tags:new("MarketCondition", { "HighDemand", "LowDemand", "HighSupply", "LowSupply", "Volatile" })
Tags:new("Goods", { "Luxury", "Consumable", "RawMaterial" })
Tags:new("Legality", { "Legal", "Restricted", "Contraband" })
Tags:new("Context", { "OrgTrade", "MissionTrade", "EventTrade" })
