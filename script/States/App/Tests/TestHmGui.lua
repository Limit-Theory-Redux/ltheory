local Test = require('States.Application')
local rng = RNG.FromTime()

local useRenderer = true

local todo = {
    {
        name = "Basic Widgets",
        elems = {
            { "Text",         true },
            { "Buttons",      true },
            { "Checkboxes",   true },
            { "Sliders",      false },
            { "Radio Groups", false },
            { "Selectable",   false },
            { "Tooltips",     false },
        },
    },

    {
        name = "Layout",
        elems = {
            { "Deferred Layout Pass", true },
            { "Horizontal Groups",    true },
            { "Vertical Groups",      true },
            { "Stacked Groups",       true },
            { "Group Padding",        true },
            { "Group Margins",        false },
            { "Child Spacing ",       true },
            { "ScrollViews",          true },
            { "Windows",              true },
        },
    },

    {
        name = "Input",
        elems = {
            { "Clip Groups",         true },
            { "Input Clipping",      true },
            { "Sticky Drag",         false },
            { "Keyboard Navigation", false },
            { "Gamepad Navigation",  false },
        },
    },

    {
        name = "Draw",
        elems = {
            { "Draw Layers",         true },
            { "Shader-Based Render", true },
            { "Glyph Render",        false },
        },
    },

    {
        name = "Technical",
        elems = {
            { "Hash Storage",              true },
            { "Hash Storage Invalidation", false },
            { "Deferred Metrics",          true },
        },
    },
}

function Test:onInit()
    -- self.bg = Tex2D.Load('./screenshot/wp2.png')
    self.renderer = Renderer()
end

function Test:onInput() end

local code = [[
static void MemPool_Grow (MemPool* self) {
  uint16 newBlockIndex = self->blockCount++;
  self->capacity += self->blockSize;

  /* Grow the list of pool blocks. */
  self->blocks = (void**)MemRealloc(self->blocks, self->blockCount * sizeof(void*));

  /* Allocate a new block and place at the back of the list. */
  void* newBlock = MemAlloc(self->cellSize * self->blockSize);
  self->blocks[newBlockIndex] = newBlock;

  /* Wire up the free list for this block. Note that we can assume the existing
   * free list is empty if the pool is requesting to grow, hence we overwrite
   * the existing list head. The block's initial freelist is wired sequentially
   * ( 0 -> 1 -> 2 ) for optimal cache locality. */
  void** prev = &self->freeList;
  char* pCurr = (char*)newBlock;
  for (uint32 i = 0; i < self->blockSize; ++i) {
    *prev = (void*)pCurr;
    prev = (void**)pCurr;
    pCurr += self->cellSize;
  }
  *prev = 0;
}
]]

function Test:showSimple()
    Gui:beginWindow('HmGui Test', InputInstance)
    Gui:beginHorizontalContainer()
    Gui:button(" < ")
    Gui:setStretch(0, 1)
    Gui:button("Tab1")
    Gui:button("Tab2")
    Gui:button("Tab3")
    Gui:button(" > ")
    Gui:setStretch(0, 1)
    Gui:endContainer()
    Gui:setStretch(1, 1)

    Gui:beginHorizontalContainer()
    Gui:beginVerticalContainer()
    Gui:setPadding(4, 4)
    Gui:text("Welcome to...")
    Gui:setAlign(0.5, 0.5)

    Gui:pushTextColor(1.0, 0.0, 0.3, 1.0)
    Gui:pushFont(Cache.Font("Exo2Bold", 30))
    Gui:text("~ Hybrid Mode ~")
    Gui:popStyle(2)
    Gui:setAlign(0.5, 0.5)

    Gui:text("GUI!")
    Gui:setAlign(0.5, 0.5)
    Gui:button("Not-So-Stretchy")
    Gui:setStretch(1, 0)
    Gui:button("Stretchy")
    Gui:setStretch(1, 1)

    Gui:beginHorizontalContainer()
    for i = 1, 3 do
        Gui:beginVerticalContainer()
        for j = 1, 3 do
            Gui:button(":)")
        end
        Gui:endContainer()
        Gui:setStretch(1, 1)
    end
    Gui:endContainer()
    Gui:setStretch(1, 1)
    Gui:endContainer()
    Gui:setAlign(0, 0.0)
    Gui:setStretch(1, 1)

    Gui:beginVerticalContainer()
    Gui:setPadding(4, 4)
    if Gui:button("-- OPT 1 --") then Log.Debug("Opt 1!") end
    Gui:button("-- OPT 2 --")
    Gui:checkbox("Yas", true)
    Gui:checkbox("Nope", false)
    Gui:checkbox("Possibly?", false)
    Gui:button("DONE")
    Gui:endContainer()
    Gui:setAlign(0, 1.0)
    Gui:setStretch(1, 1)

    Gui:beginVerticalContainer()
    Gui:setPadding(4, 4)
    for i = 1, 9 do
        Gui:beginHorizontalContainer()
        for j = 1, i do
            Gui:button(format("%d.%d", i, j))
        end
        Gui:endContainer()
        Gui:setAlign(0.5, 0.5)
    end
    Gui:endContainer()
    self:showTodoInner()
    Gui:endContainer()
    Gui:setStretch(1, 0)

    Gui:text("Behold, the codez! \\o/")
    Gui:beginHorizontalContainer()
    for i = 1, 2 do
        Gui:beginScroll(200)
        Gui:pushTextColor(0.1, 0.5, 1.0, 1.0)
        Gui:pushFont(Cache.Font('FiraMono', 10))
        local lines = code:split('\n')
        for _, line in ipairs(lines) do
            Gui:text(line)
        end
        Gui:popStyle(2)
        Gui:endScroll(InputInstance)
    end
    Gui:endContainer()
    Gui:endWindow()
    Gui:setAlign(0.5, 0.5)
end

function Test:showTodoInner()
    Gui:beginScroll(256)
    Gui:setSpacing(8)
    for _, group in ipairs(todo) do
        Gui:textEx(Cache.Font('Rajdhani', 18), group.name, 1, 1, 1, 1)
        Gui:beginVerticalContainer()
        Gui:setSpacing(2)
        Gui:setPaddingLeft(12)
        for _, v in ipairs(group.elems) do
            v[2] = Gui:checkbox(v[1], v[2])
        end
        Gui:endContainer()
    end
    Gui:endScroll(InputInstance)
end

function Test:showTodo()
    Gui:beginWindow("HmGui Todo List", InputInstance)
    Gui:textEx(Cache.Font('Iceland', 20), 'HmGui Todo List', 0.3, 0.4, 0.5, 0.5)
    Gui:setAlign(0.5, 0.5)
    self:showTodoInner()
    Gui:endWindow()
    Gui:setAlign(0.5, 0.5)
end

function Test:showMetrics()
    Gui:beginWindow("Metrics", InputInstance)
    Gui:text(format("fps: %.2f", 1.0 / self.dt))
    Gui:endWindow()
end

function Test:onUpdate(dt)
    Profiler.Begin('Gui:update')
    Gui:beginGui(self.resX, self.resY, InputInstance)
    -- Gui:image(self.bg)
    self:showSimple()
    -- self:showMetrics()
    self:showTodo()
    Gui:endGui(InputInstance)
    Profiler.End()
end

function Test:onDraw()
    if useRenderer then
        self.renderer:start(self.resX, self.resY)
        Viewport.Push(0, 0, self.resX, self.resY, true)
        Gui:draw()
        Viewport.Pop()
        self.renderer:stop()
        self.renderer:present(0, 0, self.resX, self.resY)
    else
        Gui:draw()
    end
end

return Test
