-- Mesh ------------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef struct Mesh {} Mesh;
    ]]

    return 1, 'Mesh'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local Mesh

    do -- C Definitions
        ffi.cdef [[
            void    Mesh_Free              (Mesh*);
            Mesh*   Mesh_Create            ();
            Mesh*   Mesh_Clone             (Mesh const*);
            Mesh*   Mesh_Load              (cstr name);
            Bytes*  Mesh_ToBytes           (Mesh const*);
            Mesh*   Mesh_FromBytes         (Bytes* buf);
            Mesh*   Mesh_FromObj           (cstr bytes);
            Mesh*   Mesh_Box               (int res);
            Mesh*   Mesh_BoxSphere         (int res);
            Mesh*   Mesh_Plane             (Vec3f origin, Vec3f du, Vec3f dv, int resU, int resV);
            void    Mesh_AddIndex          (Mesh*, int newIndex);
            void    Mesh_AddMesh           (Mesh*, Mesh const* other);
            void    Mesh_AddQuad           (Mesh*, int i1, int i2, int i3, int i4);
            void    Mesh_AddTri            (Mesh*, int i1, int i2, int i3);
            void    Mesh_AddVertex         (Mesh*, float px, float py, float pz, float nx, float ny, float nz, float u, float v);
            void    Mesh_AddVertexRaw      (Mesh*, Vertex const* vertex);
            void    Mesh_DrawBind          (Mesh*);
            void    Mesh_DrawBound         (Mesh const*);
            void    Mesh_DrawUnbind        (Mesh const*);
            void    Mesh_Draw              (Mesh*);
            void    Mesh_DrawNormals       (Mesh const*, float scale);
            void    Mesh_GetBound          (Mesh*, Box3f* out);
            void    Mesh_GetCenter         (Mesh*, Vec3f* out);
            int     Mesh_GetIndexCount     (Mesh const*);
            void    Mesh_LockIndexData     (Mesh*, void (*)(int*, uint64));
            float   Mesh_GetRadius         (Mesh*);
            uint64  Mesh_GetVersion        (Mesh const*);
            void    Mesh_IncVersion        (Mesh*);
            uint32  Mesh_Validate          (Mesh const*);
            Vertex* Mesh_GetVertex         (Mesh*, int index);
            int     Mesh_GetVertexCount    (Mesh const*);
            void    Mesh_LockVertexData    (Mesh*, void (*)(Vertex*, uint64));
            void    Mesh_ReserveIndexData  (Mesh*, int capacity);
            void    Mesh_ReserveVertexData (Mesh*, int capacity);
            Mesh*   Mesh_Center            (Mesh*);
            Mesh*   Mesh_Invert            (Mesh*);
            Mesh*   Mesh_RotateX           (Mesh*, float rads);
            Mesh*   Mesh_RotateY           (Mesh*, float rads);
            Mesh*   Mesh_RotateZ           (Mesh*, float rads);
            Mesh*   Mesh_RotateYPR         (Mesh*, float yaw, float pitch, float roll);
            Mesh*   Mesh_Scale             (Mesh*, float x, float y, float z);
            Mesh*   Mesh_ScaleUniform      (Mesh*, float s);
            Mesh*   Mesh_Translate         (Mesh*, float x, float y, float z);
            void    Mesh_Transform         (Mesh*, Matrix const* matrix);
            void    Mesh_ComputeNormals    (Mesh*);
            void    Mesh_SplitNormals      (Mesh*, float minDot);
            void    Mesh_ComputeAO         (Mesh*, float radius);
            void    Mesh_ComputeOcclusion  (Mesh*, Tex3D* sdf, float radius);
        ]]
    end

    do -- Global Symbol Table
        Mesh = {
            Create            = function()
                local _instance = libphx.Mesh_Create()
                return Core.ManagedObject(_instance, libphx.Mesh_Free)
            end,
            Load              = function(name)
                local _instance = libphx.Mesh_Load(name)
                return Core.ManagedObject(_instance, libphx.Mesh_Free)
            end,
            FromBytes         = function(buf)
                local _instance = libphx.Mesh_FromBytes(buf)
                return Core.ManagedObject(_instance, libphx.Mesh_Free)
            end,
            FromObj           = function(bytes)
                local _instance = libphx.Mesh_FromObj(bytes)
                return Core.ManagedObject(_instance, libphx.Mesh_Free)
            end,
            Box               = function(res)
                local _instance = libphx.Mesh_Box(res)
                return Core.ManagedObject(_instance, libphx.Mesh_Free)
            end,
            BoxSphere         = function(res)
                local _instance = libphx.Mesh_BoxSphere(res)
                return Core.ManagedObject(_instance, libphx.Mesh_Free)
            end,
            Plane             = function(origin, du, dv, resU, resV)
                local _instance = libphx.Mesh_Plane(origin, du, dv, resU, resV)
                return Core.ManagedObject(_instance, libphx.Mesh_Free)
            end,
        }

        if onDef_Mesh then onDef_Mesh(Mesh, mt) end
        Mesh = setmetatable(Mesh, mt)
    end

    do -- Metatype for class instances
        local t  = ffi.typeof('Mesh')
        local mt = {
            __index = {
                clone             = function(self)
                    local _instance = libphx.Mesh_Clone(self)
                    return Core.ManagedObject(_instance, libphx.Mesh_Free)
                end,
                toBytes           = function(self)
                    local _instance = libphx.Mesh_ToBytes(self)
                    return Core.ManagedObject(_instance, libphx.Bytes_Free)
                end,
                addIndex          = libphx.Mesh_AddIndex,
                addMesh           = libphx.Mesh_AddMesh,
                addQuad           = libphx.Mesh_AddQuad,
                addTri            = libphx.Mesh_AddTri,
                addVertex         = libphx.Mesh_AddVertex,
                addVertexRaw      = libphx.Mesh_AddVertexRaw,
                drawBind          = libphx.Mesh_DrawBind,
                drawBound         = libphx.Mesh_DrawBound,
                drawUnbind        = libphx.Mesh_DrawUnbind,
                draw              = libphx.Mesh_Draw,
                drawNormals       = libphx.Mesh_DrawNormals,
                getBound          = libphx.Mesh_GetBound,
                getCenter         = libphx.Mesh_GetCenter,
                getIndexCount     = libphx.Mesh_GetIndexCount,
                lockIndexData     = libphx.Mesh_LockIndexData,
                getRadius         = libphx.Mesh_GetRadius,
                getVersion        = libphx.Mesh_GetVersion,
                incVersion        = libphx.Mesh_IncVersion,
                validate          = libphx.Mesh_Validate,
                getVertex         = libphx.Mesh_GetVertex,
                getVertexCount    = libphx.Mesh_GetVertexCount,
                lockVertexData    = libphx.Mesh_LockVertexData,
                reserveIndexData  = libphx.Mesh_ReserveIndexData,
                reserveVertexData = libphx.Mesh_ReserveVertexData,
                center            = libphx.Mesh_Center,
                invert            = libphx.Mesh_Invert,
                rotateX           = libphx.Mesh_RotateX,
                rotateY           = libphx.Mesh_RotateY,
                rotateZ           = libphx.Mesh_RotateZ,
                rotateYPR         = libphx.Mesh_RotateYPR,
                scale             = libphx.Mesh_Scale,
                scaleUniform      = libphx.Mesh_ScaleUniform,
                translate         = libphx.Mesh_Translate,
                transform         = libphx.Mesh_Transform,
                computeNormals    = libphx.Mesh_ComputeNormals,
                splitNormals      = libphx.Mesh_SplitNormals,
                computeAO         = libphx.Mesh_ComputeAO,
                computeOcclusion  = libphx.Mesh_ComputeOcclusion,
            },
        }

        if onDef_Mesh_t then onDef_Mesh_t(t, mt) end
        Mesh_t = ffi.metatype(t, mt)
    end

    return Mesh
end

return Loader
