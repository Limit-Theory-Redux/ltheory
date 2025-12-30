-- AUTO GENERATED. DO NOT MODIFY!
---@meta

---@class ShaderError
ShaderError = {}

-- Returns the number of shader errors in the queue
---@return integer
function ShaderError.GetCount() end

-- Returns whether there are new (unacknowledged) errors
---@return boolean
function ShaderError.HasNewErrors() end

-- Acknowledges all current errors (clears the "new" flag)
function ShaderError.AcknowledgeErrors() end

-- Gets the shader key for error at index (0-based)
---@param index integer
---@return string?
function ShaderError.GetShaderKey(index) end

-- Gets the error type for error at index ("compile" or "link")
---@param index integer
---@return string?
function ShaderError.GetErrorType(index) end

-- Gets the error message for error at index
---@param index integer
---@return string?
function ShaderError.GetMessage(index) end

-- Gets the timestamp for error at index
---@param index integer
---@return integer
function ShaderError.GetTimestamp(index) end

-- Clears all errors from the queue
function ShaderError.Clear() end

-- Clears a specific error by index
---@param index integer
function ShaderError.ClearAt(index) end

-- Clears all errors for a specific shader key
---@param shaderKey string
function ShaderError.ClearForShader(shaderKey) end

-- Called each frame to update internal state
function ShaderError.Update() end

-- Gets the most recent error message (for quick display)
---@return string?
function ShaderError.GetLatestMessage() end

-- Gets the most recent shader key that had an error
---@return string?
function ShaderError.GetLatestShaderKey() end

