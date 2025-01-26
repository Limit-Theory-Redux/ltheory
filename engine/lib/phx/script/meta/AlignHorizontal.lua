-- AUTO GENERATED. DO NOT MODIFY!
---@meta

-- Horizontal alignment of the container elements.
---@class AlignHorizontal
---@field Default integer Default alignment (Left). If element has default alignment then container's children alignment will be taken in account in layouting.
---@field Center integer Center element inside parent container.
---@field Left integer Align element to the left inside parent container.
---@field Right integer Align element to the right inside parent container.
---@field Expand integer Expand element horizontally inside parent container. Container with expand alignment will always fit its parent width. This is in contrast to stretch alignment in which case width can be bigger than the parent one.
---@field Stretch integer Stretch element horizontally inside parent container. Container with stretch alignment will grow in size to stick to the parent sides or to envelop it's children if they are bigger.
AlignHorizontal = {
    -- Default alignment (Left).
    -- If element has default alignment then container's children alignment will be taken in account in layouting.
    Default = 0,
    -- Center element inside parent container.
    Center = 1,
    -- Align element to the left inside parent container.
    Left = 2,
    -- Align element to the right inside parent container.
    Right = 3,
    -- Expand element horizontally inside parent container.
    -- Container with expand alignment will always fit its parent width.
    -- This is in contrast to stretch alignment in which case width can be bigger than the parent one.
    Expand = 4,
    -- Stretch element horizontally inside parent container.
    -- Container with stretch alignment will grow in size to stick to the parent sides or to envelop it's children if they are bigger.
    Stretch = 5,
}

