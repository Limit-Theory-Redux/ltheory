-- AUTO GENERATED. DO NOT MODIFY!
-- Matrix ----------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef struct Matrix {
            float m[16];
        } Matrix;
    ]]

    return 1, 'Matrix'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local Matrix

    do -- C Definitions
        ffi.cdef [[
            void    Matrix_Free               (Matrix*);
            bool    Matrix_Equal              (Matrix const*, Matrix const* other);
            bool    Matrix_ApproximatelyEqual (Matrix const*, Matrix const* other);
            Matrix* Matrix_Inverse            (Matrix const*);
            Matrix* Matrix_InverseTranspose   (Matrix const*);
            Matrix* Matrix_Sum                (Matrix const*, Matrix const* other);
            Matrix* Matrix_Transpose          (Matrix const*);
            void    Matrix_IInverse           (Matrix*);
            void    Matrix_IScale             (Matrix*, float scale);
            void    Matrix_ITranspose         (Matrix*);
            Matrix* Matrix_Identity           ();
            Matrix* Matrix_LookAt             (Vec3f const* pos, Vec3f const* at, Vec3f const* up);
            Matrix* Matrix_LookUp             (Vec3f const* pos, Vec3f const* look, Vec3f const* up);
            Matrix* Matrix_Perspective        (float degreesFovy, float aspect, float n, float f);
            Matrix* Matrix_Product            (Matrix const*, Matrix const* other);
            Matrix* Matrix_RotationX          (float rads);
            Matrix* Matrix_RotationY          (float rads);
            Matrix* Matrix_RotationZ          (float rads);
            Matrix* Matrix_Scaling            (float sx, float sy, float sz);
            Matrix* Matrix_SRT                (float sx, float sy, float sz, float ry, float rp, float rr, float tx, float ty, float tz);
            Matrix* Matrix_Translation        (float tx, float ty, float tz);
            Matrix* Matrix_YawPitchRoll       (float yaw, float pitch, float roll);
            void    Matrix_MulBox             (Matrix const*, Box3f const* in0, Box3f* out);
            void    Matrix_MulDir             (Matrix const*, Vec3f const* d, Vec3f* out);
            void    Matrix_MulPoint           (Matrix const*, Vec3f const* p, Vec3f* out);
            void    Matrix_MulVec             (Matrix const*, Vec4f const* v, Vec4f* out);
            void    Matrix_GetForward         (Matrix const*, Vec3f* out);
            void    Matrix_GetRight           (Matrix const*, Vec3f* out);
            void    Matrix_GetUp              (Matrix const*, Vec3f* out);
            void    Matrix_GetPos             (Matrix const*, Vec3f* out);
            void    Matrix_GetRow             (Matrix const*, int row, Vec4f* out);
            Matrix* Matrix_FromBasis          (Vec3f const* x, Vec3f const* y, Vec3f const* z);
            Matrix* Matrix_FromPosRot         (Vec3f const* pos, Quat const* rot);
            Matrix* Matrix_FromPosRotScale    (Vec3f const* pos, Quat const* rot, float scale);
            Matrix* Matrix_FromPosBasis       (Vec3f const* pos, Vec3f const* x, Vec3f const* y, Vec3f const* z);
            Matrix* Matrix_FromQuat           (Quat const* q);
            Quat*   Matrix_ToQuat             (Matrix const*);
            void    Matrix_Print              (Matrix const*);
            cstr    Matrix_ToString           (Matrix const*);
        ]]
    end

    do -- Global Symbol Table
        Matrix = {
            Identity           = function()
                local _instance = libphx.Matrix_Identity()
                return Core.ManagedObject(_instance, libphx.Matrix_Free)
            end,
            LookAt             = function(pos, at, up)
                local _instance = libphx.Matrix_LookAt(pos, at, up)
                return Core.ManagedObject(_instance, libphx.Matrix_Free)
            end,
            LookUp             = function(pos, look, up)
                local _instance = libphx.Matrix_LookUp(pos, look, up)
                return Core.ManagedObject(_instance, libphx.Matrix_Free)
            end,
            Perspective        = function(degreesFovy, aspect, n, f)
                local _instance = libphx.Matrix_Perspective(degreesFovy, aspect, n, f)
                return Core.ManagedObject(_instance, libphx.Matrix_Free)
            end,
            RotationX          = function(rads)
                local _instance = libphx.Matrix_RotationX(rads)
                return Core.ManagedObject(_instance, libphx.Matrix_Free)
            end,
            RotationY          = function(rads)
                local _instance = libphx.Matrix_RotationY(rads)
                return Core.ManagedObject(_instance, libphx.Matrix_Free)
            end,
            RotationZ          = function(rads)
                local _instance = libphx.Matrix_RotationZ(rads)
                return Core.ManagedObject(_instance, libphx.Matrix_Free)
            end,
            Scaling            = function(sx, sy, sz)
                local _instance = libphx.Matrix_Scaling(sx, sy, sz)
                return Core.ManagedObject(_instance, libphx.Matrix_Free)
            end,
            SRT                = function(sx, sy, sz, ry, rp, rr, tx, ty, tz)
                local _instance = libphx.Matrix_SRT(sx, sy, sz, ry, rp, rr, tx, ty, tz)
                return Core.ManagedObject(_instance, libphx.Matrix_Free)
            end,
            Translation        = function(tx, ty, tz)
                local _instance = libphx.Matrix_Translation(tx, ty, tz)
                return Core.ManagedObject(_instance, libphx.Matrix_Free)
            end,
            YawPitchRoll       = function(yaw, pitch, roll)
                local _instance = libphx.Matrix_YawPitchRoll(yaw, pitch, roll)
                return Core.ManagedObject(_instance, libphx.Matrix_Free)
            end,
            FromBasis          = function(x, y, z)
                local _instance = libphx.Matrix_FromBasis(x, y, z)
                return Core.ManagedObject(_instance, libphx.Matrix_Free)
            end,
            FromPosRot         = function(pos, rot)
                local _instance = libphx.Matrix_FromPosRot(pos, rot)
                return Core.ManagedObject(_instance, libphx.Matrix_Free)
            end,
            FromPosRotScale    = function(pos, rot, scale)
                local _instance = libphx.Matrix_FromPosRotScale(pos, rot, scale)
                return Core.ManagedObject(_instance, libphx.Matrix_Free)
            end,
            FromPosBasis       = function(pos, x, y, z)
                local _instance = libphx.Matrix_FromPosBasis(pos, x, y, z)
                return Core.ManagedObject(_instance, libphx.Matrix_Free)
            end,
            FromQuat           = function(q)
                local _instance = libphx.Matrix_FromQuat(q)
                return Core.ManagedObject(_instance, libphx.Matrix_Free)
            end,
        }

        local mt = {
            __call = function(t, ...) return Matrix_t(...) end,
        }

        if onDef_Matrix then onDef_Matrix(Matrix, mt) end
        Matrix = setmetatable(Matrix, mt)
    end

    do -- Metatype for class instances
        local t  = ffi.typeof('Matrix')
        local mt = {
            __tostring = function(self) return ffi.string(libphx.Matrix_ToString(self)) end,
            __index = {
                clone              = function(x) return Matrix_t(x) end,
                equal              = libphx.Matrix_Equal,
                approximatelyEqual = libphx.Matrix_ApproximatelyEqual,
                inverse            = function(self)
                    local _instance = libphx.Matrix_Inverse(self)
                    return Core.ManagedObject(_instance, libphx.Matrix_Free)
                end,
                inverseTranspose   = function(self)
                    local _instance = libphx.Matrix_InverseTranspose(self)
                    return Core.ManagedObject(_instance, libphx.Matrix_Free)
                end,
                sum                = function(self, other)
                    local _instance = libphx.Matrix_Sum(self, other)
                    return Core.ManagedObject(_instance, libphx.Matrix_Free)
                end,
                transpose          = function(self)
                    local _instance = libphx.Matrix_Transpose(self)
                    return Core.ManagedObject(_instance, libphx.Matrix_Free)
                end,
                iInverse           = libphx.Matrix_IInverse,
                iScale             = libphx.Matrix_IScale,
                iTranspose         = libphx.Matrix_ITranspose,
                product            = function(self, other)
                    local _instance = libphx.Matrix_Product(self, other)
                    return Core.ManagedObject(_instance, libphx.Matrix_Free)
                end,
                mulBox             = libphx.Matrix_MulBox,
                mulDir             = libphx.Matrix_MulDir,
                mulPoint           = libphx.Matrix_MulPoint,
                mulVec             = libphx.Matrix_MulVec,
                getForward         = libphx.Matrix_GetForward,
                getRight           = libphx.Matrix_GetRight,
                getUp              = libphx.Matrix_GetUp,
                getPos             = libphx.Matrix_GetPos,
                getRow             = libphx.Matrix_GetRow,
                toQuat             = function(self)
                    local _instance = libphx.Matrix_ToQuat(self)
                    return Core.ManagedObject(_instance, libphx.Quat_Free)
                end,
                print              = libphx.Matrix_Print,
                toString           = libphx.Matrix_ToString,
            },
        }

        if onDef_Matrix_t then onDef_Matrix_t(t, mt) end
        Matrix_t = ffi.metatype(t, mt)
    end

    return Matrix
end

return Loader
