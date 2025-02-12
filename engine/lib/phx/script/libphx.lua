local ffi = require('ffi')
local jit = require('jit')

local libphx = {}
do -- Basic Typedefs
    ffi.cdef [[
    typedef uint8          BSPNodeRel;
    typedef int32          CollisionGroup;
    typedef int32          CollisionMask;
    typedef int32          CubeFace;
    typedef uint8          CursorControl;
    typedef int32          DataFormat;
    typedef uint32         Error;
    typedef uint64         GamepadId;
    typedef uchar          Key;
    typedef int32          Metric;
    typedef int32          Modifier;
    typedef uint8          PhysicsType;
    typedef int32          PixelFormat;
    typedef uint8          PointClassification;
    typedef uint8          PolygonClassification;
    typedef int32          ResourceType;
    typedef int32          SocketType;
    typedef uint8          SoundState;
    typedef int32          State;
    typedef int32          TexFilter;
    typedef int32          TexFormat;
    typedef int32          TexWrapMode;
  ]]
end

do -- Function Pointer Typedefs
    ffi.cdef [[
    typedef void (*ValueForeach) (void* value, void* userData);
    typedef int  (*ThreadFn    ) (void* data);
    typedef int  (*ThreadPoolFn) (int threadIndex, int threadCount, void* data);
  ]]
end

do -- Opaque Structs
    ffi.cdef [[
    typedef struct BSP          {} BSP;
    typedef struct BoxTree      {} BoxTree;
    typedef struct File         {} File;
    typedef struct HashGrid     {} HashGrid;
    typedef struct HashGridElem {} HashGridElem;
    typedef struct InputBinding {} InputBinding;
    typedef struct KDTree       {} KDTree;
    typedef struct MemPool      {} MemPool;
    typedef struct MemStack     {} MemStack;
    typedef struct Octree       {} Octree;
    typedef struct RNG          {} RNG;
    typedef struct RmGui        {} RmGui;
    typedef struct SDF          {} SDF;
    typedef struct Socket       {} Socket;
  ]]

    libphx.Opaques = {
        'BSP',
        'BoxTree',
        'File',
        'HashGrid',
        'HashGridElem',
        'KDTree',
        'MemPool',
        'MemStack',
        'Octree',
        'RNG',
        'RmGui',
        'SDF',
        'Socket',
    }
end

do -- Transparent Structs
    ffi.cdef [[
    typedef struct BSPNodeRef {
      int32 index;
      uint8 triangleCount;
    } BSPNodeRef;

    typedef struct Box3d {
      double lowerx;
      double lowery;
      double lowerz;
      double upperx;
      double uppery;
      double upperz;
    } Box3d;

    typedef struct Box3f {
      float lowerx;
      float lowery;
      float lowerz;
      float upperx;
      float uppery;
      float upperz;
    } Box3f;

    typedef struct Box3i {
      int lowerx;
      int lowery;
      int lowerz;
      int upperx;
      int uppery;
      int upperz;
    } Box3i;

    typedef struct Collision {
      uint32     index;
      uint32     count;
      RigidBody* body0;
      RigidBody* body1;
    } Collision;

    typedef struct IntersectSphereProfiling {
      int32                nodes;
      int32                leaves;
      int32                triangles;
      int32                triangleTests_size;
      int32                triangleTests_capacity;
      struct TriangleTest* triangleTests_data;
    } IntersectSphereProfiling;

    typedef struct LineSegment {
      double p0x;
      double p0y;
      double p0z;
      double p1x;
      double p1y;
      double p1z;
    } LineSegment;

    typedef struct Ray {
      double px;
      double py;
      double pz;
      double dirx;
      double diry;
      double dirz;
      double tMin;
      double tMax;
    } Ray;

    typedef struct RayCastResult {
      RigidBody* body;
      float      normx;
      float      normy;
      float      normz;
      double     posx;
      double     posy;
      double     posz;
      float      t;
    } RayCastResult;

    typedef struct ShapeCastResult {
      RigidBody** hits;
      uint32      hits_len;
    } ShapeCastResult;

    typedef struct Sphere {
      float px;
      float py;
      float pz;
      float r;
    } Sphere;

    typedef struct Time {
      int second;
      int minute;
      int hour;
      int dayOfWeek;
      int dayOfMonth;
      int dayOfYear;
      int month;
      int year;
    } Time;

    typedef struct Vec3f {
      float x;
      float y;
      float z;
    } Vec3f;

    typedef struct Triangle {
      Vec3f vertices[3];
    } Triangle;

    typedef struct TriangleTest {
      struct Triangle* triangle;
      bool             hit;
    } TriangleTest;

    typedef struct Vec2d {
      double x;
      double y;
    } Vec2d;

    typedef struct Vec2f {
      float x;
      float y;
    } Vec2f;

    typedef struct Vec2i {
      int x;
      int y;
    } Vec2i;

    typedef struct Vec2u {
      uint x;
      uint y;
    } Vec2u;

    typedef struct Vec3d {
      double x;
      double y;
      double z;
    } Vec3d;

    typedef struct Vec3i {
      int x;
      int y;
      int z;
    } Vec3i;

    typedef struct Vec3u {
      uint x;
      uint y;
      uint z;
    } Vec3u;

    typedef struct Vec4d {
      double x;
      double y;
      double z;
      double w;
    } Vec4d;

    typedef struct Vec4f {
      float x;
      float y;
      float z;
      float w;
    } Vec4f;

    typedef struct Vec4i {
      int x;
      int y;
      int z;
      int w;
    } Vec4i;

    typedef struct Vec4u {
      uint x;
      uint y;
      uint z;
      uint w;
    } Vec4u;

    typedef struct Vertex {
      float px;
      float py;
      float pz;
      float nx;
      float ny;
      float nz;
      float uvx;
      float uvy;
    } Vertex;

    typedef struct Color {
      float r;
      float g;
      float b;
      float a;
    } Color;

    typedef struct Position {
      double x;
      double y;
      double z;
    } Position;
  ]]

    libphx.Structs = {
        'BSPNodeRef',
        'Box3d',
        'Box3f',
        'Box3i',
        'Collision',
        'IntersectSphereProfiling',
        'LineSegment',
        'Ray',
        'RayCastResult',
        'ShapeCastResult',
        'Sphere',
        'Vec3f',
        'Triangle',
        'TriangleTest',
        'Vec2d',
        'Vec2f',
        'Vec2i',
        'Vec3d',
        'Vec3i',
        'Vec4d',
        'Vec4f',
        'Vec4i',
        'Vertex',
        'Color',
        'Position',
    }
end

do -- Load Library
    libphx.lib = ffi.C
    -- local debug = __debug__ and 'd' or ''
    -- local path = string.format('phx%s', debug)
    -- libphx.lib = ffi.load(path, false)
    -- assert(libphx.lib, 'Failed to load %s', path)
end

return libphx
