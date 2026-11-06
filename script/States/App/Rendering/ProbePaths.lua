local ProbePaths = {}

local function directory()
    local base = Config.paths and Config.paths.files
    if not base or base == "" then
        error("Rendering validation requires Config.paths.files")
    end
    base = base:gsub("[/\\\\]+$", "")

    local path = format("%s/rendering-validation", base)
    Directory.Create(path)
    if not File.IsDir(path) then
        error("Failed to create rendering validation directory: " .. path)
    end
    return path
end

function ProbePaths.file(name)
    assert(name and name ~= "", "Rendering validation artifact name is required")
    return format("%s/%s", directory(), name)
end

return ProbePaths
