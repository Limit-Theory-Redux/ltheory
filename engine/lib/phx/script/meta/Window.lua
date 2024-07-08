---@meta

---@class Window
Window = {}

function Window:beginDraw() end

function Window:endDraw() end

-- The window title.
---@return string
function Window:title() end

-- Set the window title.
---@param title string
function Window:setTitle(title) end

-- The window cursor.
---@return Cursor
function Window:cursor() end

-- The window present mode.
---@return PresentMode
function Window:presentMode() end

-- Set window present mode.
---@param presentMode PresentMode
function Window:setPresentMode(presentMode) end

-- Setting this to true will attempt to maximize the window.
-- 
-- Setting it to false will attempt to un-maximize the window.
---@param maximized boolean
function Window:setMaximized(maximized) end

-- Setting this to true will attempt to minimize the window.
-- 
-- Setting it to false will attempt to un-minimize the window.
---@param minimized boolean
function Window:setMinimized(minimized) end

-- The window's client position in physical pixels.
-- 
-- See [`WindowPosition`] for an explanation about logical/physical sizes.
---@return Vec2i
function Window:position() end

-- Set the window's client position in the center of the current monitor.
function Window:setCenteredPosition() end

-- Set the window's client position in physical pixels.
-- 
-- See [`WindowPosition`] for an explanation about logical/physical sizes.
---@param x integer
---@param y integer
function Window:setPosition(x, y) end

-- The window's client area width in logical pixels.
-- 
-- See [`WindowResolution`] for an explanation about logical/physical sizes.
---@return number
function Window:width() end

-- The window's client area height in logical pixels.
-- 
-- See [`WindowResolution`] for an explanation about logical/physical sizes.
---@return number
function Window:height() end

-- The window's client area size in logical pixels.
-- 
-- See [`WindowResolution`] for an explanation about logical/physical sizes.
---@return Vec2f
function Window:size() end

-- Set the window's client area size in logical pixels.
-- 
-- See [`WindowResolution`] for an explanation about logical/physical sizes.
---@param width number
---@param height number
function Window:setSize(width, height) end

-- The window's client area width in physical pixels.
-- 
-- See [`WindowResolution`] for an explanation about logical/physical sizes.
---@return integer
function Window:physicalWidth() end

-- The window's client area height in physical pixels.
-- 
-- See [`WindowResolution`] for an explanation about logical/physical sizes.
---@return integer
function Window:physicalHeight() end

-- The window's client area size in physical pixels.
-- 
-- See [`WindowResolution`] for an explanation about logical/physical sizes.
---@return Vec2i
function Window:physicalSize() end

-- Set the window's client area size in physical pixels.
-- 
-- See [`WindowResolution`] for an explanation about logical/physical sizes.
---@param width integer
---@param height integer
function Window:setPhysicalSize(width, height) end

-- Is the window resizable?
---@return boolean
function Window:isResizable() end

-- Should the window be resizable?
---@param resizable boolean
function Window:setResizable(resizable) end

-- Has the window decorations?
---@return boolean
function Window:hasDecorations() end

-- Should the window have decorations?
---@param decorations boolean
function Window:setDecorations(decorations) end

-- Is the window focused?
---@return boolean
function Window:isFocused() end

-- Should the window be focused?
---@param focused boolean
function Window:setFocused(focused) end

---@param fs boolean
---@param exclusive boolean
function Window:setFullscreen(fs, exclusive) end

-- The window's scale factor.
-- 
-- Ratio of physical size to logical size, see [`WindowResolution`].
---@return number
function Window:scaleFactor() end

-- The cursor position in this window in logical pixels.
-- 
-- Returns `None` if the cursor is outside the window area.
-- 
-- See [`WindowResolution`] for an explanation about logical/physical sizes.
---@return Vec2f
function Window:cursorPosition() end

-- Set the cursor position in this window in logical pixels.
-- 
-- See [`WindowResolution`] for an explanation about logical/physical sizes.
---@param position Vec2f
function Window:setCursorPosition(position) end

-- The cursor position in this window in physical pixels.
-- 
-- Returns `None` if the cursor is outside the window area.
-- 
-- See [`WindowResolution`] for an explanation about logical/physical sizes.
---@return Vec2f
function Window:physicalCursorPosition() end

-- Set the cursor position in this window in physical pixels.
-- 
-- See [`WindowResolution`] for an explanation about logical/physical sizes.
---@param position Vec2d
function Window:setPhysicalCursorPosition(position) end

