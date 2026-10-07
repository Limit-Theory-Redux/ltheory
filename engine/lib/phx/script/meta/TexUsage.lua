-- AUTO GENERATED. DO NOT MODIFY!
---@meta

-- Usage bits for `TexUsages` and for the Lua `desc.usage` field
-- (`bit.bor(TexUsage.Sampled, TexUsage.CopySrc)`).
---@class TexUsage
---@field Sampled integer Sampled by shaders.
---@field Attachment integer Rendered to (a pass attachment). Not valid for 1D textures.
---@field CopySrc integer Read back or copied from.
---@field CopyDst integer Uploaded to or copied into.
TexUsage = {
    -- Sampled by shaders.
    Sampled = 1,
    -- Rendered to (a pass attachment). Not valid for 1D textures.
    Attachment = 2,
    -- Read back or copied from.
    CopySrc = 4,
    -- Uploaded to or copied into.
    CopyDst = 8,
}

