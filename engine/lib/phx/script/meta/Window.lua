---@meta

Window = Window

function Window:beginDraw(self) end

function Window:endDraw(self) end

---The window title.
---@return string
function Window:title(self) end

---Set the window title.
---@param title string
function Window:setTitle(self, title) end

---The window cursor.
---@return Cursor
function Window:cursor(self) end

---The window present mode.
---@return PresentMode
function Window:presentMode(self) end

---Set window present mode.
---@param present_mode PresentMode
function Window:setPresentMode(self, present_mode) end

---Setting this to true will attempt to maximize the window.
---
---Setting it to false will attempt to un-maximize the window.
---@param maximized boolean
function Window:setMaximized(self, maximized) end

---Setting this to true will attempt to minimize the window.
---
---Setting it to false will attempt to un-minimize the window.
---@param minimized boolean
function Window:setMinimized(self, minimized) end

---The window's client position in physical pixels.
---
---See [`WindowPosition`] for an explanation about logical/physical sizes.
---@return IVec2
function Window:position(self) end

---Set the window's client position in the center of the current monitor.
function Window:setCenteredPosition(self) end

---Set the window's client position in physical pixels.
---
---See [`WindowPosition`] for an explanation about logical/physical sizes.
---@param x integer
---@param y integer
function Window:setPosition(self, x, y) end

---The window's client area width in logical pixels.
---
---See [`WindowResolution`] for an explanation about logical/physical sizes.
---@return number
function Window:width(self) end

---The window's client area height in logical pixels.
---
---See [`WindowResolution`] for an explanation about logical/physical sizes.
---@return number
function Window:height(self) end

---The window's client area size in logical pixels.
---
---See [`WindowResolution`] for an explanation about logical/physical sizes.
---@return Vec2
function Window:size(self) end

---Set the window's client area size in logical pixels.
---
---See [`WindowResolution`] for an explanation about logical/physical sizes.
---@param width number
---@param height number
function Window:setSize(self, width, height) end

---The window's client area width in physical pixels.
---
---See [`WindowResolution`] for an explanation about logical/physical sizes.
---@return integer
function Window:physicalWidth(self) end

---The window's client area height in physical pixels.
---
---See [`WindowResolution`] for an explanation about logical/physical sizes.
---@return integer
function Window:physicalHeight(self) end

---The window's client area size in physical pixels.
---
---See [`WindowResolution`] for an explanation about logical/physical sizes.
---@return IVec2
function Window:physicalSize(self) end

---Set the window's client area size in physical pixels.
---
---See [`WindowResolution`] for an explanation about logical/physical sizes.
---@param width integer
---@param height integer
function Window:setPhysicalSize(self, width, height) end

---Is the window resizable?
---@return boolean
function Window:isResizable(self) end

---Should the window be resizable?
---@param resizable boolean
function Window:setResizable(self, resizable) end

---Has the window decorations?
---@return boolean
function Window:hasDecorations(self) end

---Should the window have decorations?
---@param decorations boolean
function Window:setDecorations(self, decorations) end

---Is the window transparent?
---@return boolean
function Window:isTransparent(self) end

---Should the window be transparent?
---@param transparent boolean
function Window:setTransparent(self, transparent) end

---Is the window focused?
---@return boolean
function Window:isFocused(self) end

---Should the window be focused?
---@param focused boolean
function Window:setFocused(self, focused) end

---@param fs boolean
function Window:setFullscreen(self, fs) end

function Window:toggleFullscreen(self) end

---The window's scale factor.
---
---Ratio of physical size to logical size, see [`WindowResolution`].
---@return number
function Window:scaleFactor(self) end

---The cursor position in this window in logical pixels.
---
---Returns `None` if the cursor is outside the window area.
---
---See [`WindowResolution`] for an explanation about logical/physical sizes.
---@return Vec2
function Window:cursorPosition(self) end

---Set the cursor position in this window in logical pixels.
---
---See [`WindowResolution`] for an explanation about logical/physical sizes.
---@param position Vec2
function Window:setCursorPosition(self, position) end

---The cursor position in this window in physical pixels.
---
---Returns `None` if the cursor is outside the window area.
---
---See [`WindowResolution`] for an explanation about logical/physical sizes.
---@return Vec2
function Window:physicalCursorPosition(self) end

---Set the cursor position in this window in physical pixels.
---
---See [`WindowResolution`] for an explanation about logical/physical sizes.
---@param position DVec2
function Window:setPhysicalCursorPosition(self, position) end

