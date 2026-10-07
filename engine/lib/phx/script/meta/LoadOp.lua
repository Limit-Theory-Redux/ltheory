-- AUTO GENERATED. DO NOT MODIFY!
---@meta

-- What a pass does with an attachment's previous contents at begin.
---@class LoadOp
---@field Load integer Keep the existing contents.
---@field Clear integer Clear to the attachment's clear value.
---@field DontCare integer Contents are undefined; the pass overwrites every pixel it reads back.
LoadOp = {
    -- Keep the existing contents.
    Load = 0,
    -- Clear to the attachment's clear value.
    Clear = 1,
    -- Contents are undefined; the pass overwrites every pixel it reads back.
    DontCare = 2,
}

