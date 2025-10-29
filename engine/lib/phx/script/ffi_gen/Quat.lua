-- AUTO GENERATED. DO NOT MODIFY!
-- Quat ------------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef struct Quat {
            float x;
            float y;
            float z;
            float w;
        } Quat;
    ]]

    return 1, 'Quat'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local Quat

    do -- C Definitions
        ffi.cdef [[
            void   Quat_Free               (Quat*);
            Quat*  Quat_Create             (float x, float y, float z, float w);
            Quat*  Quat_Identity           ();
            void   Quat_GetAxisX           (Quat const*, Vec3f* out);
            void   Quat_GetAxisY           (Quat const*, Vec3f* out);
            void   Quat_GetAxisZ           (Quat const*, Vec3f* out);
            void   Quat_GetForward         (Quat const*, Vec3f* out);
            void   Quat_GetRight           (Quat const*, Vec3f* out);
            void   Quat_GetUp              (Quat const*, Vec3f* out);
            Quat*  Quat_Canonicalize       (Quat const*);
            void   Quat_ICanonicalize      (Quat*);
            float  Quat_Dot                (Quat const*, Quat const* p);
            bool   Quat_Equal              (Quat const*, Quat const* p);
            bool   Quat_ApproximatelyEqual (Quat const*, Quat const* p);
            Quat*  Quat_Inverse            (Quat const*);
            void   Quat_IInverse           (Quat*);
            Quat*  Quat_Lerp               (Quat const*, Quat const* p, float t);
            void   Quat_ILerp              (Quat*, Quat const* p, float t);
            Quat*  Quat_Mul                (Quat const*, Quat const* p);
            void   Quat_IMul               (Quat*, Quat const* p);
            void   Quat_MulV               (Quat const*, Vec3f const* v, Vec3f* out);
            Quat*  Quat_Normalize          (Quat const*);
            void   Quat_INormalize         (Quat*);
            Quat*  Quat_Scale              (Quat const*, float scale);
            void   Quat_IScale             (Quat*, float scale);
            Quat*  Quat_Slerp              (Quat const*, Quat const* p, float t);
            void   Quat_ISlerp             (Quat*, Quat const* p, float t);
            Quat*  Quat_FromAxisAngle      (Vec3f const* axis, float radians);
            Quat*  Quat_FromEuler          (float yaw, float pitch, float roll);
            Quat*  Quat_FromLook           (Vec3f const* forward, Vec3f const* up);
            Quat*  Quat_LookAt             (Vec3f const* eye, Vec3f const* target, Vec3f const* up);
            Quat*  Quat_FromRotateTo       (Vec3f const* from, Vec3f const* to);
            cstr   Quat_ToString           (Quat const*);
            Error* Quat_Validate           (Quat const*);
        ]]
    end

    do -- Global Symbol Table
        Quat = {
            Create             = function(x, y, z, w)
                local _instance = libphx.Quat_Create(x, y, z, w)
                return Core.ManagedObject(_instance, libphx.Quat_Free)
            end,
            Identity           = function()
                local _instance = libphx.Quat_Identity()
                return Core.ManagedObject(_instance, libphx.Quat_Free)
            end,
            FromAxisAngle      = function(axis, radians)
                local _instance = libphx.Quat_FromAxisAngle(axis, radians)
                return Core.ManagedObject(_instance, libphx.Quat_Free)
            end,
            FromEuler          = function(yaw, pitch, roll)
                local _instance = libphx.Quat_FromEuler(yaw, pitch, roll)
                return Core.ManagedObject(_instance, libphx.Quat_Free)
            end,
            FromLook           = function(forward, up)
                local _instance = libphx.Quat_FromLook(forward, up)
                return Core.ManagedObject(_instance, libphx.Quat_Free)
            end,
            LookAt             = function(eye, target, up)
                local _instance = libphx.Quat_LookAt(eye, target, up)
                return Core.ManagedObject(_instance, libphx.Quat_Free)
            end,
            FromRotateTo       = function(from, to)
                local _instance = libphx.Quat_FromRotateTo(from, to)
                return Core.ManagedObject(_instance, libphx.Quat_Free)
            end,
        }

        local mt = {
            __call = function(t, ...) return Quat_t(...) end,
        }

        if onDef_Quat then onDef_Quat(Quat, mt) end
        Quat = setmetatable(Quat, mt)
    end

    do -- Metatype for class instances
        local t  = ffi.typeof('Quat')
        local mt = {
            __tostring = function(self) return ffi.string(libphx.Quat_ToString(self)) end,
            __index = {
                clone              = function(x) return Quat_t(x) end,
                getAxisX           = libphx.Quat_GetAxisX,
                getAxisY           = libphx.Quat_GetAxisY,
                getAxisZ           = libphx.Quat_GetAxisZ,
                getForward         = libphx.Quat_GetForward,
                getRight           = libphx.Quat_GetRight,
                getUp              = libphx.Quat_GetUp,
                canonicalize       = function(self)
                    local _instance = libphx.Quat_Canonicalize(self)
                    return Core.ManagedObject(_instance, libphx.Quat_Free)
                end,
                iCanonicalize      = libphx.Quat_ICanonicalize,
                dot                = libphx.Quat_Dot,
                equal              = libphx.Quat_Equal,
                approximatelyEqual = libphx.Quat_ApproximatelyEqual,
                inverse            = function(self)
                    local _instance = libphx.Quat_Inverse(self)
                    return Core.ManagedObject(_instance, libphx.Quat_Free)
                end,
                iInverse           = libphx.Quat_IInverse,
                lerp               = function(self, p, t)
                    local _instance = libphx.Quat_Lerp(self, p, t)
                    return Core.ManagedObject(_instance, libphx.Quat_Free)
                end,
                iLerp              = libphx.Quat_ILerp,
                mul                = function(self, p)
                    local _instance = libphx.Quat_Mul(self, p)
                    return Core.ManagedObject(_instance, libphx.Quat_Free)
                end,
                iMul               = libphx.Quat_IMul,
                mulV               = libphx.Quat_MulV,
                normalize          = function(self)
                    local _instance = libphx.Quat_Normalize(self)
                    return Core.ManagedObject(_instance, libphx.Quat_Free)
                end,
                iNormalize         = libphx.Quat_INormalize,
                scale              = function(self, scale)
                    local _instance = libphx.Quat_Scale(self, scale)
                    return Core.ManagedObject(_instance, libphx.Quat_Free)
                end,
                iScale             = libphx.Quat_IScale,
                slerp              = function(self, p, t)
                    local _instance = libphx.Quat_Slerp(self, p, t)
                    return Core.ManagedObject(_instance, libphx.Quat_Free)
                end,
                iSlerp             = libphx.Quat_ISlerp,
                toString           = libphx.Quat_ToString,
                validate           = function(self)
                    local _instance = libphx.Quat_Validate(self)
                    return Core.ManagedObject(_instance, libphx.Error_Free)
                end,
            },
        }

        if onDef_Quat_t then onDef_Quat_t(t, mt) end
        Quat_t = ffi.metatype(t, mt)
    end

    return Quat
end

return Loader
