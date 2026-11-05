local ConstructComponents = require("Modules.Constructs.Components")

---Connects ships to a `NavWorld`: registers a ship by its flight profile, and writes an agent's
---control axes into the ship's control input.
---@class NavAgentHelper
local NavAgentHelper = {}

-- Positions of the control axes in an agent's slice of the axes buffer, as Lua numbers.
local AXIS_PITCH, AXIS_YAW, AXIS_ROLL = tonumber(NavAxis.Pitch), tonumber(NavAxis.Yaw), tonumber(NavAxis.Roll)
local AXIS_FORWARD, AXIS_RIGHT, AXIS_UP =
    tonumber(NavAxis.ThrustForward), tonumber(NavAxis.ThrustRight), tonumber(NavAxis.ThrustUp)

---Register the ship's body as an agent of `nav`, with the drags, thrusts and torques of its flight
---profile. The body must stay in the `NavWorld`'s physics world while the `NavWorld` steps.
---@param nav NavWorld
---@param shipEntity Entity
---@param rb RigidBody
---@return integer agent The agent's id in `nav`
function NavAgentHelper.addShip(nav, shipEntity, rb)
    local profile = ConstructComponents.FlightProfile.ProfileOf(shipEntity)
    return nav:addAgent(
        rb,
        profile.linearDrag, profile.angularDrag,
        profile.thrustForward, profile.thrustRight, profile.thrustUp,
        profile.torquePitch, profile.torqueYaw, profile.torqueRoll
    )
end

---Copy one agent's slice of the buffer that `NavWorld:copyAxes` filled into a control input.
---@param axes ffi.cdata* The `copyAxes` buffer: `NavWorld.AxisCount()` doubles per agent
---@param agent integer The agent's id
---@param input ControlInputComponent
function NavAgentHelper.writeAxes(axes, agent, input)
    local base = agent * NavWorld.AxisCount()
    input:setPitch(axes[base + AXIS_PITCH])
    input:setYaw(axes[base + AXIS_YAW])
    input:setRoll(axes[base + AXIS_ROLL])
    input:setThrustForward(axes[base + AXIS_FORWARD])
    input:setThrustRight(axes[base + AXIS_RIGHT])
    input:setThrustUp(axes[base + AXIS_UP])
end

return NavAgentHelper
