-- AUTO GENERATED. DO NOT MODIFY!
---@meta

---@class ReadbackTicket
ReadbackTicket = {}

-- The read is over: `data` has the pixels, or the read failed (`failed`).
-- Poll this once per frame; it never blocks.
---@return boolean
function ReadbackTicket:ready() end

-- Block until the read is over (see `ready`); `false` if it failed.
-- For code that must see the pixels at a fixed frame, such as
-- deterministic captures; anything else polls `ready` and never waits.
---@param r Renderer
---@return boolean
function ReadbackTicket:wait(r) end

-- The read could not be done; `data` is empty.
---@return boolean
function ReadbackTicket:failed() end

-- The pixels in the requested format, rows from the first up (a copy;
-- empty until `ready`, and after a failure).
---@return Bytes
function ReadbackTicket:data() end

---@return integer
function ReadbackTicket:getWidth() end

---@return integer
function ReadbackTicket:getHeight() end

-- Give the ticket up. `data` is empty afterwards; the pixels of a read
-- still in flight are dropped when they arrive.
function ReadbackTicket:release() end
