-- AUTO GENERATED. DO NOT MODIFY!
---@meta

---@class RenderPassDesc
RenderPassDesc = {}

---@param label string
---@return RenderPassDesc
function RenderPassDesc.Create(label) end

-- Set color attachment `index` (0-3, contiguous). `r,g,b,a` is the clear
-- value, used only with `LoadOp.Clear`.
---@param index integer
---@param view TexView
---@param load LoadOp
---@param r number
---@param g number
---@param b number
---@param a number
function RenderPassDesc:color(index, view, load, r, g, b, a) end

-- Set the depth attachment. `d` is the clear value, used only with
-- `LoadOp.Clear`.
---@param view TexView
---@param load LoadOp
---@param d number
function RenderPassDesc:depth(view, load, d) end

-- Target the window's backbuffer (`width` x `height` pixels) instead of
-- texture attachments.
---@param width integer
---@param height integer
---@param load LoadOp
---@param r number
---@param g number
---@param b number
---@param a number
function RenderPassDesc:backbuffer(width, height, load, r, g, b, a) end

-- Load op for the backbuffer's depth buffer.
---@param load LoadOp
---@param d number
function RenderPassDesc:backbufferDepth(load, d) end

