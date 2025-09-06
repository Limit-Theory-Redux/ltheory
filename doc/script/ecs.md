# Introduction to the Limit Theory Redux Entity Component System
## Basics
### General Promise
The Entity Component System (ECS) is an architectural pattern commonly used in game development and data-driven applications. It emphasizes composition over inheritance, providing flexibility, scalability, and performance benefits. This guide will walk you through the core concepts of ECS.

ECS is a design pattern that separates data (components) from behavior (systems), making it easier to manage complex applications. It consists of three main parts:

**Entities**: Unique identifiers representing objects or concepts in the system.

**Components**: Data containers that define the properties or state of an entity.

**Systems**: Logic that operates on entities with specific components.

This separation ensures that data is not tied to behavior, enabling high modularity and reusability.

In LTR all entities & components are stored in the Registry. This allows us to use one single access point to all relevant objects in the game. E.g. this also helps when you only want to access components of a specific type. You gain performance by not requesting all entities with all their components at once but rather accessing the components you want directly from the storage.

### Modules

See [modules documentation](modules.md).

### Entities
An entity is a unique identifier that acts as a container for components. It does not have any data or behavior on its own. Think of entities as empty shells that become meaningful when associated with components.

In LTR entity definition files operate a little bit (but not quite) like blueprints which you might know from other game engines.

Entity definitions are created as functions that return an Entity instance with the appropriate components.

Here is how you create a simple entity:

```lua
local Entity = require("Core.ECS.Entity")
local Economy = require("Modules.Economy.Components")
local Physics = require("Modules.Physics.Components")

---@param definition ItemDefinition
---@param quantity number
---@return EntityId
return function(definition, quantity)
    return Entity(
        definition.name,
        Physics.Mass(definition.mass),
        Economy.Quantity(quantity)
    )
end
```

As you can see, first the Entity module is imported, followed by the Components module. The Entity function constructs an entity with all the specified components.

### Components
Components are plain data structures that define the properties or state of an entity. They are small and focused on a single responsibility. By attaching different combinations of components to entities, you can represent diverse behaviors and characteristics.

They hold minimal game logic, only the most basic data related methods should be provided. Everything else should be defined in a system that handles the specific data.

Here's how you define a component from the component class:

```lua
local Component = require("Core.ECS.Component")

---@class NameComponent: Component
---@overload fun(self: NameComponent, name: string): NameComponent subclass internal
---@overload fun(name: string|nil): NameComponent subclass external
local NameComponent = Subclass("NameComponent", Component, function(self, name)
    self:setComponentName("NameComponent")

    self:setName(name)
end)

---@param name string
function NameComponent:setName(name)
    self.name = name or "Undefined"
end

---@return string
function NameComponent:getName()
    return self.name
end

return NameComponent
```

As you can see a component is very simple in logic. It only provides the necessary data operation methods, usually getters and setters.

### Systems
Systems contain the logic that processes entities with specific components. They query entities with a particular set of components, perform operations on their data, and update the system’s state as necessary.

Lets have a deeper look at the marketplace system:

#### First the system class is created and all needed modules are imported:

```lua
-- Systems
local Registry = require("Core.ECS.Registry")
local InventorySystem = require("Modules.Economy.Systems").Inventory

-- Utilities
local QuickProfiler = require("Shared.Tools.QuickProfiler")
local Helper = require("Shared.Helpers.MarketplaceSystemHelper")

local Items = require("Shared.Registries.Items")

---@class MarketplaceSystem
---@overload fun(self: MarketplaceSystem): MarketplaceSystem class internal
---@overload fun(): MarketplaceSystem class external
local MarketplaceSystem = Class("MarketplaceSystem", function(self)
    ---@diagnostic disable-next-line: invisible
    self:registerVars()
    ---@diagnostic disable-next-line: invisible
    self:registerEvents()
end)
```

#### Then we set all needed class variables and subscribe to all needed engine events:
```lua
---@private
function MarketplaceSystem:registerVars()
    ---@private
    self.profiler = QuickProfiler("MarketplaceSystem", false, false)

    self.rng = RNG.FromTime()
    self.maxUpdateRateDeviation = 0.5
    self.updateRate = 2
end

---@private
function MarketplaceSystem:registerEvents()
    EventBus:subscribe(Event.PreRender, self, self.onPreRender)
end
```

#### In this case we want to do operations in our PreRender engine event. We start out with defining a profiler, so we have metrics for later performance improvement. Since we want to work with marketplaces, we can query the Registry to iterate over all entities with components of type "MarketplaceComponent". This is far more efficient than having to loop through all entities and ask them if they have that component.

#### Here we just do the marketplace logic we want to have for our economy. It iterates through all existing marketplaces, checks if they have a trader, handles trades and sets the next update point in time. The next update is also slightly randomized, so updates are spread across multiple frames for our marketplaces instead of being squeezed into one frame.

```lua
function MarketplaceSystem:onPreRender()
    self.profiler:start()

    local now = TimeStamp.Now()

    for _, marketplace in Registry:iterEntities(MarketplaceComponent) do
        local traderEntityId = marketplace:getTrader()

        if not traderEntityId then
            goto skipMarketplace
        end

        local nextUpdate = marketplace:getNextUpdate()

        if not nextUpdate then
            nextUpdate = TimeStamp.GetFuture(self.updateRate + self.rng:getUniformRange(0, self.maxUpdateRateDeviation))
            marketplace:setNextUpdate(nextUpdate)
            goto skipMarketplace
        end

        -- update
        if now:getDifference(nextUpdate) <= 0 then
            nextUpdate = TimeStamp.GetFuture(self.updateRate + self.rng:getUniformRange(0, self.maxUpdateRateDeviation))
            marketplace:setNextUpdate(nextUpdate)
            --[[ Todo
                - Update orders
                - Update item flow
            ]]

            if Registry:hasEntity(traderEntityId) then
                local bids = marketplace:getBids()
                local asks = marketplace:getAsks()

                local bidsEntities, asksEntities = Helper.getOrderEntities(bids, asks)
                self:processTrades(marketplace, bidsEntities, asksEntities)
            end
        end

        :: skipMarketplace ::
    end
    self.profiler:stop()
end
```

#### Here we process our trades. This works by iterating through all bids and asks and then looks up the order components inside the entity containing the asks / bids. This is another way of accessing components. This allows for much flexibility when working with entities and components alike. The Lua Language Server is set up so that we get autocomplete for component data automatically when assigning the result of `Registry:get` to a local.

```lua
---@param marketplace MarketplaceComponent
---@param bids table<EntityId>
---@param asks table<EntityId>
function MarketplaceSystem:processTrades(marketplace, bids, asks)
    for bidId in Iterator(bids) do
        for askId in Iterator(asks) do
            local bidItemTypeCmp = Registry:get(bidId, OrderItemTypeComponent)
            local bidPriceCmp = Registry:get(bidId, PriceComponent)
            local bidQuantityCmp = Registry:get(bidId, QuantityComponent)

            local askItemTypeCmp = Registry:get(askId, OrderItemTypeComponent)
            local askPriceCmp = Registry:get(askId, PriceComponent)
            local askQuantityCmp = Registry:get(askId, QuantityComponent)
```

#### The data we want to work with is accessed from the components themselves as explained before.
```lua
            local bidItemType = bidItemTypeCmp:getItemType()
            local bidPrice = bidPriceCmp:getPrice()
            local bidQuantity = bidQuantityCmp:getQuantity()
            local askItemType = askItemTypeCmp:getItemType()
            local askPrice = askPriceCmp:getPrice()
            local askQuantity = askQuantityCmp:getQuantity()
```

#### Here we can see that we are getting the entity ID from a marketplace component. Components hold data about which entity they are part of.

```lua
            -- Verify Inventory
            self.marketplaceParentEntity = marketplace:getEntityId()
            self.marketplaceInventoryCmp = Registry:get(self.marketplaceParentEntity, InventoryComponent)
```

#### Here we use helper which was defined in a seperate file. Sometimes we want some additional methods that don´t have to be part of the system itself for cleanliness. Helper files can be defined for that purpose.

```lua
            Helper.printInventory(self.marketplaceParentEntity, self.marketplaceInventoryCmp)

            if bidItemType == askItemType and bidPrice >= askPrice then
                -- todo: reserve items here, put trade into trade queue for performance control
                -- todo: verify bank account in trade

                -- Calculate trade quantity
                local tradeQuantity = math.min(bidQuantity, askQuantity)
```

#### We access another system here. Functionality should be split reasonably between different systems. Every system should have a definite purpose. As we don´t want the marketplace to handle our inventories by itself we created an `InventorySystem`. This way other systems also can make use of inventory logic without resulting in duplicate code. The idea is to have a lot of systems that are as generic as possible so they can be reused, reducing redundancy where appropriate.

```lua
                -- Attempt to take the required items from the inventory
                local items = InventorySystem:take(self.marketplaceInventoryCmp, askItemType, tradeQuantity)

                if items then
                    -- Put traded items into the marketplace inventory (to simulate transfer)
                    for _, item in ipairs(items) do
                        Registry:destroyEntity(item) --! temp destroy
                    end

                    Log.Debug("[Transaction] Trader 1 %s (%d) -> Trader 2 for price %d credits", Items:getDefinition(bidItemType).name,
                        tradeQuantity,
                        bidPrice)

                    -- Update the inventory quantities
                    bidQuantity = bidQuantity - tradeQuantity
                    askQuantity = askQuantity - tradeQuantity
```

#### At the end of our transaction we destroy the order entity for the bids and asks that are completed. The `Registry:destroyEntity()` method leads to dropping the entity as a whole from the Registry and all components that are linked to it. It simply gets wiped out, so this should be handled with care.

```lua
                    -- Update or remove the bid and ask orders
                    if bidQuantity == 0 then
                        marketplace:removeBid(bidId)
                        Registry:destroyEntity(bidId)
                    else
                        bidQuantityCmp:setQuantity(bidQuantity)
                    end

                    if askQuantity == 0 then
                        marketplace:removeAsk(askId)
                        Registry:destroyEntity(askId)
                    else
                        askQuantityCmp:setQuantity(askQuantity)
                    end

                    Helper.printInventory(self.marketplaceParentEntity, self.marketplaceInventoryCmp)
                end

                break
            end
        end
    end
end

return MarketplaceSystem()
```
