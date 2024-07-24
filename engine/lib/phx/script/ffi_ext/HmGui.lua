local libphx = require('libphx').lib

function onDef_HmGui_t(t, mt)
    mt.__index.endGui = function(self)
        libphx.HmGui_EndGui(self, Input)
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
        Gui:rect();
        Gui:setAlignment(AlignHorizontal.Stretch, AlignVertical.Stretch);
        Gui:setBackgroundColor(Color(0, 0, 0, 0))
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
