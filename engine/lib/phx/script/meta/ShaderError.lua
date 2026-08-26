-- AUTO GENERATED. DO NOT MODIFY!
---@meta

---@class ShaderError
ShaderError = {}

-- Returns the number of shader errors in the queue.
---@param r Renderer
---@return integer
function ShaderError.GetCount(r) end

-- Returns whether there are new (unacknowledged) errors.
---@param r Renderer
---@return boolean
function ShaderError.HasNewErrors(r) end

-- Acknowledges all current errors (clears the "new" flag).
---@param r Renderer
function ShaderError.AcknowledgeErrors(r) end

-- Gets the shader key for the error at index (0-based).
---@param r Renderer
---@param index integer
---@return string?
function ShaderError.GetShaderKey(r, index) end

-- Gets the error type for the error at index ("compile" or "link").
---@param r Renderer
---@param index integer
---@return string?
function ShaderError.GetErrorType(r, index) end

-- Gets the error message for the error at index.
---@param r Renderer
---@param index integer
---@return string?
function ShaderError.GetMessage(r, index) end

-- Gets the timestamp for the error at index.
---@param r Renderer
---@param index integer
---@return integer
function ShaderError.GetTimestamp(r, index) end

-- Clears all errors from the queue.
---@param r Renderer
function ShaderError.Clear(r) end

-- Clears a specific error by index.
---@param r Renderer
---@param index integer
function ShaderError.ClearAt(r, index) end

-- Clears all errors for a specific shader key.
---@param r Renderer
---@param shaderKey string
function ShaderError.ClearForShader(r, shaderKey) end

-- Called each frame to update internal state.
---@param r Renderer
function ShaderError.Update(r) end

-- Gets the most recent error message (for quick display).
---@param r Renderer
---@return string?
function ShaderError.GetLatestMessage(r) end

-- Gets the most recent shader key that had an error.
---@param r Renderer
---@return string?
function ShaderError.GetLatestShaderKey(r) end

