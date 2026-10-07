local libphx = require('libphx').lib
local memory

function onDef_Profiler(t, mt)
    t.BeginMemoryProfile = function()
        GC.Collect()
        --GC.Stop()
        memory = GC.GetMemory()
    end

    t.EndMemoryProfile = function()
        local dMemory = GC.GetMemory() - memory
        --GC.Start() **disable automatic since we do manually GC in Application.lua (@IllustrisJack)**
        return dMemory
    end

    t.TimeGPU = function(name, fn)
        Renderer:gpuFinish()
        local begin = TimeStamp.Now()
        fn()
        Renderer:gpuFinish()
        local duration = begin:getElapsedMs()
        Log.Info('%s : %.2f ms', name, duration)
    end
end
