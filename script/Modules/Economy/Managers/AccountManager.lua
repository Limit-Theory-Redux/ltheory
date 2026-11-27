local Registry = require("Core.ECS.Registry")
local Entity = require("Core.ECS.Entity")
local Economy = require("Modules.Economy.Components")

---Finds who pays for an entity and moves credits between accounts. An entity that trades (a ship, a
---station) pays from its own `PlayerBankAccount`, or else from the account of the entity its
---`OwnershipComponent` names (the player that owns a ship).
---@class AccountManager
---@overload fun(self: AccountManager): AccountManager class internal
---@overload fun(): AccountManager class external
---
---Moves can name a category ("income", "sales", "purchases", "patrol", "ships", "wages", "loot"...):
---the ledger then adds the amount, with its sign, to the account's owner and the category, so where
---credits come from and go can be read (`takeLedger`). Moves into and out of escrow name none.
local AccountManager = Class("AccountManager", function(self)
    ---@type table<integer, table<string, number>> Entity id -> category -> credits since the last take
    self.ledger = {}
    -- What an owner earns (sales and income), credits a second, averaged over about `earningsS`.
    self.earningsS = 1800
    ---@type table<integer, { rate: number, at: number }>
    self.earnings = {}
end)

-- The categories that are earnings: what a station makes by its work.
local EARNINGS = { sales = true, income = true }

---Simulated seconds now.
---@return number
local function now()
    return require("Modules.Economy.Systems.MarketplaceSystem"):now()
end

---What the holder's account owner earns a second, averaged over about `earningsS` (sales and
---production income), as of now.
---@param holder Entity|integer|nil
---@return number credits A second
function AccountManager:earningRate(holder)
    local account = self:accountOf(holder)
    local id = account and account:getEntityId()
    local entry = id and self.earnings[id]
    if not entry then return 0 end
    return entry.rate * math.exp(-(now() - entry.at) / self.earningsS)
end

---Adds a categorised move to the ledger.
---@param account PlayerBankAccount|nil
---@param category string|nil
---@param amount number Signed: what the account gained
function AccountManager:book(account, category, amount)
    if not category or not account or amount == 0 then return end
    local id = account:getEntityId()
    if not id then return end
    local row = self.ledger[id]
    if not row then
        row = {}
        self.ledger[id] = row
    end
    row[category] = (row[category] or 0) + amount
    if EARNINGS[category] and amount > 0 then
        local t = now()
        local entry = self.earnings[id] or { rate = 0, at = t }
        entry.rate = entry.rate * math.exp(-(t - entry.at) / self.earningsS) + amount / self.earningsS
        entry.at = t
        self.earnings[id] = entry
    end
end

---The ledger since the last take, which starts a new one.
---@return table<integer, table<string, number>>
function AccountManager:takeLedger()
    local ledger = self.ledger
    self.ledger = {}
    return ledger
end

---@param entityOrId Entity|integer|nil
---@return Entity|nil
local function resolve(entityOrId)
    if entityOrId == nil then return nil end
    local entity = rawtype(entityOrId) == "number" and Entity(entityOrId) or entityOrId
    if not Registry:hasEntity(entity) then return nil end
    return entity
end

---@param holder Entity|integer|nil
---@return PlayerBankAccount|nil account
function AccountManager:accountOf(holder)
    local entity = resolve(holder)
    if not entity then return nil end
    local account = entity:get(Economy.PlayerBankAccount)
    if account then return account end
    local ownership = entity:get(Economy.Ownership)
    local owner = ownership and resolve(ownership:getOwner())
    return owner and owner:get(Economy.PlayerBankAccount) or nil
end

---@param holder Entity|integer|nil
---@return number balance 0 for an entity without an account
function AccountManager:balanceOf(holder)
    local account = self:accountOf(holder)
    return account and account:getAccountBalance() or 0
end

---Takes credits out of an account (for an escrow or a payment). Fails, taking nothing, when the
---account lacks them.
---@param account PlayerBankAccount|nil
---@param amount number
---@param category string|nil For the ledger
---@return boolean taken
function AccountManager:withdraw(account, amount, category)
    if amount <= 0 then return amount == 0 end
    if not account or account:getAccountBalance() < amount then return false end
    account:take(amount)
    self:book(account, category, -amount)
    return true
end

---@param account PlayerBankAccount|nil
---@param amount number
---@param category string|nil For the ledger
---@return boolean deposited False when there is no account to deposit into
function AccountManager:deposit(account, amount, category)
    if not account then return false end
    if amount > 0 then
        account:put(amount)
        self:book(account, category, amount)
    end
    return true
end

---@param from PlayerBankAccount|nil
---@param to PlayerBankAccount|nil
---@param amount number
---@param category string|nil For the ledger, on both sides
---@return boolean transferred
function AccountManager:transfer(from, to, amount, category)
    if not to or not self:withdraw(from, amount, category) then return false end
    to:put(amount)
    self:book(to, category, amount)
    return true
end

return AccountManager()
