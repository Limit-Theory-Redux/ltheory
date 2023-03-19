--[[ TODO ----------------------------------------------------------------------
  - Generators should output ShipTypes, not just meshes (so that generating
    algorithm can select mount points, etc)
  - Now that entities are well-unified, these metatypes should be unified as
    well; we only need one 'prototype' class.
----------------------------------------------------------------------------]]--

local Ship = require('GameObjects.Entities.Ship.Ship')
local ShipI = require('GameObjects.Entities.Ship.ShipInvis')
local SocketType = require('GameObjects.Entities.Ship.SocketType')

local ShipType = class(function (self, seed, generator, scale)
  local rng = RNG.Create(seed)
  self.seed = seed
--printf("new ship seed = %d", seed)
  self.mesh = generator(seed, Config.gen.shipRes):managed()
  self.bsp = BSP.Create(self.mesh):managed()
  self.scale = scale

  self.sockets = {
    [SocketType.Thruster] = {},
    [SocketType.Turret] = {},
  }

  for i = 1, Config.gen.nTurrets do
    local p = Gen.GenUtil.FindMountPoint(self.mesh, self.bsp, rng, Vec3f(0, 1, 0), Vec3f(0, 0, 1), 1000)
    if p then
      insert(self.sockets[SocketType.Turret], p * Vec3f( 1, 1, 1))
      insert(self.sockets[SocketType.Turret], p * Vec3f(-1, 1, 1))
    end
  end

  for i = 1, Config.gen.nThrusters do
    local p = Gen.GenUtil.FindMountPoint(self.mesh, self.bsp, rng, Vec3f(0, 0, -1), Vec3f(0, 0, -1), 1000)
    if p then
      insert(self.sockets[SocketType.Thruster], p * Vec3f( 1, 1, 1))
      insert(self.sockets[SocketType.Thruster], p * Vec3f(-1, 1, 1))
    end
  end

  rng:free()
end)

function ShipType:instantiate ()
  if Config.game.gameMode == 1 then
    return ShipI(self) -- enable game startup mode
  else
    return Ship(self) -- enable flight mode
  end
end

return ShipType
