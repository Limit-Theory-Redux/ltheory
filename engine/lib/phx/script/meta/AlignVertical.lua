-- AUTO GENERATED. DO NOT MODIFY!
---@meta

-- Vertical alignment of the container elements.
---@class AlignVertical
---@field Default integer Default alignment (Top). If element has default alignment then container's children alignment will be taken in account in layouting.
---@field Center integer Center element inside parent container.
---@field Top integer Align element to the top inside parent container.
---@field Bottom integer Align element to the bottom inside parent container.
---@field Expand integer Expand element vertically inside parent container. Container with expand alignment will always fit its parent height. This is in contrast to stretch alignment in which case height can be bigger than the parent one.
---@field Stretch integer Stretch element vertically inside parent container. Container with stretch alignment will grow in size to stick to the parent sides or to envelop it's children if they are bigger.
AlignVertical = {
    -- Default alignment (Top).
    -- If element has default alignment then container's children alignment will be taken in account in layouting.
    Default = 0,
    -- Center element inside parent container.
    Center = 1,
    -- Align element to the top inside parent container.
    Top = 2,
    -- Align element to the bottom inside parent container.
    Bottom = 3,
    -- Expand element vertically inside parent container.
    -- Container with expand alignment will always fit its parent height.
    -- This is in contrast to stretch alignment in which case height can be bigger than the parent one.
    Expand = 4,
    -- Stretch element vertically inside parent container.
    -- Container with stretch alignment will grow in size to stick to the parent sides or to envelop it's children if they are bigger.
    Stretch = 5,
}

