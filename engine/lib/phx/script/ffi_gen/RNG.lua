-- AUTO GENERATED. DO NOT MODIFY!
-- RNG -------------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef struct RNG {} RNG;
    ]]

    return 1, 'RNG'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local RNG

    do -- C Definitions
        ffi.cdef [[
            void   RNG_Free            (RNG*);
            RNG*   RNG_Create          (uint64 seed);
            RNG*   RNG_FromStr         (cstr s);
            RNG*   RNG_FromTime        ();
            void   RNG_Rewind          (RNG*);
            bool   RNG_Chance          (RNG*, double probability);
            int    RNG_Get31           (RNG*);
            uint32 RNG_Get32           (RNG*);
            uint64 RNG_Get64           (RNG*);
            double RNG_GetAngle        (RNG*);
            int    RNG_GetInt          (RNG*, int lower, int upper);
            RNG*   RNG_GetRNG          (RNG*);
            double RNG_GetUniform      (RNG*);
            double RNG_GetUniformRange (RNG*, double lower, double upper);
            double RNG_GetErlang       (RNG*, int k);
            double RNG_GetExp          (RNG*);
            double RNG_GetGaussian     (RNG*);
            void   RNG_GetAxis2        (RNG*, Vec2f* out);
            void   RNG_GetAxis3        (RNG*, Vec3f* out);
            void   RNG_GetDir2         (RNG*, Vec2f* out);
            void   RNG_GetDir3         (RNG*, Vec3f* out);
            void   RNG_GetDisc         (RNG*, Vec2f* out);
            double RNG_GetSign         (RNG*);
            void   RNG_GetSphere       (RNG*, Vec3f* out);
            void   RNG_GetVec2         (RNG*, Vec2f* out, double lower, double upper);
            void   RNG_GetVec3         (RNG*, Vec3f* out, double lower, double upper);
            void   RNG_GetVec4         (RNG*, Vec4f* out, double lower, double upper);
            void   RNG_GetQuat         (RNG*, Quat* out);
        ]]
    end

    do -- Global Symbol Table
        RNG = {
            Create          = function(seed)
                local _instance = libphx.RNG_Create(seed)
                return Core.ManagedObject(_instance, libphx.RNG_Free)
            end,
            FromStr         = function(s)
                local _instance = libphx.RNG_FromStr(s)
                return Core.ManagedObject(_instance, libphx.RNG_Free)
            end,
            FromTime        = function()
                local _instance = libphx.RNG_FromTime()
                return Core.ManagedObject(_instance, libphx.RNG_Free)
            end,
        }

        if onDef_RNG then onDef_RNG(RNG, mt) end
        RNG = setmetatable(RNG, mt)
    end

    do -- Metatype for class instances
        local t  = ffi.typeof('RNG')
        local mt = {
            __index = {
                rewind          = libphx.RNG_Rewind,
                chance          = libphx.RNG_Chance,
                get31           = libphx.RNG_Get31,
                get32           = libphx.RNG_Get32,
                get64           = libphx.RNG_Get64,
                getAngle        = libphx.RNG_GetAngle,
                getInt          = libphx.RNG_GetInt,
                getRNG          = function(self)
                    local _instance = libphx.RNG_GetRNG(self)
                    return Core.ManagedObject(_instance, libphx.RNG_Free)
                end,
                getUniform      = libphx.RNG_GetUniform,
                getUniformRange = libphx.RNG_GetUniformRange,
                getErlang       = libphx.RNG_GetErlang,
                getExp          = libphx.RNG_GetExp,
                getGaussian     = libphx.RNG_GetGaussian,
                getAxis2        = libphx.RNG_GetAxis2,
                getAxis3        = libphx.RNG_GetAxis3,
                getDir2         = libphx.RNG_GetDir2,
                getDir3         = libphx.RNG_GetDir3,
                getDisc         = libphx.RNG_GetDisc,
                getSign         = libphx.RNG_GetSign,
                getSphere       = libphx.RNG_GetSphere,
                getVec2         = libphx.RNG_GetVec2,
                getVec3         = libphx.RNG_GetVec3,
                getVec4         = libphx.RNG_GetVec4,
                getQuat         = libphx.RNG_GetQuat,
            },
        }

        if onDef_RNG_t then onDef_RNG_t(t, mt) end
        RNG_t = ffi.metatype(t, mt)
    end

    return RNG
end

return Loader
