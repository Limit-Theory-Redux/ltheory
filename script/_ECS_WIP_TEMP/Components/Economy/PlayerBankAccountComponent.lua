local Component = require("_ECS_WIP_TEMP.Components.Component")

---@class BankAccount
---@field balance integer current account balance
---@field transactions table<Transaction> all account transactions

---@class Transaction
---@field from Entity
---@field to Entity
---@field timestamp TimeStamp

---@class PlayerBankAccount: Component
---@overload fun(self: PlayerBankAccount, initialBalance: number): PlayerBankAccount subclass internal
---@overload fun(initialBalance: number): PlayerBankAccount subclass external
local PlayerBankAccount = Subclass("PlayerBankAccount", Component, function(self, initialBalance)
    self:setComponentName("EconomyPlayerBankAccount")

    -- Set Component Archetype
    self:setArchetype(Enums.ComponentArchetype.PlayerBankAccount)

    self:addPlayerBankAccount(initialBalance)

    self:registerEventHandler("BankAccount:AddTransaction", self.addTransaction)
    self:registerEventHandler("BankAccount:SetAccountBalance", self.setAccountBalance)
end)

---@param initialBalance number
function PlayerBankAccount:addPlayerBankAccount(initialBalance)
    self.account = {
        balance = initialBalance,
        transactions = {}
    }
end

---@param transaction Transaction
function PlayerBankAccount:addTransaction(transaction)
    insert(self.account.transactions, transaction)
end

--* move this into a system
-- ---@param receiver Entity
-- ---@param amount number
-- ---@return boolean wasSuccessful
-- function PlayerBankAccount:requestTransfer(receiver, amount)
--     if amount > self:getAccountBalance() then
--         Log.Warn("Player does not have enough account balance for this transaction")
--         return false
--     end
--
--     local receiverBankAccountComponent = receiver:findComponent("PlayerBankAccount")
--     ---@cast receiverBankAccountComponent PlayerBankAccount
--
--     if not receiverBankAccountComponent then
--         Log.Warn("Receiver bank account not found for entity: " .. tostring(receiver:getGuid()))
--         return false
--     end
--
--     self:removeBalance(amount)
--     receiverBankAccountComponent:addBalance(amount)
--
--     ---@type Transaction
--     local thisTransaction = {
--         from = self,
--         to = receiverBankAccountComponent,
--         amount = amount,
--         timestamp = TimeStamp.Now()
--     }
--     self:addTransaction(thisTransaction)
--     return true
-- end

---@param balance number
function PlayerBankAccount:setAccountBalance(balance)
    self.account.balance = balance
end

---@return BankAccount
function PlayerBankAccount:getBankAccount()
    return self.account
end

---@return integer
function PlayerBankAccount:getAccountBalance()
    return self.account.balance
end

---@return table<Transaction>
function PlayerBankAccount:getAccountTransactions()
    return self.account.transactions
end

return PlayerBankAccount
