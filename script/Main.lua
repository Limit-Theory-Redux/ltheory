package.path = package.path .. ';./libphx/script/?.lua'
package.path = package.path .. ';./script/?.lua'
package.path = package.path .. ';./script/?.ext.lua'
package.path = package.path .. ';./script/?.ffi.lua'

require('Init')

Core.Call(function ()
  local app = __app__ or 'LTheoryRedux'
  GlobalRestrict.On()

  dofile('./script/Config/App.lua')
  if io.exists ('./script/Config/Local.lua') then dofile('./script/Config/Local.lua') end

  Namespace.Load('UI')
  Namespace.LoadInline('Systems')
  Namespace.LoadInline('GameObjects')

  jit.opt.start(
    format('maxtrace=%d',   Config.jit.tune.maxTrace),
    format('maxrecord=%d',  Config.jit.tune.maxRecord),
    format('maxirconst=%d', Config.jit.tune.maxConst),
    format('maxside=%d',    Config.jit.tune.maxSide),
    format('maxsnap=%d',    Config.jit.tune.maxSnap),
    format('hotloop=%d',    Config.jit.tune.hotLoop),
    format('hotexit=%d',    Config.jit.tune.hotExit),
    format('tryside=%d',    Config.jit.tune.trySide),
    format('instunroll=%d', Config.jit.tune.instUnroll),
    format('loopunroll=%d', Config.jit.tune.loopUnroll),
    format('callunroll=%d', Config.jit.tune.callUnroll),
    format('recunroll=%d',  Config.jit.tune.recUnroll),
    format('sizemcode=%d',  Config.jit.tune.sizeMCode),
    format('maxmcode=%d',   Config.jit.tune.maxMCode)
  )

  --local logG = io.open("_g.log", "w+")
  --io.output(logG)
  --io.write(Inspect(_G))
  --io.close(logG)
  require('States.App.' .. app):run()
  GlobalRestrict.Off()
end)
