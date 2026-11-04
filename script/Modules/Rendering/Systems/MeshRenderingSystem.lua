--[[
    Mesh entities are drawn by RenderCoreSystem: `buildPassLists` adds one
    transform and one item per mesh to its SceneList, and each scene pass calls
    `scene:submit(pass, blendMode)` (frustum cull, sort by pipeline/material/mesh,
    per-draw callbacks of the survivors, one DrawBlock per drawn mesh).
    See doc/engine/render-api-v2.md, 3b.
--]]
