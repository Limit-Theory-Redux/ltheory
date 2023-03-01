local istype = ffi.istype
local Words = {}

local cons = Distribution()
cons:add('b', 1.5)
cons:add('c', 2.8)
cons:add('d', 4.3)
cons:add('f', 2.2)
cons:add('g', 2.0)
cons:add('h', 6.1)
cons:add('j', 0.2)
cons:add('k', 0.8)
cons:add('l', 4.0)
cons:add('m', 2.4)
cons:add('n', 6.7)
cons:add('p', 1.9)
cons:add('q', 0.1)
cons:add('r', 6.0)
cons:add('s', 6.3)
cons:add('t', 9.1)
cons:add('v', 1.0)
cons:add('w', 2.4)
cons:add('x', 0.2)
cons:add('z', 0.1)

cons:add('ll', 0.4)
cons:add('ss', 0.6)
cons:add('tt', 0.9)
cons:add('ff', 0.2)
cons:add('rr', 0.6)
cons:add('nn', 0.6)
cons:add('pp', 0.2)
cons:add('cc', 0.3)

local vowels = Distribution()
vowels:add('a',  8.2)
vowels:add('e', 12.7)
vowels:add('i',  7.0)
vowels:add('o',  7.5)
vowels:add('u',  2.8)
vowels:add('y',  2.0)

vowels:add('ee',  1.2)
vowels:add('oo',  0.7)

function Words.genName (rng)
  -- Word generator
  local name = {}
  for i = 1, rng:getInt(2, 5) do
    insert(name, cons:sample(rng))
    insert(name, vowels:sample(rng))
  end
  name[1] = name[1]:upper()
  name = join(name)
  return name
end

function Words.getCoolName (rngv)
  -- Create an object name that, if the first name is short, adds a second name for presumed uniqueness
  local name  = Words.genName(rngv)
  local name2 = Words.genName(rngv)
  if name:len() < 7 and rngv:getInt(0, 100) < (40 + ((7 - name:len()) * 20)) then
    name = name .. " " .. name2
  end
  return name
end

return Words
