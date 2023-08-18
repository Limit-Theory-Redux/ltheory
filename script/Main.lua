package.path = package.path .. ';./engine/lib/phx/script/?.lua'
package.path = package.path .. ';./script/?.lua'
package.path = package.path .. ';./script/?.ext.lua'
package.path = package.path .. ';./script/?.ffi.lua'

require('Init')

Core.Call(function()
    local app = __app__ or 'LTheoryRedux'
    GlobalRestrict.On()

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
        error("Application was not specified")
    end

    AppInit = function(engine)
        printf('Application(%s).setEngine(%s)', appState, engine)
        Core.Call(function() appState.setEngine(engine) end)
    end

    AppFrame = function()
        Core.Call(function() appState.onFrame() end)
    end

    AppClose = function()
        Core.Call(function() appState.doExit() end)
        GlobalRestrict.Off()
    end
end)
