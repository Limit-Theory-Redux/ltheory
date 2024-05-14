local function initPropsFromFuncs(self)
    local propFromFuncs = {
        { GuiPropertyValue.FromBool,   GuiPropertyValue.getBool },
        { GuiPropertyValue.FromI8,     GuiPropertyValue.getI8 },
        { GuiPropertyValue.FromU8,     GuiPropertyValue.getU8 },
        { GuiPropertyValue.FromI16,    GuiPropertyValue.getI16 },
        { GuiPropertyValue.FromU16,    GuiPropertyValue.getU16 },
        { GuiPropertyValue.FromI32,    GuiPropertyValue.getI32 },
        { GuiPropertyValue.FromU32,    GuiPropertyValue.getU32 },
        { GuiPropertyValue.FromI64,    GuiPropertyValue.getI64 },
        { GuiPropertyValue.FromU64,    GuiPropertyValue.getU64 },
        { GuiPropertyValue.FromF32,    GuiPropertyValue.getF32 },
        { GuiPropertyValue.FromF64,    GuiPropertyValue.getF64 },
        { GuiPropertyValue.FromVec2,   GuiPropertyValue.getVec2 },
        { GuiPropertyValue.FromVec3,   GuiPropertyValue.getVec3 },
        { GuiPropertyValue.FromVec4,   GuiPropertyValue.getVec4 },
        { GuiPropertyValue.FromIvec2,  GuiPropertyValue.getIvec2 },
        { GuiPropertyValue.FromIvec3,  GuiPropertyValue.getIvec3 },
        { GuiPropertyValue.FromIvec4,  GuiPropertyValue.getIvec4 },
        { GuiPropertyValue.FromUvec2,  GuiPropertyValue.getUvec2 },
        { GuiPropertyValue.FromUvec3,  GuiPropertyValue.getUvec3 },
        { GuiPropertyValue.FromUvec4,  GuiPropertyValue.getUvec4 },
        { GuiPropertyValue.FromDvec2,  GuiPropertyValue.getDvec2 },
        { GuiPropertyValue.FromDvec3,  GuiPropertyValue.getDvec3 },
        { GuiPropertyValue.FromDvec4,  GuiPropertyValue.getDvec4 },
        { GuiPropertyValue.FromColor,  GuiPropertyValue.getColor },
        { GuiPropertyValue.FromBox3,   GuiPropertyValue.getBox3 },
        { GuiPropertyValue.FromString, GuiPropertyValue.getString },
        { GuiPropertyValue.FromFont,   GuiPropertyValue.getFont },
    }

    local propCount = tonumber(Gui:getPropertiesCount() - 1)

    for id = 0, propCount do
        local ty = tonumber(Gui:getPropertyType(id) + 1)
        self.propFromFuncMap[id] = propFromFuncs[ty]
    end

    Log.Debug("Initialized HmGui propFromFuncs")
end

function onDef_HmGui_t(t, mt)
    mt.__index.propFromFuncMap = {}

    mt.__index.setProperty = function(self, id, value)
        if self.propFromFuncMap and #self.propFromFuncMap == 0 then
            -- init props once
            initPropsFromFuncs(self)
        end

        Gui:setPropertyValue(id, self.propFromFuncMap[id][1](value))
    end

    mt.__index.getProperty = function(self, id)
        if self.propFromFuncMap and #self.propFromFuncMap == 0 then
            -- init props once
            initPropsFromFuncs(self)
        end

        return self.propFromFuncMap[id][2](Gui:getPropertyValue(id))
    end
end
