--[[----------------------------------------------------------------------------
  Base class for AI actions. Actions can be pushed and popped from action
  stacks on entities that have an 'Actions' component.

  clone           : Must return a copy of the action *in a clean state* (e.g.,
                    an attack action should return a clone that has the same
                    target, etc., but free of any bookeeping data that the
                    running instance may have accumulated).

  onUpdateActive  : Called only for the top action in the stack (the
                    'current' action)

  onUpdatePassive : Called for every action in the stack
----------------------------------------------------------------------------]]--

local Action = class(function (self) end)

-- Virtual ---------------------------------------------------------------------

function Action:clone ()
  assert(false, 'NYI @ Action.clone')
end

function Action:getName ()
  assert(false, 'NYI @ Action.getName')
end

function Action:onStart (e) end
function Action:onStop (e) end
function Action:onUpdateActive (e, dt) end
function Action:onUpdatePassive (e, dt) end

-- Helper ----------------------------------------------------------------------

local kLeadTime = 1
local expMap = Core.FFI.Math.ExpMap1Signed

-- TODO : This is a *major* bottleneck; AI steering / thrust controller needs
--        to be pushed to C. Probably pathing / nav grid as well?
function Action:flyToward (e, targetPos, targetForward, targetUp)
  local c = e:getThrustController()
  if not c then return end

  local ePos = e:getPos()
  local course = targetPos - e:getPos()
  local dist = course:length()
  if dist < 1e-6 then return end
  course = course - e:getVelocity():scale(kLeadTime)

  -- TODO : Fwd alignment can cause docking failure
  local forward  = (course + targetForward:scale(1.0)):normalize()
  local yawPitch = e:getForward():cross(forward)
  local roll     = e:getUp():cross(targetUp)

  -- TODO: Instead of reducing NPC ship max speed, need to tie speed and maneuverability
  --       to ship's mass (currently radius... or scale... or both....) and reduce pitch
  --       and yaw as z-axis (length) increases in the larger ship hulls.
  c.forward = expMap(  2.0 * e:getForward():dot(forward)) / 3
--  c.forward = expMap(  2.0 * e:getForward():dot(forward))
  c.right   = expMap(  2.0 * e:getRight():dot(forward))
  c.up      = expMap(  2.0 * e:getUp():dot(forward))
  c.yaw     = expMap( -1.0 * e:getUp():dot(yawPitch))
  c.pitch   = expMap(  1.0 * e:getRight():dot(yawPitch))
  c.roll    = expMap( -1.0 * e:getForward():dot(roll))

  if not e.travelDriveActive then
    if e == GameState.player.currentShip or e.usesBoost then
      c.boost = 0.0
      local newBoost = 1.0 - exp(-max(0.0, (dist / 150.0) - 1.0))
--    if newBoost > 0 and e:discharge(newBoost) then -- applies without normal boostCost (disabled for now)
      c.boost = newBoost
--    end
    end
  elseif e.travelDriveActive then
    c.boost = 3 -- could also draw lots of energy from capacitor here if we want that
  end
end

--------------------------------------------------------------------------------

return Action
