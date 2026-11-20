# Cameras

Camera controllers live in `script/Modules/Cameras/Managers/CameraControllers/`. Each one drives a camera entity's transform in `onPreRender(dt)`.

## Chase camera

`ChaseCameraController` sits behind and above its target ship, at an offset given in the ship's local axes (`posRel`). The offset scales with the ship's bounding radius and the zoom (`radius`), so ships of every size fill the view alike.

- **Smoothing.** The camera smooths its offset from the ship, not its world position, with a factor of `1 - exp(-10·dt)`; its rotation is smoothed the same way. A smoothed world position would trail a ship moving at speed `v` by `v / 10`, without bound, so a ship at travel-drive speed would run out of view. Smoothing the offset keeps the lag on turns and on changes of the trail, and none from the speed itself.
- **Speed trail.** The offset grows with the ship's speed `v` by the share `maxLag · min(1, ln(1 + v/vRef) / ln(1 + vTop/vRef))` of the rest distance. `vRef` and `vTop` are `trailReferenceRatio` and `trailTopSpeedRatio` times the ship's flight-profile max speed, and the player's camera sets `vTop` to the travel drive's top speed. Speeds span several decades, from normal flight to the full drive, so a linear trail would either stop growing during normal flight or never show during it. The log curve keeps each decade visible: with the player's settings (`maxLag` 1.5, `vRef` a tenth of max speed, `vTop` 40,000 times it), normal flight at full speed adds about 9 % to the distance, a drive at 200 times about 30 %, and the full drive 50 %. The trail grows with the ship's speed, so it follows the ship's acceleration and the drive's ramps.
- The speed is the ship's world speed, so a ship carried by a gravity well trails by its speed with the well's body included.
