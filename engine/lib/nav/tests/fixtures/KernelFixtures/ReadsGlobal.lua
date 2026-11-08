---Fixture of a broken kernel: reads a global that no state defines.
local ReadsGlobal = {}

function ReadsGlobal.run()
    return Config.value
end

return ReadsGlobal
