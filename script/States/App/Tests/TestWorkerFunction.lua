package.path = package.path .. ';./engine/lib/phx/script/?.lua'
package.path = package.path .. ';./script/?.lua'

require('Init')

local WorkerFunction = require("Core.Util.WorkerFunction")

Run = WorkerFunction.Create(function(payload)
    return tostring(payload) .. "_OUT"
end)
