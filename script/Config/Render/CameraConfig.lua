-- Render Settings Moved from 'Render/RenderPipeline' --
Config.render.camera = {
    fov                 = 70,   -- Float: 50 - 100
    -- Both zNear/zFar potentially duplicated in 'Config/genConfig'
    -- Log depth means the GPU never clips at zNear, so this only bounds frustum culling (1 m)
    zNear               = 1e-4,
    zFar                = 1e8,  -- increased for solar system scale rendering
}