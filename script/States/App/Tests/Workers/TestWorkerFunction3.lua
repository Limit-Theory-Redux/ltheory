package.path = package.path .. ';./engine/lib/phx/script/?.lua'
package.path = package.path .. ';./script/?.lua'

require('Init')

local WorkerFunction = require("Core.Util.WorkerFunction")

-- Behaviour selected by the task: "error" raises, "nil" returns nothing, anything else is echoed.
Run = WorkerFunction.Create(function(payload)
    if payload == "error" then
        error("intentional worker error")
    end
    if payload == "nil" then
        return nil
    end
    return payload
end)
