-- AUTO GENERATED. DO NOT MODIFY!
---@meta

---@class Hash
Hash = {}

-- Fowler–Noll–Vo
---@param buf integer[]
---@param buf_size integer
---@return integer
function Hash.FNV32(buf, buf_size) end

---@param buf integer[]
---@param buf_size integer
---@return integer
function Hash.FNV64(buf, buf_size) end

---@param s string
---@return integer
function Hash.FNVStr32(s) end

---@param s string
---@return integer
function Hash.FNVStr64(s) end

---@return integer
function Hash.FNV64_Init() end

---@param this integer
---@param buf integer[]
---@param buf_size integer
---@return integer
function Hash.FNV64_Incremental(this, buf, buf_size) end

