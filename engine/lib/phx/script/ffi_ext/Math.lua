local libphx = require('libphx').lib

---@class Math
---@field Tau number
---@field Pi number
---@field Pi2 number
---@field Pi4 number
---@field Pi6 number
---@field Infinity number
---@field Float32MaxInt number
---@field Float64MaxInt number
---@field ToRadians fun(degrees: number): number
---@field ToDegrees fun(radians: number): number
---@field Approx fun(a: number, b: number): boolean
---@field Avg fun(a: number, b: number): number
---@field DivAndMod fun(x: number, divisor: number): (number, number)
---@field EMA fun(last: number, current: number, dt: number, period: number): number
---@field GeomAvg fun(a: number, b: number): number
---@field Lerp fun(a: number, b: number, t: number): number
---@field LerpSnap fun(a: number, b: number, t: number): number
---@field Impact fun(pSrc: Position, pDst: Position, vSrc: Vec3f, vDst: Vec3f, speed: number): (number, Position)?
---@field InverseLerp fun(a: number, b: number, v: number): number
---@field OrthoVector fun(e1: Vec3f): Vec3f
---@field OrthoBasis fun(e1: Vec3f): (Vec3f, Vec3f)
---@field IsInfinite fun(x: number): boolean
---@field IsNaN fun(x: number): boolean
---@field IsOk fun(x: number): boolean
---@field AbsMax fun(a: number, b: number): number
---@field SmoothStep3 fun(x: number): number
---@field SmoothStep5 fun(x: number): number
---@field SmoothStep7 fun(x: number): number
---@field Round fun(x: number): number
---@field Saturate fun(x: number): number
---@field Sign fun(x: number): number
---@field Sign0 fun(x: number): number
---@field Snap fun(x: number, a: number, b: number, epsilon: number): number
---@field Spherical fun(radius: number, pitch: number, yaw: number): Vec3f
---@field Wrap fun(x: number, min: number, max: number): number

function onDef_Math(t, mt)
    t.Tau           = 6.283185307179586232
    t.Pi            = 3.141592653589793116
    t.Pi2           = 1.570796326794896558
    t.Pi4           = 0.785398163397448279
    t.Pi6           = 0.52359877559829881566
    t.Infinity      = math.huge

    t.Float32MaxInt = 16777216         -- 2^24
    t.Float64MaxInt = 9007199254740992 -- 2^53

    t.ToRadians     = rad
    t.ToDegrees     = deg

    t.Approx        = function(a, b)
        return abs(a - b) < 1e-4
    end

    t.Avg           = function(a, b)
        return (a + b) / 2.0
    end

    t.DivAndMod     = function(x, divisor)
        local div = floor(x / divisor)
        local mod = x - div * divisor
        return div, mod
    end

    -- Exponential moving average
    t.EMA           = function(last, current, dt, period)
        local factor = exp(-dt / period)
        return factor * last + (1.0 - factor) * current
    end

    t.GeomAvg       = function(a, b)
        return (sqrt(a * b))
    end

    t.Lerp          = function(a, b, t)
        return a + (b - a) * t
    end

    t.LerpSnap      = function(a, b, t)
        local r = a + (b - a) * t
        if abs(a - r) < 1e-3 then r = a end
        if abs(b - r) < 1e-3 then r = b end
        return r
    end

    -- Compute projectile impact time and point given position and velocity of
    -- source / destination + closing speed of projectile. Assumes projectile
    -- inherits the velocity of source (in addition to the closing speed).
    -- Returns impact_time, impact_point or nil if no solution exists
    t.Impact        = function(pSrc, pDst, vSrc, vDst, speed)
        local dp = pDst - pSrc
        local dv = vDst - vSrc
        local a = dv:lengthSquared() - speed * speed
        local b = 2.0 * dv:dot(dp)
        local c = dp:lengthSquared()
        local d = b * b - 4.0 * a * c
        if d <= 0 then return nil end
        d = sqrt(d)
        local t = max((-b - d) / (2.0 * a), (-b + d) / 2.0 * a)
        return t, pDst + dv:scale(t)
    end

    t.InverseLerp   = function(a, b, v)
        return (v - a) / (b - a)
    end

    t.OrthoVector   = function(e1)
        local absY = abs(e1.y)
        if absY < 0.5 then
            return Vec3f(e1.z, 0, -e1.x):normalize()
        else
            return Vec3f(e1.y, -e1.x, 0):normalize()
        end
    end

    t.OrthoBasis    = function(e1)
        local e2 = Math.OrthoVector(e1)
        local e3 = e1:cross(e2):normalize()
        return e2, e3
    end

    t.IsInfinite    = function(x)
        return x == math.huge or x == -math.huge
    end

    t.IsNaN         = function(x)
        return x ~= x
    end

    t.IsOk          = function(x)
        return not Math.IsInfinite(x) and not Math.IsNaN(x)
    end

    t.AbsMax        = function(a, b)
        return abs(b) > abs(a) and b or a
    end

    -- Smoothstep 3rd order ('Smoothstep')
    -- 3x^2 - 2x^3
    t.SmoothStep3   = function(x)
        return x * x * (3 - 2 * x)
    end

    -- Smoothstep 5th order ('Smootherstep')
    -- 6x^5 - 15x^4 + 10x^3
    t.SmoothStep5   = function(x)
        return x * x * x * (10 + x * (-15 + 6 * x))
    end

    -- Smoothstep 7th order ('Smootheststep')
    -- -20x^7 + 70x^6 - 84x^5 + 35x^4
    t.SmoothStep7   = function(x)
        return x * x * x * x * (35 + x * (-84 + x * (70 - 20 * x)))
    end

    t.Round         = function(x)
        return floor(x + 0.5)
    end

    t.Saturate      = function(x)
        return max(0.0, min(1.0, x))
    end

    t.Sign          = function(x)
        return (x >= 0 and 1) or -1
    end

    t.Sign0         = function(x)
        return (x > 0 and 1) or (x < 0 and -1) or 0
    end

    t.Snap          = function(x, a, b, epsilon)
        if abs(x - a) < epsilon then return a end
        if abs(x - b) < epsilon then return b end
        return x
    end

    t.Spherical     = function(radius, pitch, yaw)
        return (Vec3f(
            radius * cos(pitch) * cos(yaw),
            radius * sin(pitch),
            radius * cos(pitch) * sin(yaw)))
    end

    -- WARNING : Integers only! Floats will wrap incorrectly.
    t.Wrap          = function(x, min, max)
        -- NOTE : Lua's integer mod function wraps negative back to positive,
        --        hence the following simple formula works:
        --           wrapped = offset % range + min
        return (x - min) % (max - min + 1) + min
    end
end
