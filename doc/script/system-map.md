# System maps

`SystemMap` is the top-down 2D map of the star system and `SystemMap3D` the 3D one (both in the CelestialObjects module). The autopilot's route is drawn on both by `RouteOverlayRenderSystem` (UI module), through a world-to-screen mapping that each map supplies, so the two maps and the flight view draw it the same way. The navigation terms are in the [glossary](nav/glossary.md).

## The route

`RouteOverlayRenderSystem:draw(overlay, toScreen, ctx)` takes an overlay table (`shipPos`, `targetPos`, `route`, `flown`, `progress`, `planning`) and draws:

- the target, the next waypoint, and the distance and ETA;
- the legs the ship has left to fly (`overlay.route`, from `Nav.route`), in white;
- the legs it has already passed (`overlay.flown`, from `Nav.routeFlown`), in a dimmer blue-grey, ending at the ship.

The passed legs are those of the current plan, so they start again when a new plan is made ([navigation orders](nav/navigation.md#queue-and-status)). `overlay.color` (a `Color`) draws the legs left to fly in it instead of white.

### Many ships

A map can show other ships too, through three fields of its state, which the caller sets:

- `ships`: a list of `{ entity, label, color }`. Each is a point in `color` (`{ r, g, b }`), and its label shows while it is selected. The label and the colour are read each frame, so a caller can change them.
- `routeOverlay`: one ship's full overlay, as above.
- `extraRoutes`: a list of overlays that `RouteOverlayRenderSystem:drawRoute` draws, which are only the passed legs and the legs left to fly, so that a map with many ships has no target and distance texts to read through. They are drawn under `routeOverlay`, so the selected ship's route lies on top.

`selectEntity(state, entity)` selects an entity's entry as a click on it would, so the caller can move the selection to a ship it picked elsewhere, and read `state.selected` to follow a click.

## Circles on the 2D map

Each body the system planner knows (star, planets, moons and stations, [system planner](nav/system-planner.md)) gets three circles under its point:

![A body's disc, the planner's clearance ring and its gravity well, drawn around its point](../images/system-map/circles.svg)

- **Disc**: the body's size, the bounding radius of its rigid body. It is the same radius the renderer and the planner use.
- **Red ring**: the distance the planner keeps a path outside of, `PlannerSnapshot.safeRadius`: the radius times `Config.game.autoPilot.safeRadiusFactor` for the body's kind. This is why a path goes around a body at some distance.
- **Blue ring**: the body's gravity well, the radius of its zone ([zones](zones.md)).

The rings are independent of one another, so a ring can lie inside another (a moon's well may be inside its red ring). A legend at the bottom left of the map names them.

A circle smaller than 2 px is not drawn, and a moon whose orbit is smaller than 20 px on screen is hidden with its circles. A circle whose quad would be more than four screens wide is not drawn; when the disc covers the whole map area, the area is filled instead, so a body does not vanish when the view is inside it.

## The map is a projection

The 2D map is an orthographic top-down view: world `x` and `z` give the screen position, and the height `y` is dropped. A disc is the body projected on the map plane, at the same scale as the distances. The ship is projected the same way.

![A ship above a planet is drawn inside the planet's disc, though it is farther from the planet than its radius](../images/system-map/projection.svg)

A ship above or below a body is therefore drawn over its disc while it is outside the body. The planar offset on the map is at most the 3D distance, and the two agree only when the ship is in the plane of the body. The map's selected-body panel gives the 3D distance (`Distance`) and the radius (`Radius`): when the distance is larger than the radius for a ship that the map shows inside the disc, the ship is above or below the body. `SystemMap3D` shows the real separation.
