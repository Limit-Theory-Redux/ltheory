# Formations

The `Formations` module (`script/Modules/Formations`): ships that fly together on a leader, in a
shape. Patrol wings use it ([Factions](factions.md)); any code can make a formation.

## A formation

A formation is an entity (`FormationComponent`) with a shape and its ships in slot order: the first
leads, flies its own orders, and is never told where to go by the formation. Each other ship
(`FormationMemberComponent`) holds its slot with a navigation Follow order on the leader.

Shapes are data (`Shared/Definitions/FormationDefs.lua`): a function from the n-th wingman to its
slot in spacings (right, up, forward) along the leader's level axes. There are a wedge, a line
abreast, a column, an echelon and a diamond; a new shape is an entry there. The spacing is
`spacingRadii` (6) bounding radii of the formation's largest ship.

```lua
local wing = FormationSystem:create("Wedge", { leader, second, third })
Nav.order(leader, Nav.Orders.Patrol { points = route, loop = true })
FormationSystem:join(wing, fourth)       -- or join(wing, ship, slot)
FormationSystem:leave(second)
FormationSystem:isWingman(third)         -- true: its flight is the formation's
```

## Keeping it

`FormationSystem` tends every formation each `intervalS` (0.25 s):

- a member that is gone is dropped, and when the leader is gone the next ship leads;
- when the wing changed (a member, the leader, the shape) or a wingman's Follow has ended, the
  slots are given out again, as Follow orders with the slot's offset in metres;
- the leader is held to `paceShare` (0.5) of the slowest member's top speed (`Nav.setMaxSpeed`), so
  the outer slots have speed to spare in turns; the cap goes when it stops leading.

A wingman's Follow is a Routine order: an order of higher priority (a fight) takes over the ship, and
the Follow holds the slot again when it ends. How a wingman flies its slot (predictively, in the
leader's level frame) is the [Follow behaviour](../engine/nav.md).

## What keeps a wing tight

Measured in `Testbeds/Formation` (a leader on a looping patrol of 16 points and four wingmen in a
wedge; the log gives each wingman's distance from its slot in spacings):

- slots in the leader's level frame: a leader banking into a turn at several radians a second no
  longer flings the side slots up and down;
- prediction, and a gentle approach: the wingmen settle instead of swinging across their slots;
- a leader at half its top speed, on a route of many points: a sharp corner at top speed changes the
  leader's velocity faster than any wingman can follow.

`LTHEORY_FORMATION_SHAPE`, `LTHEORY_FORMATION_WINGMEN`, `LTHEORY_FORMATION_SPACING`,
`LTHEORY_FORMATION_PACE` and `LTHEORY_FORMATION_TRACE` set the testbed up and trace one wingman.
