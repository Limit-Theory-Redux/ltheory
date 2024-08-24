-- Render Settings Moved from 'Render/RenderPipeline' --
Config.render.camera = {
    fov                 = 70,   -- Float: 50 - 100
    -- Both zNear/zFar potentially duplicated in 'Config/genConfig'
    zNear               = 0.1,  -- default: 0.1 -- for NDC Coordinate System
    zFar                = 1e6,  -- default: 1e6 -- for NDC Coordinate System
}