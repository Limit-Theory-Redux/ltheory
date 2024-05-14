local libphx = require('libphx').lib

local function initPropsFromFuncs(self)
    local propFromFuncs = {
        GuiPropertyValue.FromBool,
        GuiPropertyValue.FromI8,
        GuiPropertyValue.FromU8,
        GuiPropertyValue.FromI16,
        GuiPropertyValue.FromU16,
        GuiPropertyValue.FromI32,
        GuiPropertyValue.FromU32,
        GuiPropertyValue.FromI64,
        GuiPropertyValue.FromU64,
        GuiPropertyValue.FromF32,
        GuiPropertyValue.FromF64,
        GuiPropertyValue.FromVec2,
        GuiPropertyValue.FromVec3,
        GuiPropertyValue.FromVec4,
        GuiPropertyValue.FromIvec2,
        GuiPropertyValue.FromIvec3,
        GuiPropertyValue.FromIvec4,
        GuiPropertyValue.FromUvec2,
        GuiPropertyValue.FromUvec3,
        GuiPropertyValue.FromUvec4,
        GuiPropertyValue.FromDvec2,
        GuiPropertyValue.FromDvec3,
        GuiPropertyValue.FromDvec4,
        GuiPropertyValue.FromColor,
        GuiPropertyValue.FromBox3,
        GuiPropertyValue.FromString,
        GuiPropertyValue.FromFont,
    }

    local propCount = tonumber(Gui:getPropertiesCount() - 1)

    for id = 0, propCount do
        local ty = Gui:getPropertyType(id)
        self.propFromFuncMap[id] = propFromFuncs[ty]
    end

    Log.Debug("Initialized HmGui propFromFuncs")
end

function onDef_HmGui_t(t, mt)
    mt.__index.propFromFuncMap = {}

    mt.__index.setPropertyValue = function(self, id, value)
        if self.propFromFuncMap and #self.propFromFuncMap == 0 then
            -- init props once
            initPropsFromFuncs(self)
        end
        Gui:setProperty(id, self.propFromFuncMap[id](value))
    end
end
