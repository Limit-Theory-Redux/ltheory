return {
    Entities = {
        Camera = require('Modules.Rendering.Entities.CameraEntity'),
    },
    Components = {
        CameraData = require("Modules.Rendering.Components.CameraDataComponent"),
        Effect = require('Modules.Rendering.Components.EffectComponent'),
        Render = require('Modules.Rendering.Components.RenderComponent'),
    },
    Systems = {
        Camera = require('Modules.Rendering.Systems.CameraSystem'),
        MeshRendering = require('Modules.Rendering.Systems.MeshRenderingSystem'),
        RenderCore = require('Modules.Rendering.Systems.RenderCoreSystem'),
    }
}
