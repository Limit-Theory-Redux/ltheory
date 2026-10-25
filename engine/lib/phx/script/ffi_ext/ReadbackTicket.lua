-- `ticket:free()` gives a readback ticket up before the garbage collector
-- would (`release` on the Rust side; the ticket reads as empty afterwards).
function onDef_ReadbackTicket_t(t, mt)
    mt.__index.free = mt.__index.release
end
