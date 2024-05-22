local function initPropsFromFuncs(self)
    local propFromFuncs = {
        { set = GuiPropertyValue.FromBool,   get = GuiPropertyValue.getBool },
        { set = GuiPropertyValue.FromI8,     get = GuiPropertyValue.getI8 },
        { set = GuiPropertyValue.FromU8,     get = GuiPropertyValue.getU8 },
        { set = GuiPropertyValue.FromI16,    get = GuiPropertyValue.getI16 },
        { set = GuiPropertyValue.FromU16,    get = GuiPropertyValue.getU16 },
        { set = GuiPropertyValue.FromI32,    get = GuiPropertyValue.getI32 },
        { set = GuiPropertyValue.FromU32,    get = GuiPropertyValue.getU32 },
        { set = GuiPropertyValue.FromI64,    get = GuiPropertyValue.getI64 },
        { set = GuiPropertyValue.FromU64,    get = GuiPropertyValue.getU64 },
        { set = GuiPropertyValue.FromF32,    get = GuiPropertyValue.getF32 },
        { set = GuiPropertyValue.FromF64,    get = GuiPropertyValue.getF64 },
        { set = GuiPropertyValue.FromVec2,   get = GuiPropertyValue.getVec2 },
        { set = GuiPropertyValue.FromVec3,   get = GuiPropertyValue.getVec3 },
        { set = GuiPropertyValue.FromVec4,   get = GuiPropertyValue.getVec4 },
        { set = GuiPropertyValue.FromIvec2,  get = GuiPropertyValue.getIvec2 },
        { set = GuiPropertyValue.FromIvec3,  get = GuiPropertyValue.getIvec3 },
        { set = GuiPropertyValue.FromIvec4,  get = GuiPropertyValue.getIvec4 },
        { set = GuiPropertyValue.FromUvec2,  get = GuiPropertyValue.getUvec2 },
        { set = GuiPropertyValue.FromUvec3,  get = GuiPropertyValue.getUvec3 },
        { set = GuiPropertyValue.FromUvec4,  get = GuiPropertyValue.getUvec4 },
        { set = GuiPropertyValue.FromDvec2,  get = GuiPropertyValue.getDvec2 },
        { set = GuiPropertyValue.FromDvec3,  get = GuiPropertyValue.getDvec3 },
        { set = GuiPropertyValue.FromDvec4,  get = GuiPropertyValue.getDvec4 },
        { set = GuiPropertyValue.FromColor,  get = GuiPropertyValue.getColor },
        { set = GuiPropertyValue.FromBox3,   get = GuiPropertyValue.getBox3 },
        { set = GuiPropertyValue.FromString, get = GuiPropertyValue.getString },
        { set = GuiPropertyValue.FromFont,   get = GuiPropertyValue.getFont },
    }

    local propCount = tonumber(Gui:getPropertiesCount() - 1)

    for id = 0, propCount do
        local ty = tonumber(Gui:getPropertyType(id) + 1)
        self.propFromFuncMap[id] = propFromFuncs[ty]
    end

    Log.Debug("Initialized HmGui propFromFuncs")
end

function onDef_HmGui_t(t, mt)
    mt.__index.init = function(self) -- dirtiest but fastest solution
        mt.__index.propFromFuncMap = {}

        -- init props once
        initPropsFromFuncs(self)
    end

    mt.__index.setStyleProperty = function(self, style_id, prop_id, value)
        Gui:setStylePropertyValue(style_id, prop_id, self.propFromFuncMap[prop_id].set(value))
    end

    mt.__index.setProperty = function(self, id, value)
        Gui:setPropertyValue(id, self.propFromFuncMap[id].set(value))
    end

    mt.__index.getProperty = function(self, id)
        return self.propFromFuncMap[id].get(Gui:getPropertyValue(id))
    end

    mt.__index.beginWindow = function(self, name)
        Gui:beginStackContainer()
        Gui:beginVerticalContainer()
        Gui:setAlignment(AlignHorizontal.Stretch, AlignVertical.Stretch)
        Gui:setPadding(8, 8)
    end

    mt.__index.endWindow = function(self)
        Gui:endContainer()
        Gui:endContainer()
    end

    --- Invisible element that stretches in all directions.
    --- Use for pushing neighbor elements to the sides. See [`Self::checkbox`] for example.
    mt.__index.spacer = function(self)
        Gui:setProperty(GuiProperties.BackgroundColor, Color(0, 0, 0, 0));
        Gui:rect();
        Gui:setAlignment(AlignHorizontal.Stretch, AlignVertical.Stretch);
    end

    mt.__index.horizontalDivider = function(self, height)
        Gui:rect();
        Gui:setFixedHeight(height);
        Gui:setHorizontalAlignment(AlignHorizontal.Stretch);
    end

    mt.__index.verticalDivider = function(self, width)
        Gui:rect();
        Gui:setFixedWidth(width);
        Gui:setVerticalAlignment(AlignVertical.Stretch);
    end
end
