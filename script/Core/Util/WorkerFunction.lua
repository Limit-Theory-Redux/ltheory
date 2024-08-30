local WorkerFunction = {}

-- Wrap worker function in another one with payload FFI GC management.
---@param f fun(any): any Payload function
---@return (fun(number): number?)? -- Worker function wrapped in function with FFI Payload data management
function WorkerFunction.Create(f)
    if type(f) ~= 'function' then
        Log.Error("expected worker function")
        return nil
    end

    local libphx = require('libphx').lib
    local PayloadConverter = require("Core.Util.PayloadConverter")

    return function(payload)
        -- convert integer to the payload pointer sent from the Rust side
        local payloadPtr = ffi.cast("Payload*", payload)
        -- register payload in GC to avoid memory leaks
        local managedPayload = Core.ManagedObject(payloadPtr, libphx.Payload_Free)
        local result = f(PayloadConverter:payloadToValue(managedPayload))
        local outPayloadPtr = PayloadConverter:valueToPayload(result, true)
        -- 'forget' about payload before sending it to the Rust
        ffi.gc(outPayloadPtr, nil)
        -- cast payload pointer to number to be sent to Rust
        local outPayload = tonumber(ffi.cast("uintptr_t", outPayloadPtr))
        return outPayload
    end
end

return WorkerFunction
