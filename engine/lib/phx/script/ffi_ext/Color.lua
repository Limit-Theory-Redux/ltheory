local libphx = require('libphx').lib

function onDef_Color_t(t, mt)
    local Color = t

    mt.__index.clone = function(self)
        return (Color(self.r, self.g, self.b, self.a))
    end

    mt.__index.desaturate = function(self, amount)
        local x = (self.r + self.g + self.b) / 3.0
        self.r = Lerp(self.r, x, amount)
        self.g = Lerp(self.g, x, amount)
        self.b = Lerp(self.b, x, amount)
    end

    mt.__index.lerp = function(self, target, t)
        return Color(
            (1.0 - t) * self.r + t * target.r,
            (1.0 - t) * self.g + t * target.g,
            (1.0 - t) * self.b + t * target.b,
            (1.0 - t) * self.a + t * target.a)
    end

    mt.__index.lum = function(self)
        return 0.2126 * self.r + 0.7152 * self.g + 0.0722 * self.b
    end

    mt.__index.toVec3 = function(self)
        return Vec3f(self.r, self.g, self.b)
    end
end

--------------------------------------------------------------------------------

local function hueComponent(p, q, t)
    if t < 0 then t = t + 1 end
    if t > 1 then t = t - 1 end
    if t < 1 / 6 then return p + (q - p) * 6 * t end
    if t < 1 / 2 then return q end
    if t < 2 / 3 then return p + (q - p) * (2 / 3 - t) * 6 end
    return p;
end

function onDef_Color(t, mt)
    t.FromHSL = function(h, s, l)
        local q
        if l < 0.5 then
            q = l * (1 + s)
        else
            q = l + s - l * s
        end

        local p = 2 * l - q;
        local r = hueComponent(p, q, h + 1 / 3);
        local g = hueComponent(p, q, h);
        local b = hueComponent(p, q, h - 1 / 3);
        return (Color(r, g, b))
    end

    t.FromTemperature = function(K, gamma)
        K = K / 100
        gamma = gamma or 2.2
        local r, g, b

        if K <= 66 then
            r = 255
        else
            r = 329.698727446 * ((K - 60) ^ -0.1332047592)
        end

        if K <= 66 then
            g = 99.4708025861 * log(K) - 161.1195681661
        else
            g = 288.1221695283 * ((K - 60) ^ -0.0755148492)
        end

        if K >= 66 then
            b = 255
        elseif K <= 19 then
            b = 0
        else
            b = 138.5177312231 * log(K - 10) - 305.0447927307
        end
        r = Math.Clamp(r / 255, 0, 1) ^ gamma
        g = Math.Clamp(g / 255, 0, 1) ^ gamma
        b = Math.Clamp(b / 255, 0, 1) ^ gamma
        local l = sqrt(r * r, g * g, b * b)
        return (Color(r / l, g / l, b / l))
    end

    t.Random = function(rng, minHue, maxHue, minSat, maxSat, minLum, maxLum)
        local hue = rng:getUniformRange(minHue, maxHue)
        local sat = rng:getUniformRange(minSat, maxSat)
        local lum = rng:getUniformRange(minLum, maxLum)
        return (Color.FromHSL(hue, sat, lum))
    end
end
