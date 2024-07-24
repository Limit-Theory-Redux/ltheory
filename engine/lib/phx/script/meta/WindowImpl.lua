---@meta

---@class WindowImpl
WindowImpl = {}

function WindowImpl:beginDraw() end

function WindowImpl:endDraw() end

-- The window title.
---@return string
function WindowImpl:title() end

-- Set the window title.
---@param title string
function WindowImpl:setTitle(title) end

-- The window cursor.
---@return Cursor
function WindowImpl:cursor() end

-- The window present mode.
---@return PresentMode
function WindowImpl:presentMode() end

-- Set window present mode.
---@param presentMode PresentMode
function WindowImpl:setPresentMode(presentMode) end

-- Setting this to true will attempt to maximize the window.
-- 
-- Setting it to false will attempt to un-maximize the window.
---@param maximized boolean
function WindowImpl:setMaximized(maximized) end

-- Setting this to true will attempt to minimize the window.
-- 
-- Setting it to false will attempt to un-minimize the window.
---@param minimized boolean
function WindowImpl:setMinimized(minimized) end

-- The window's client position in physical pixels.
-- 
-- See [`WindowPosition`] for an explanation about logical/physical sizes.
---@return Vec2i
function WindowImpl:position() end

-- Set the window's client position in the center of the current monitor.
function WindowImpl:setCenteredPosition() end

-- Set the window's client position in physical pixels.
-- 
-- See [`WindowPosition`] for an explanation about logical/physical sizes.
---@param x integer
---@param y integer
function WindowImpl:setPosition(x, y) end

-- The window's client area width in logical pixels.
-- 
-- See [`WindowResolution`] for an explanation about logical/physical sizes.
---@return number
function WindowImpl:width() end

-- The window's client area height in logical pixels.
-- 
-- See [`WindowResolution`] for an explanation about logical/physical sizes.
---@return number
function WindowImpl:height() end

-- The window's client area size in logical pixels.
-- 
-- See [`WindowResolution`] for an explanation about logical/physical sizes.
---@return Vec2f
function WindowImpl:size() end

-- Set the window's client area size in logical pixels.
-- 
-- See [`WindowResolution`] for an explanation about logical/physical sizes.
---@param width number
---@param height number
function WindowImpl:setSize(width, height) end

-- The window's client area width in physical pixels.
-- 
-- See [`WindowResolution`] for an explanation about logical/physical sizes.
---@return integer
function WindowImpl:physicalWidth() end

-- The window's client area height in physical pixels.
-- 
-- See [`WindowResolution`] for an explanation about logical/physical sizes.
---@return integer
function WindowImpl:physicalHeight() end

-- The window's client area size in physical pixels.
-- 
-- See [`WindowResolution`] for an explanation about logical/physical sizes.
---@return Vec2i
function WindowImpl:physicalSize() end

-- Set the window's client area size in physical pixels.
-- 
-- See [`WindowResolution`] for an explanation about logical/physical sizes.
---@param width integer
---@param height integer
function WindowImpl:setPhysicalSize(width, height) end

-- Is the window resizable?
---@return boolean
function WindowImpl:isResizable() end

-- Should the window be resizable?
---@param resizable boolean
function WindowImpl:setResizable(resizable) end

-- Has the window decorations?
---@return boolean
function WindowImpl:hasDecorations() end

-- Should the window have decorations?
---@param decorations boolean
function WindowImpl:setDecorations(decorations) end

-- Is the window focused?
---@return boolean
function WindowImpl:isFocused() end

-- Should the window be focused?
---@param focused boolean
function WindowImpl:setFocused(focused) end

---@param fs boolean
---@param exclusive boolean
function WindowImpl:setFullscreen(fs, exclusive) end

-- The window's scale factor.
-- 
-- Ratio of physical size to logical size, see [`WindowResolution`].
---@return number
function WindowImpl:scaleFactor() end

-- The cursor position in this window in logical pixels.
-- 
-- Returns `None` if the cursor is outside the window area.
-- 
-- See [`WindowResolution`] for an explanation about logical/physical sizes.
---@return Vec2f
function WindowImpl:cursorPosition() end

-- Set the cursor position in this window in logical pixels.
-- 
-- See [`WindowResolution`] for an explanation about logical/physical sizes.
---@param position Vec2f
function WindowImpl:setCursorPosition(position) end

-- The cursor position in this window in physical pixels.
-- 
-- Returns `None` if the cursor is outside the window area.
-- 
-- See [`WindowResolution`] for an explanation about logical/physical sizes.
---@return Vec2f
function WindowImpl:physicalCursorPosition() end

-- Set the cursor position in this window in physical pixels.
-- 
-- See [`WindowResolution`] for an explanation about logical/physical sizes.
---@param position Vec2d
function WindowImpl:setPhysicalCursorPosition(position) end

