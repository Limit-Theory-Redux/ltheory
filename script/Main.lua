package.path = package.path .. ';./engine/lib/phx/script/?.lua'
package.path = package.path .. ';./script/?.lua'
package.path = package.path .. ';./script/?.ext.lua'
package.path = package.path .. ';./script/?.ffi.lua'

EngineInstance = {}
InputInstance = {}
WindowInstance = {}
Gui = {}

require('Init')

function SetEngine(engine)
    Log.Debug("SetEngine")

    EngineInstance = ffi.cast('Engine*', engine)

    InputInstance = EngineInstance:input()
    WindowInstance = EngineInstance:window()
    Gui = EngineInstance:hmGui()
end

function InitSystem()
    Log.Debug("InitSystem")

    Core.Call(function()
        local app = __app__ or 'LTheoryRedux'

        Log.Debug("Application name: %s", app)

        GlobalRestrict.On()

        dofile('./script/Config/Version.lua')

        if Config.gameVersion ~= "0.0.0" and Config.gameVersion ~= Engine.GetVersion() then
            Log.Error("Engine and script version mismatch. Engine: %s. Script: %s.", Engine.GetVersion(), Config.gameVersion)
        end

        dofile('./script/Config/App.lua')

        -- Load Enums
        for _, fname in ipairs(io.listdirex(Config.paths.enums)) do
            dofile(Config.paths.enums .. fname)
        end

        -- Load Types
        for _, fname in ipairs(io.listdirex(Config.paths.types)) do
            dofile(Config.paths.types .. fname)
        end

        Namespace.Load('UI')
        Namespace.LoadInline('Systems')
        Namespace.LoadInline('GameObjects')

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

        --local logG = io.open("_g.log", "w+")
        --io.output(logG)
        --io.write(Inspect(_G))
        --io.close(logG)
        local foundState, state = pcall(require, 'States.App.' .. app)
        local foundTest, test = pcall(require, 'States.App.Tests.' .. app)

        local appState = nil

        if foundState then
            appState = state
        elseif foundTest then
            appState = test
        end

        if appState == nil then
            Log.Error("Application was not specified")
        end

        AppInit = function()
            Core.Call(appState.appInit, appState)
        end

        AppFrame = function()
            Core.Call(appState.onFrame, appState)
        end

        AppClose = function()
            Core.Call(appState.doExit, appState)
            GlobalRestrict.Off()
        end
    end)
end
