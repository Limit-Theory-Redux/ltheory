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
    HmGuiInstance:beginWindow('HmGui Test', InputInstance)
    HmGuiInstance:beginGroupX()
    HmGuiInstance:button(" < ")
    HmGuiInstance:setStretch(0, 1)
    HmGuiInstance:button("Tab1")
    HmGuiInstance:button("Tab2")
    HmGuiInstance:button("Tab3")
    HmGuiInstance:button(" > ")
    HmGuiInstance:setStretch(0, 1)
    HmGuiInstance:endGroup()
    HmGuiInstance:setStretch(1, 1)

    HmGuiInstance:beginGroupX()
    HmGuiInstance:beginGroupY()
    HmGuiInstance:setPadding(4, 4)
    HmGuiInstance:text("Welcome to...")
    HmGuiInstance:setAlign(0.5, 0.5)

    HmGuiInstance:pushTextColor(1.0, 0.0, 0.3, 1.0)
    HmGuiInstance:pushFont(Cache.Font("Exo2Bold", 30))
    HmGuiInstance:text("~ Hybrid Mode ~")
    HmGuiInstance:popStyle(2)
    HmGuiInstance:setAlign(0.5, 0.5)

    HmGuiInstance:text("GUI!")
    HmGuiInstance:setAlign(0.5, 0.5)
    HmGuiInstance:button("Not-So-Stretchy")
    HmGuiInstance:setStretch(1, 0)
    HmGuiInstance:button("Stretchy")
    HmGuiInstance:setStretch(1, 1)

    HmGuiInstance:beginGroupX()
    for i = 1, 3 do
        HmGuiInstance:beginGroupY()
        for j = 1, 3 do
            HmGuiInstance:button(":)")
        end
        HmGuiInstance:endGroup()
        HmGuiInstance:setStretch(1, 1)
    end
    HmGuiInstance:endGroup()
    HmGuiInstance:setStretch(1, 1)
    HmGuiInstance:endGroup()
    HmGuiInstance:setAlign(0, 0.0)
    HmGuiInstance:setStretch(1, 1)

    HmGuiInstance:beginGroupY()
    HmGuiInstance:setPadding(4, 4)
    if HmGuiInstance:button("-- OPT 1 --") then Log.Debug("Opt 1!") end
    HmGuiInstance:button("-- OPT 2 --")
    HmGuiInstance:checkbox("Yas", true)
    HmGuiInstance:checkbox("Nope", false)
    HmGuiInstance:checkbox("Possibly?", false)
    HmGuiInstance:button("DONE")
    HmGuiInstance:endGroup()
    HmGuiInstance:setAlign(0, 1.0)
    HmGuiInstance:setStretch(1, 1)

    HmGuiInstance:beginGroupY()
    HmGuiInstance:setPadding(4, 4)
    for i = 1, 9 do
        HmGuiInstance:beginGroupX()
        for j = 1, i do
            HmGuiInstance:button(format("%d.%d", i, j))
        end
        HmGuiInstance:endGroup()
        HmGuiInstance:setAlign(0.5, 0.5)
    end
    HmGuiInstance:endGroup()
    self:showTodoInner()
    HmGuiInstance:endGroup()
    HmGuiInstance:setStretch(1, 0)

    HmGuiInstance:text("Behold, the codez! \\o/")
    HmGuiInstance:beginGroupX()
    for i = 1, 2 do
        HmGuiInstance:beginScroll(200)
        HmGuiInstance:pushTextColor(0.1, 0.5, 1.0, 1.0)
        HmGuiInstance:pushFont(Cache.Font('FiraMono', 10))
        local lines = code:split('\n')
        for _, line in ipairs(lines) do
            HmGuiInstance:text(line)
        end
        HmGuiInstance:popStyle(2)
        HmGuiInstance:endScroll(InputInstance)
    end
    HmGuiInstance:endGroup()
    HmGuiInstance:endWindow()
    HmGuiInstance:setAlign(0.5, 0.5)
end

function Test:showTodoInner()
    HmGuiInstance:beginScroll(256)
    HmGuiInstance:setSpacing(8)
    for _, group in ipairs(todo) do
        HmGuiInstance:textEx(Cache.Font('Rajdhani', 18), group.name, 1, 1, 1, 1)
        HmGuiInstance:beginGroupY()
        HmGuiInstance:setSpacing(2)
        HmGuiInstance:setPaddingLeft(12)
        for _, v in ipairs(group.elems) do
            v[2] = HmGuiInstance:checkbox(v[1], v[2])
        end
        HmGuiInstance:endGroup()
    end
    HmGuiInstance:endScroll(InputInstance)
end

function Test:showTodo()
    HmGuiInstance:beginWindow("HmGui Todo List", InputInstance)
    HmGuiInstance:textEx(Cache.Font('Iceland', 20), 'HmGui Todo List', 0.3, 0.4, 0.5, 0.5)
    HmGuiInstance:setAlign(0.5, 0.5)
    self:showTodoInner()
    HmGuiInstance:endWindow()
    HmGuiInstance:setAlign(0.5, 0.5)
end

function Test:showMetrics()
    HmGuiInstance:beginWindow("Metrics", InputInstance)
    HmGuiInstance:text(format("fps: %.2f", 1.0 / self.dt))
    HmGuiInstance:endWindow()
end

function Test:onUpdate(dt)
    Profiler.Begin('HmGuiInstance:update')
    HmGuiInstance:beginGui(self.resX, self.resY, InputInstance)
    -- HmGuiInstance:image(self.bg)
    self:showSimple()
    -- self:showMetrics()
    self:showTodo()
    HmGuiInstance:endGui(InputInstance)
    Profiler.End()
end

function Test:onDraw()
    if useRenderer then
        self.renderer:start(self.resX, self.resY)
        Viewport.Push(0, 0, self.resX, self.resY, true)
        HmGuiInstance:draw()
        Viewport.Pop()
        self.renderer:stop()
        self.renderer:present(0, 0, self.resX, self.resY)
    else
        HmGuiInstance:draw()
    end
end

return Test
