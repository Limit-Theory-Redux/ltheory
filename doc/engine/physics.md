# Physics

`Physics` is a world of `RigidBody`s simulated with Rapier in double precision. Positions are `Position`s (f64), so queries keep ship-scale precision anywhere in a star system. Code: `engine/lib/phx/src/physics/`.

## Queries

| Lua | Returns |
|---|---|
| `rayCast(ray)` | The first body the ray hits, with its id, the hit point, the normal and `t`. |
| `rayCastFiltered(ray, exclude, mask)` | The same, with the filters below. |
| `sphereCast(pos, radius)`, `boxCast(pos, rot, halfExtents)` | Every body the shape overlaps, unsorted, with their ids. |
| `sphereOverlap(pos, radius)`, `boxOverlap(pos, rot, halfExtents)` | Whether the shape overlaps any body. |
| `sphereSweep(from, to, radius, exclude, mask)` | The first body a sphere touches while its centre moves from `from` to `to`: its id, the contact point and normal on that body, and `t`, the distance the centre travels before the contact. A sphere that starts inside a body hits it at 0. |
| `queryRadius(pos, radius, exclude, mask)` | The ids of the bodies within `radius`, nearest first, and the distance from `pos` to each body's surface (0 inside). |

The array results (`sphereCast`, `boxCast`, `queryRadius`) point into engine storage that stays valid until the next call of the same query. Copy what you keep.

### Ids

A body reports the id set with `RigidBody:setUserId`, as a `uint64`; 0 means none. `RigidBodyComponent` copies its entity's id onto its body, so a query result leads back to the entity through `Registry:getEntityById(result.id)`. Child bodies of a compound keep their own ids and are listed separately.

### Filters

- `exclude` (optional) skips every body of the compound the given body belongs to: its parent and all of the parent's children.
- `mask` (optional, all groups by default) keeps only bodies whose collision group (`setCollisionGroup`) shares a bit with it.
- Queries return non-collidable bodies (`setCollidable(false)`) like any other: collidability affects contacts only. To skip them, give them a collision group outside the mask.

## Tuning at ship scale

Rapier's defaults suit bodies about a metre across. The game's ships are a few thousandths of a game unit, so apps with ship flight tune the world when they create it, with values from `Config.game.physics`:

- `Physics:setLengthUnit(gu)`: the typical object size. Rapier scales its tolerances by it, such as the overlap it ignores and the speed below which a body may fall asleep. The game uses about a ship's size.
- `Physics:setContactNaturalFrequency(hz)`: how stiffly contacts push overlapping bodies apart. A high frequency keeps fast ships from sinking deep into each other or into larger bodies.
- `RigidBody:setSoftCcdPrediction(distance)`: soft continuous collision detection. Contacts are predicted up to `distance` ahead of the body, so a fast body is caught before it passes through another within one step. 0 disables it.

The soft-CCD distance is about one frame of travel at the body's speed cap (`Config.game.physics.softCcdDistance`). It grows the body's broad-phase box by that distance, so it follows the current cap rather than the largest one: whatever changes a ship's speed cap sets it again. The engine doesn't recompute it from the velocity each step.

Apps that set nothing keep Rapier's defaults.
