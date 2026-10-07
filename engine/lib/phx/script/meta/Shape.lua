-- AUTO GENERATED. DO NOT MODIFY!
---@meta

-- The 2D primitives, each with its own pipeline (fragment shader, blend mode
-- baked in). Parameters (`a..d` of `Imm.Shape`):
-- 
-- | shape | a | b | c | d |
-- |---|---|---|---|---|
-- | Circle, Hex, Ring, RingGlow, RingDim | radius | | | |
-- | Panel | inner alpha | bevel | | |
-- | Wedge | r1 | r2 | to | tw |
-- | Annulus | inner radius | outer radius | | |
-- | Box, Grid, PanelGlow, Point, PointGlow | none | | | |
-- 
-- `Triangle` and `LineGlow*` have their own entry points (`Imm.TriGlow`,
-- `Imm.LineGlow`).
---@class Shape
---@field Solid integer 
---@field Image integer 
---@field Text integer 
---@field TextAdditive integer 
---@field Box integer 
---@field Circle integer 
---@field Grid integer 
---@field Hex integer 
---@field Icon integer 
---@field Panel integer 
---@field PanelGlow integer 
---@field Point integer 
---@field PointGlow integer 
---@field Ring integer 
---@field RingGlow integer 
---@field RingDim integer 
---@field Triangle integer 
---@field Wedge integer 
---@field Annulus integer 
---@field LineGlow integer 
Shape = {
    Solid = 0,
    Image = 1,
    Text = 2,
    TextAdditive = 3,
    Box = 4,
    Circle = 5,
    Grid = 6,
    Hex = 7,
    Icon = 8,
    Panel = 9,
    PanelGlow = 10,
    Point = 11,
    PointGlow = 12,
    Ring = 13,
    RingGlow = 14,
    RingDim = 15,
    Triangle = 16,
    Wedge = 17,
    Annulus = 18,
    LineGlow = 19,
}

