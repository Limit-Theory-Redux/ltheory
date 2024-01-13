local Action = require('GameObjects.Action')

local kLeadTime = 1.0
local orbitTime = 30

local MineAt = subclass(Action, function(self, source, target, miningTimePerItem)
    assert(source:hasYield())
    self.source = source
    self.target = target
    self.duration = math.floor(miningTimePerItem)
    self.etimer = 0.0
    self.currentTime = RNG.FromTime():getInt(1, orbitTime)
    self.orbitTime = orbitTime
    self.orbitRadius = source:getRadius() * 1.25 --TODO: replace with mining laser range later
    self.rAxis = RNG.FromTime():getInt(1, 2)
    --printf("MineAt %s from %s to %s", self.source:getYield().item:getName(), self.source:getName(), self.target:getName())
end)

function MineAt:clone()
    return MineAt(self.source, self.target, self.duration, self.etimer)
end

function MineAt:getName()
    return format('MineAt @ %s at %s for %s every %d seconds',
        self.source:getYield().item:getName(),
        self.source:getName(),
        self.target:getName(),
        self.duration)
end

function MineAt:onUpdateActive(e, dt)
    Profiler.Begin('Actions.MineAt.onUpdateActive')
    -- orbit
    self.currentTime = self.currentTime + dt
    local orbitTarget = self.source

    local vector = Vec3f()
    -- define 2 axis orbits
    -- TODO replace with random axis later
    if self.rAxis == 1 then
        vector.x = (math.cos(2 * math.pi * self.currentTime / self.orbitTime) * self.orbitRadius)
        vector.y = 0
        vector.z = (math.sin(2 * math.pi * self.currentTime / self.orbitTime) * self.orbitRadius)
    elseif self.rAxis == 2 then
        vector.x = (math.cos(2 * math.pi * self.currentTime / self.orbitTime) * self.orbitRadius)
        vector.y = (math.sin(2 * math.pi * self.currentTime / self.orbitTime) * self.orbitRadius)
        vector.z = 0
    end

    if self.target:hasDockable() and self.target:isDockable() and not self.target:isBanned(e) and self.target:hasTrader() then
        local item = self.source:getYield().item
        local maxBids = self.target:getTrader():getBidVolumeForAsset(item, e)

        -- Mine 1 unit of item every [duration in seconds as specified when pushing the MineAt action]
        -- (unless instantJobs is true)
        if maxBids > 0 then
            if GameState.debug.instantJobs then
                -- Immediately mine as many units as are bid or as the asset has capacity for
                local addedCount = 0
                for i = 1, maxBids do
                    --Log.Debug("MineAt MINE (instant): [%s (%s)] mining 1 unit of %s from %s, delivering to %s (wants %d)",
                    -- e:getName(), e:getOwner():getName(), item:getName(), self.source:getName(), self.target:getName(),
                    -- maxBids)

                    -- Try to add 1 unit of the item (note that item size is its mass, not necessarily 1 unit of cargo space)
                    if not e:mgrInventoryAddItem(item, 1) then
                        break
                    else
                        addedCount = addedCount + 1
                    end
                end

                if addedCount < maxBids then
                    Log.Debug(
                        "MineAt STOP (instant): [%s (%s)] has mined %d total units of %s from %s, but %s has %s bids!",
                        e:getName(), e:getOwner():getName(), e:mgrInventoryGetItemCount(item), item:getName(),
                        self.source:getName(),
                        self.target:getName(), maxBids)
                end

                e:popAction() -- instant: stop mining when any attempt to mine all units for available bids has completed
            else
                -- Orbit
                self:flyToward(e,
                    orbitTarget:toWorldScaled(vector) + orbitTarget:getVelocity():scale(kLeadTime),
                    orbitTarget:getForward(),
                    orbitTarget:getUp()
                )

                -- Mine 1 unit only when the duration timer for mining this type of item has expired
                self.etimer = self.etimer + dt
                if self.etimer > self.duration then
                    self.etimer = 0

                    Log.Debug(
                        "MineAt MINE (regular): [%s (%s)] mining 1 unit of %s (mass = %s) from %s, delivering to %s (wants %d)",
                        e:getName(), e:getOwner():getName(), item:getName(), item:getMass(), self.source:getName(),
                        self.target:getName(),
                        maxBids)

                    -- Try to add 1 unit of the item (note that item size is its mass, not necessarily 1 unit of cargo space)
                    if not e:mgrInventoryAddItem(item, 1) then
                        Log.Debug(
                            "MineAt STOP (regular): [%s (%s)] mined %d units of %s from %s, but %s wanted %d units!",
                            e:getName(), e:getOwner():getName(), e:mgrInventoryGetItemCount(item), item:getName(),
                            self.source:getName(),
                            self.target:getName(), maxBids)
                        e:popAction() -- regular: stop mining if asset ran out of cargo capacity for 1 unit of this item
                    else
                        -- Remove 1 unit of item from the source if any remain
                        if not self.source:decreaseYield() then
                            Log.Debug(
                                "MineAt STOP (regular): [%s (%s)] mined %d units of %s from %s (%s wanted %d), but yield = 0!",
                                e:getName(), e:getOwner():getName(), e:mgrInventoryGetItemCount(item), item:getName(),
                                self.source:getName(),
                                self.target:getName(), maxBids)
                            e:popAction() -- regular: stop mining if target had no more units of item left to mine
                        end
                    end
                end
            end
        end
    else
        e:popAction() -- instant: stop mining when any attempt to mine all units for available bids has completed
    end
    Profiler.End()
end

return MineAt
