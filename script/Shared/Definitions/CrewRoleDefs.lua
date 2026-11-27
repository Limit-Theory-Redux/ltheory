---The roles a person can hold where people serve: aboard a ship now, at a station later. A role that
---`commands` makes its holder the one who decides for the post (the post's AI rides on them); a post
---has one commander at a time, and command can pass to another member. `share` is the part of what
---the post earns that its owner pays the holder, when the owner employs them (a self-employed owner
---keeps it all). Roles are data: a new one is an entry here.

---@class CrewRoleDefinition
---@field id string
---@field name string
---@field commands boolean
---@field share number Part of the post's earnings, 0 to 1
---@field rank integer Who takes command when it falls vacant: the highest rank aboard

---@type table<string, CrewRoleDefinition>
local CrewRoles = {
    Captain = { id = "Captain", name = "Captain", commands = true, share = 0.10, rank = 3 },
    Mate = { id = "Mate", name = "Mate", commands = false, share = 0.06, rank = 2 },
    Hand = { id = "Hand", name = "Hand", commands = false, share = 0.03, rank = 1 },
}

return CrewRoles
