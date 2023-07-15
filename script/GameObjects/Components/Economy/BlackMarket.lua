local Entity = require('GameObjects.Entity')

local OrderType = {
    Buy = {
        -- Buy orders are sorted such that last element has the highest bid price
        comparator = function (a, b) return a.price < b.price end
    },

    Sell = {
        -- Sell orders are sorted such that last element has the lowest ask price
        comparator = function (a, b) return a.price > b.price end
    },
}

local function addOrder(type, self, actor, item, count, price)
    local data = self.blackMarketData
    if not data[item] then
        data[item] = {
            ordersBuy = {},
            ordersSell = {},
        }
    end

    local orders = type == OrderType.Buy and
        data[item].ordersBuy or
        data[item].ordersSell

    insert(orders, {
        actor = actor,
        item = item,
        count = count,
        countTotal = count,
        price = price,
    })

    -- TODO : Remove credits / items, put in escrow

    sort(orders, type.comparator)
end

local BlackMarket = class(function (self)
    self.data = {}
    self.escrow = {}
end)

function BlackMarket:addBuy(...)
    addOrder(OrderType.Buy, self, ...)
end

function BlackMarket:addSell(...)
    addOrder(OrderType.Sell, self, ...)
end

function BlackMarket:update(e, dt)
    if not GameState.paused then
        Profiler.Begin('BlackMarket.Update')
        for k, v in pairs(self.data) do
            while
                #v.ordersBuy > 0 and
                #v.ordersSell > 0 and
                v.ordersSell[#v.ordersSell].price <= v.ordersBuy[#v.ordersBuy].price
            do
                local orderBuy = v.ordersBuy[#v.ordersBuy]
                local orderSell = v.ordersSell[#v.ordersSell]
                local count = min(orderBuy.count, orderSell.count)
                local price = orderSell.price
                local total = count * price

                -- TODO : Transfer credits from escrow, transfer item from storage
                orderBuy.count = orderBuy.count - count
                orderSell.count = orderSell.count - count

                if false then
                    e:addMessage('Black Market Transaction: %d %s from %s -> %s @ %d/unit (%d total)',
                        count,
                        v.item.name,
                        orderSell.actor:getName(),
                        orderBuy.actor:getName(),
                        price,
                        total)
                end

                if orderBuy.count == 0 then table.remove(v.ordersBuy) end
                if orderSell.count == 0 then table.remove(v.ordersSell) end
            end
        end
        Profiler.End()
    end
end

--------------------------------------------------------------------------------

function Entity:addBlackMarket()
    assert(not self.blackMarket)
    self.blackMarket = BlackMarket()
    self:register(Event.Debug, Entity.debugBlackMarket)
    self:register(Event.Update, Entity.updateBlackMarket)
end

function Entity:debugBlackMarket(state)
    local ctx = state.context
    ctx:text("Black Market")
    ctx:indent()
    for k, v in pairs(self.blackMarket.data) do
        ctx:text(
            "%s : %d/%d buy/sell orders",
            k:getName(),
            #v.ordersBuy,
            #v.ordersSell)
    end
    ctx:undent()
end

function Entity:getBlackMarket()
    assert(self.blackMarket)
    return self.blackMarket
end

function Entity:hasBlackMarket()
    return self.blackMarket ~= nil
end

function Entity:removeMarket()
    assert(self.blackMarket)
    self:unregister(Event.Debug, Entity.debugMarket)
    self:unregister(Event.Update, Entity.updateMarket)
    self.blackMarket = nil
end

function Entity:updateBlackMarket(state)
    self.blackMarket:update(self, state.dt)
end

--------------------------------------------------------------------------------
