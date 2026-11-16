-- Camera Test: the default star system with every camera a key away, the
-- field of view on keys, and the camera's state on screen. (Was the CameraTest
-- state; its hand-built planet is the default system's planet now.)
--
-- F1 free, F2 orbit, F3 first person, F4 RTS (the one C doesn't cycle to);
-- C cycles chase / first person / orbit / free as always. - and = change the
-- field of view by a degree (put back when the game ends).

local CameraManager = require("Modules.Cameras.Managers.CameraManager")
local CameraDataComponent = require("Modules.Cameras.Components.CameraDataComponent")
local RenderCoreSystem = require("Modules.Rendering.Systems.RenderCoreSystem")
local Overlay = require("Config.Scenarios.Overlay")

local CAMERA_KEYS = {
    { key = Button.KeyboardF1, mode = Enums.CameraMode.Free },
    { key = Button.KeyboardF2, mode = Enums.CameraMode.Orbit },
    { key = Button.KeyboardF3, mode = Enums.CameraMode.FirstPerson },
    { key = Button.KeyboardF4, mode = Enums.CameraMode.RTS },
}

return {
    name = "Camera Test",
    description = "Every camera on a key (F1-F4), the field of view on - and =.",

    setup = function(game)
        game.scenarioData.fov = Config.render.camera.fov
    end,

    onInput = function(game, dt)
        for _, camera in ipairs(CAMERA_KEYS) do
            if Input:isPressed(camera.key) then
                game.player:setMode(camera.mode)
            end
        end
        if Input:isPressed(Button.KeyboardEqual) then
            Config.render.camera.fov = Math.Clamp(Config.render.camera.fov + 1, 1, 179)
        elseif Input:isPressed(Button.KeyboardMinus) then
            Config.render.camera.fov = Math.Clamp(Config.render.camera.fov - 1, 1, 179)
        end
    end,

    drawOverlay = function(game)
        local lines = { "Camera Test: " .. game.player:getModeName() .. " camera" }
        local camEntity = CameraManager:getActiveCameraEntity()
        local controller = camEntity and camEntity:get(CameraDataComponent):getController()
        if controller then
            local pos = controller:getPosition()
            lines[#lines + 1] = string.format("Position: %.3f, %.3f, %.3f", pos.x, pos.y, pos.z)
        end
        lines[#lines + 1] = string.format("Field of view: %.0f    %d fps, %.2f ms",
            Config.render.camera.fov, RenderCoreSystem:getSmoothFPS(), RenderCoreSystem:getSmoothFrameTime(true))
        lines[#lines + 1] = "F1 free    F2 orbit    F3 first person    F4 RTS    C cycle    - = field of view"
        Overlay.draw(lines)
    end,

    teardown = function(game)
        if game.scenarioData.fov then
            Config.render.camera.fov = game.scenarioData.fov
        end
    end,
}
