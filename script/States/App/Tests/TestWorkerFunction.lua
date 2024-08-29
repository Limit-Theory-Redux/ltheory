local WorkerFunction = require("Core.Util.WorkerFunction")

Run = WorkerFunction.Create(function(payload)
    return payload
end)
