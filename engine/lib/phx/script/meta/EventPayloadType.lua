---@meta

---@enum EventPayloadType
EventPayloadType = {
    -- Lua object pointer to communicate inside scripts only
    Lua = 0,
    Bool = 1,
    I8 = 2,
    U8 = 3,
    I16 = 4,
    U16 = 5,
    I32 = 6,
    U32 = 7,
    I64 = 8,
    U64 = 9,
    F32 = 10,
    F64 = 11,
    String = 12,
    Table = 13,
}

