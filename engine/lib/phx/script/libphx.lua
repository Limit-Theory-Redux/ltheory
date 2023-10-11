local ffi = require('ffi')
local jit = require('jit')

local libphx = {}
do -- Basic Typedefs
  ffi.cdef [[
    typedef unsigned long  ulong;
    typedef unsigned int   uint;
    typedef unsigned short ushort;
    typedef unsigned char  uchar;
    typedef char const*    cstr;
    typedef int8_t         int8;
    typedef int16_t        int16;
    typedef int32_t        int32;
    typedef int64_t        int64;
    typedef uint8_t        uint8;
    typedef uint16_t       uint16;
    typedef uint32_t       uint32;
    typedef uint64_t       uint64;

    typedef int32          BlendMode;
    typedef uint8          BSPNodeRel;
    typedef int8           Button;
    typedef int32          CollisionGroup;
    typedef int32          CollisionMask;
    typedef int32          CubeFace;
    typedef int32          CullFace;
    typedef uint8          CursorControl;
    typedef uint8          CursorGrabMode;
    typedef uint8          CursorIcon;
    typedef int32          DataFormat;
    typedef int32          DeviceType;
    typedef uint8          DragAndDropEvent;
    typedef uint32         Error;
    typedef uint8          GamepadAxis;
    typedef uint8          GamepadButton;
    typedef uint64         GamepadId;
    typedef int32          HatDir;
    typedef uint8          InputDeviceType;
    typedef uchar          Key;
    typedef uint8          KeyboardButton;
    typedef int32          Metric;
    typedef int32          Modifier;
    typedef int32          MouseButton;
    typedef uint8          MouseControl;
    typedef uint8          PhysicsType;
    typedef int32          PixelFormat;
    typedef uint8          PointClassification;
    typedef uint8          PolygonClassification;
    typedef uint8          PresentMode;
    typedef int32          ResourceType;
    typedef int32          ShaderVarType;
    typedef int32          SocketType;
    typedef uint8          SoundState;
    typedef int32          State;
    typedef uint8          SystemEvent;
    typedef int32          TexFilter;
    typedef int32          TexFormat;
    typedef int32          TexWrapMode;
    typedef uint8          TouchpadAxis;
    typedef uint32         WindowMode;
    typedef int            WindowPos;
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
    typedef struct Audio        {} Audio;
    typedef struct BSP          {} BSP;
    typedef struct BoxMesh      {} BoxMesh;
    typedef struct BoxTree      {} BoxTree;
    typedef struct Bytes        {} Bytes;
    typedef struct Cursor       {} Cursor;
    typedef struct CursorState  {} CursorState;
    typedef struct Directory    {} Directory;
    typedef struct DragAndDropState {} DragAndDropState;
    typedef struct Engine       {} Engine;
    typedef struct File         {} File;
    typedef struct Font         {} Font;
    typedef struct GamepadState {} GamepadState;
    typedef struct HashGrid     {} HashGrid;
    typedef struct HashGridElem {} HashGridElem;
    typedef struct Input        {} Input;
    typedef struct InputBinding {} InputBinding;
    typedef struct InputDevice  {} InputDevice;
    typedef struct InputDeviceId {} InputDeviceId;
    typedef struct KDTree       {} KDTree;
    typedef struct KeyboardState {} KeyboardState;
    typedef struct LodMesh      {} LodMesh;
    typedef struct MemPool      {} MemPool;
    typedef struct MemStack     {} MemStack;
    typedef struct Mesh         {} Mesh;
    typedef struct MouseState   {} MouseState;
    typedef struct Octree       {} Octree;
    typedef struct Physics      {} Physics;
    typedef struct RNG          {} RNG;
    typedef struct RigidBody    {} RigidBody;
    typedef struct RmGui        {} RmGui;
    typedef struct SDF          {} SDF;
    typedef struct Shader       {} Shader;
    typedef struct ShaderState  {} ShaderState;
    typedef struct Socket       {} Socket;
    typedef struct Sound        {} Sound;
    typedef struct SoundDesc    {} SoundDesc;
    typedef struct StrMap       {} StrMap;
    typedef struct StrMapIter   {} StrMapIter;
    typedef struct SystemEventState {} SystemEventState;
    typedef struct Tex1D        {} Tex1D;
    typedef struct Tex2D        {} Tex2D;
    typedef struct Tex3D        {} Tex3D;
    typedef struct TexCube      {} TexCube;
    typedef struct Timer        {} Timer;
    typedef struct TimeStamp    {} TimeStamp;
    typedef struct TouchpadState {} TouchpadState;
    typedef struct Trigger      {} Trigger;
    typedef struct Window       {} Window;
  ]]

  libphx.Opaques = {
    'Audio',
    'BSP',
    'BoxMesh',
    'BoxTree',
    'Bytes',
    'Cursor',
    'CursorState',
    'Directory',
    'DragAndDropState',
    'Engine',
    'File',
    'Font',
    'GamepadState',
    'HashGrid',
    'HashGridElem',
    'Input',
    'InputBinding',
    'InputDevice',
    'InputDeviceId',
    'KDTree',
    'KeyboardState',
    'LodMesh',
    'MemPool',
    'MemStack',
    'Mesh',
    'MouseState',
    'Octree',
    'Physics',
    'RNG',
    'RigidBody',
    'RmGui',
    'SDF',
    'Shader',
    'ShaderState',
    'Socket',
    'Sound',
    'SoundDesc',
    'StrMap',
    'StrMapIter',
    'SystemEventState',
    'Tex1D',
    'Tex2D',
    'Tex3D',
    'TexCube',
    'Timer',
    'TimeStamp',
    'TouchpadState',
    'Trigger',
    'Window',
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
      int        index;
      int        count;
      RigidBody* body0;
      RigidBody* body1;
    } Collision;

    typedef struct Device {
      DeviceType type;
      uint32     id;
    } Device;

    typedef struct InputEvent {
      uint32     timestamp;
      DeviceType devicetype;
      uint32     deviceid;
      Button     button;
      float      value;
      State      state;
    } InputEvent;

    typedef struct IntersectSphereProfiling {
      int32                nodes;
      int32                leaves;
      int32                triangles;
      int32                triangleTests_size;
      int32                triangleTests_capacity;
      struct TriangleTest* triangleTests_data;
    } IntersectSphereProfiling;

    typedef struct LineSegment {
      float p0x;
      float p0y;
      float p0z;
      float p1x;
      float p1y;
      float p1z;
    } LineSegment;

    typedef struct Matrix {
      float m[16];
    } Matrix;

    typedef struct Plane {
      float nx;
      float ny;
      float nz;
      float d;
    } Plane;

    typedef struct Polygon {
      int32         vertices_size;
      int32         vertices_capacity;
      struct Vec3f* vertices_data;
    } Polygon;

    typedef struct Quat {
      float x;
      float y;
      float z;
      float w;
    } Quat;

    typedef struct Ray {
      float px;
      float py;
      float pz;
      float dirx;
      float diry;
      float dirz;
      float tMin;
      float tMax;
    } Ray;

    typedef struct RayCastResult {
      RigidBody* body;
      float      normx;
      float      normy;
      float      normz;
      float      posx;
      float      posy;
      float      posz;
      float      t;
    } RayCastResult;

    typedef struct ShapeCastResult {
      int32       hits_size;
      int32       hits_capacity;
      RigidBody** hits_data;
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
  ]]

  libphx.Structs = {
    'BSPNodeRef',
    'Box3d',
    'Box3f',
    'Box3i',
    'Collision',
    'Device',
    'InputEvent',
    'IntersectSphereProfiling',
    'LineSegment',
    'Matrix',
    'Plane',
    'Polygon',
    'Quat',
    'Ray',
    'RayCastResult',
    'ShapeCastResult',
    'Sphere',
    'Time',
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
