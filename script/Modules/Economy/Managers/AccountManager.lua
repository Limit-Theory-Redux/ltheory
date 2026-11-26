local Registry = require("Core.ECS.Registry")
local Entity = require("Core.ECS.Entity")
local Economy = require("Modules.Economy.Components")

---Finds who pays for an entity and moves credits between accounts. An entity that trades (a ship, a
---station) pays from its own `PlayerBankAccount`, or else from the account of the entity its
---`OwnershipComponent` names (the player that owns a ship).
---@class AccountManager
---@overload fun(self: AccountManager): AccountManager class internal
---@overload fun(): AccountManager class external
local AccountManager = Class("AccountManager", function(self) end)

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
---@return boolean taken
function AccountManager:withdraw(account, amount)
    if amount <= 0 then return amount == 0 end
    if not account or account:getAccountBalance() < amount then return false end
    account:take(amount)
    return true
end

---@param account PlayerBankAccount|nil
---@param amount number
---@return boolean deposited False when there is no account to deposit into
function AccountManager:deposit(account, amount)
    if not account then return false end
    if amount > 0 then account:put(amount) end
    return true
end

---@param from PlayerBankAccount|nil
---@param to PlayerBankAccount|nil
---@param amount number
---@return boolean transferred
function AccountManager:transfer(from, to, amount)
    if not to or not self:withdraw(from, amount) then return false end
    to:put(amount)
    return true
end

return AccountManager()
