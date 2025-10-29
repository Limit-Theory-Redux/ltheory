package.path = package.path .. ';./engine/lib/phx/script/?.lua'
package.path = package.path .. ';./script/?.lua'
package.path = package.path .. ';./script/?/__init__.lua'
package.path = package.path .. ';./script/?.ext.lua'
package.path = package.path .. ';./script/?.ffi.lua'

require('Init')
local ErrorHandler = require('Core.Util.ErrorHandler')

function SetEngine(engine)
    Engine = ffi.cast('Engine*', engine)
    ---@cast Engine Engine

    EventBus = Engine:eventBus()
    TaskQueue = Engine:taskQueue()
    Input = Engine:input()
    Window = Engine:window()
    Gui = Engine:hmGui()
end

function InitSystem()
    Core.Call(function()
        local app = __app__ or 'LTheoryRedux'
        Log.Debug("Application name: %s", app)

        GlobalRestrict.On()

        dofile('./script/Config/Version.lua')

        if Config.gameVersion ~= "0.0.0" and Config.gameVersion ~= Core.FFI.Engine.GetVersion() then
            Log.Error("Engine and script version mismatch. Engine: %s. Script: %s.", Core.FFI.Engine.GetVersion(),
                Config.gameVersion)
        end

        -- ensure App.lua is loaded first
        dofile('./script/Config/App.lua')

        -- Load Enums
        for _, fname in ipairs(io.listdirex(Config.paths.enums)) do
            dofile(Config.paths.enums .. fname)
        end

        local configDir = io.listdir('./script/Config', true)

        local function loadConfigFiles(configDir)
            -- Ensure dependencies are loaded first
            local success, items = pcall(require, "Shared.Registries.Items")
            if not success then
                Log.Error("Failed to load Shared.Registries.Items: %s", tostring(items))
            end

            for _, fpath in ipairs(configDir) do
                -- Skip App.lua, __init__.lua, and Rulesets directory
                if fpath ~= "./script/Config/App.lua" and
                    not string.find(fpath, "__init__") and
                    not string.find(fpath, "Rulesets") then
                    Log.Debug("Loading config file: %s", fpath)
                    local success, res = pcall(dofile, fpath)
                    if not success then
                        Log.Warn("Config file %s could not be executed: %s", fpath, tostring(res))
                    end
                end
            end
        end

        -- Load Config Files
        loadConfigFiles(configDir)

        -- Load Types
        for _, fname in ipairs(io.listdirex(Config.paths.types)) do
            dofile(Config.paths.types .. fname)
        end

        Namespace.LoadInline('UI.HmGui.UICore')
        Namespace.LoadInline('UI.HmGui.Components')
        Namespace.LoadInline('UI.HmGui.Layouts')
        Namespace.LoadInline('UI.HmGui.Views')
        Namespace.LoadInline('UI.HmGui.Pages') -- needs to be loaded in correct order
        Namespace.Load('UI')

        jit.opt.start(
            format('maxtrace=%d', Config.jit.tune.maxTrace),
            format('maxrecord=%d', Config.jit.tune.maxRecord),
            format('maxirconst=%d', Config.jit.tune.maxConst),
            format('maxside=%d', Config.jit.tune.maxSide),
            format('maxsnap=%d', Config.jit.tune.maxSnap),
            format('hotloop=%d', Config.jit.tune.hotLoop),
            format('hotexit=%d', Config.jit.tune.hotExit),
            format('tryside=%d', Config.jit.tune.trySide),
            format('instunroll=%d', Config.jit.tune.instUnroll),
            format('loopunroll=%d', Config.jit.tune.loopUnroll),
            format('callunroll=%d', Config.jit.tune.callUnroll),
            format('recunroll=%d', Config.jit.tune.recUnroll),
            format('sizemcode=%d', Config.jit.tune.sizeMCode),
            format('maxmcode=%d', Config.jit.tune.maxMCode)
        )

        -- local logG = io.open("_g.log", "w+")
        -- io.output(logG)
        -- io.write(Inspect(_G))
        -- io.close(logG)

        -- check for / and replace with . to allow for subdirectory calls (e.g. ECS/UniverseCreationTest)
        if app:find("/") then app:gsub("/", ".") end

        local foundState, state = pcall(require, 'States.App.' .. app)
        local foundTest, test = pcall(require, 'States.App.Tests.' .. app)

        local appState = nil
        if foundState then
            appState = state
        elseif foundTest then
            appState = test
        end

        if appState == nil then
            -- If the error returned is "module 'States.App.<app>' not found:",
            -- then the error is in test instead.
            if tostring(state):match("^module 'States%.App%." .. app .. "' not found:") then
                Log.Error("Failed to load States.Apps.Tests.%s: %s", app, tostring(test))
            else
                Log.Error("Failed to load States.Apps.%s: %s", app, tostring(state))
            end
        end

        AppInit = function()
            Core.Call(appState.appInit, appState)
        end

        AppEventLoop = function()
            Core.Call(appState.eventLoop, appState)
        end

        AppClose = function()
            Core.Call(appState.doExit, appState)
            GlobalRestrict.Off()
        end
    end)
end

function HandleEngineError(err)
    ErrorHandler(err)
end
