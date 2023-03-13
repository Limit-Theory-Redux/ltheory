-- TODO: Using requireAll locally like this is not recommended.
-- Implementation of requireEach needs to happen.
local Entities = requireAll('GameObjects.Entities')
local Actions = requireAll('GameObjects.Actions')
local TestEcon = require('States.Application')
local Production = require('Systems.Economy.Production')
local Item = require('Systems.Economy.Item')
local SystemMap = require('Systems.CommandView.SystemMap')

local rng = RNG.FromTime()
--local rng = RNG.Create(10) -- for when the same seed is needed

local kFields = 10
local kFieldCount = 300
local kStations = 22
local kPlayers = 3
local kAssets = 100

function TestEcon:getWindowMode ()
  return Bit.Or32(WindowMode.Shown, WindowMode.Resizable)
end

function TestEcon:getTitle ()
  return 'Economy Simulation'
end

function TestEcon:onInput ()
  self.canvas:input()
end

function TestEcon:onUpdate (dt)
  self.system:update(dt)
  self.canvas:update(dt)
end

function TestEcon:onDraw ()
  self.canvas:draw(self.resX, self.resY)
end

function TestEcon:showStatus ()
  -- Display statuses of players and assets
  self.system:register(Event.Debug, function (system, state)
    local ctx = state.context
    ctx:text("AI Players:")
    for i, v in ipairs(system.players) do
      ctx:indent()
      ctx:text("[%d] %s : %d credits", i, v:getName(), v:getCredits())
      ctx:indent()
      ctx:text("Actions:")
      ctx:indent()
      for j, a in ipairs(v.actions) do
        ctx:text("%d : %s", j, a:getName())
      end
      ctx:undent()
      ctx:undent()
      if kAssets <= 3 then -- only show asset details if there are a handful for testing
        ctx:indent()
        ctx:text("Assets:") -- TODO: show number of active assets / total assets owned by this player
        ctx:indent()
        for asset in v:iterAssets() do
          ctx:text("%s", asset:getName())
          ctx:indent()
          ctx:text("Actions:")
          ctx:indent()
          for j, a in ipairs(asset.actions) do
            ctx:text("%d : %s", j, a:getName())
          end
          ctx:undent()
          ctx:undent()
        end
        ctx:undent()
        ctx:undent()
      end
      ctx:undent()
    end
  end)
end

function TestEcon:onInit ()
  -- Generate new universe for economic testing
  self.canvas = UI.Canvas()
  self.system = Entities.Test.System(rng:get64())

  -- Add system-wide AI director
  self.tradeAI = Entities.Player("AI Trade Player")
  self.tradeAI:addCredits(1e10)

  -- Add a generic ship-like entity to serve as the imaginary player ship
  self.tradeShip = Entity()
  self.tradeShip:setOwner(self.tradeAI)

  -- Use fast movement and hyperspeedup for economic testing
  Config.debug.instantJobs     = true
  Config.debug.timeAccelFactor = 100

  -- Add a planet at the origin
  local planet = self.system:spawnPlanet(false)
  planet:setPos(Vec3f(0, 0, 0)) -- move planet to origin

  -- Add Asteroid Field (and Asteroid) objects
  for i = 1, kFields do self.system:spawnAsteroidField(kFieldCount, false) end

  -- Add Station objects
  -- Must have one "free" solar energy generating station per star system
  local newStation = self.system:spawnStation(self.tradeAI, Production.Solar())
  newStation:setPos(rng:getDir3():scale(1.0 * Config.gen.scaleSystem * (1 + rng:getExp()))) -- move station
  for i = 1, 200 do
    -- Add some units of Energy for sale as a starting inventory
    -- (Be aware they will immediately be removed and accounted for in the Asks escrow counter)
    newStation:addItem(Item.Energy, 1)
    newStation.trader:addAsk(Item.Energy, math.floor(Item.Energy.energy * Config.econ.markup))
  end

  -- Now maybe add some additional stations
  for i = 2, kStations do
    -- Create a station, owned by this system's AI player, within a random AsteroidField Zone
    local newStation = self.system:spawnStation(self.tradeAI, nil)

    -- Assign the new Station to a randomly-selected AI player/owner
    -- TODO: figure out the nasty infinite loop when assigning station to a non-system-level AI player
    local ownerNum = rng:getInt(1, kPlayers)
    for i, v in ipairs(self.system.players) do
      if i == ownerNum then
        printf("New station %s should have owner %s", newStation:getName(), v:getName())
--        newStation:setOwner(v) -- causes an infinite loop somewhere
        printf("New station %s actually has owner %s", newStation:getName(), newStation:getOwner():getName())
        break
      end
    end
  end

  -- Add Players and give each one some assets
  for i = 1, kPlayers do
    local tradePlayerName = format("AI Trade Player %d", i)
    local tradePlayer = Entities.Player(tradePlayerName)

    -- Give player some starting money
--    tradePlayer:addItem(Item.Credit, Config.game.eStartCredits)
    tradePlayer:addCredits(Config.game.eStartCredits)

    -- Create assets (ships)
    self.system:spawnAI(kAssets, Actions.Wait(1), tradePlayer)
    printf("%d assets added to %s", kAssets, tradePlayerName)

    -- Configure assets
    for asset in tradePlayer:iterAssets() do
      self.system:place(rng, asset)
    end

    -- Tell AI player to start using the Think action
    tradePlayer:pushAction(Actions.Think())
  end

  self.canvas:add(SystemMap(self.system))

  TestEcon:showStatus()
end

return TestEcon
