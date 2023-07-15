ffi.cdef [[
  typedef struct _NodeT {
    int a;
    int b;
  } _NodeT;
]]

<<<<<<< HEAD
local function sb(a, b)
    local self = ffi.cast('_NodeT*', Memory.Alloc(ffi.sizeof('_NodeT')))
    self.a = a
    self.b = b
    return self
end

local function testStrMap()
    local map = StrMap.Create(16)
    local k = 1999
    for i = 1, k do
        map:set(format('key_%d', i), sb(i, i * i))
    end

    for i = 1, k do
        local x = ffi.cast('_NodeT*', map:get(format('key_%d', i)))
        assert(x.a * x.a == x.b)
    end

    local it = map:iterate()
    while it:hasMore() do
        Memory.Free(it:getValue())
        it:advance()
    end
    it:free()

    map:free()
end

while true do
    testStrMap()
=======
local function sb (a, b)
  local self = ffi.cast('_NodeT*', Memory.Alloc(ffi.sizeof('_NodeT')))
  self.a = a
  self.b = b
  return self
end

local function testStrMap ()
  local map = StrMap.Create(16)
  local k = 1999
  for i = 1, k do
    map:set(format('key_%d', i), sb(i, i*i))
  end

  for i = 1, k do
    local x = ffi.cast('_NodeT*', map:get(format('key_%d', i)))
    assert(x.a * x.a == x.b)
  end

  local it = map:iterate()
  while it:hasMore() do
    Memory.Free(it:getValue())
    it:advance()
  end
  it:free()

  map:free()
end

while true do
  testStrMap()
>>>>>>> 1b58bb0278295d31845972084d1313877cd21e29
end

os.exit(0)
