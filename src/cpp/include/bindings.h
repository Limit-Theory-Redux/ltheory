#include <cstdarg>
#include <cstdint>
#include <cstdlib>
#include <ostream>
#include <new>

constexpr static const uintptr_t DeviceType_COUNT = 4;

constexpr static const uintptr_t GamepadAxis_SIZE = 6;

constexpr static const uintptr_t GamepadButton_SIZE = 15;

constexpr static const uint32_t Layout_None = 0;

constexpr static const uint32_t Layout_Stack = 1;

constexpr static const uint32_t Layout_Vertical = 2;

constexpr static const uint32_t Layout_Horizontal = 3;

constexpr static const uint32_t Widget_Group = 0;

constexpr static const uint32_t Widget_Text = 1;

constexpr static const uint32_t Widget_Rect = 2;

constexpr static const uint32_t Widget_Image = 3;

constexpr static const uint32_t FocusStyle_None = 0;

constexpr static const uint32_t FocusStyle_Fill = 1;

constexpr static const uint32_t FocusStyle_Outline = 2;

constexpr static const uint32_t FocusStyle_Underline = 3;

constexpr static const int32_t FocusType_Mouse = 0;

constexpr static const int32_t FocusType_Scroll = 1;

constexpr static const int32_t FocusType_SIZE = 2;

constexpr static const int32_t FocusType_Keyboard = 1;

constexpr static const float PLANE_THICKNESS_EPSILON = 1e-4;

constexpr static const float POINT_INTERSECTION_EPSILON = (2.0 * PLANE_THICKNESS_EPSILON);

constexpr static const float RAY_INTERSECTION_EPSILON = (8.0 * PLANE_THICKNESS_EPSILON);

constexpr static const float SPHERE_INTERSECTION_EPSILON = (2.0 * PLANE_THICKNESS_EPSILON);

constexpr static const uintptr_t ResourceType_COUNT = 10;

enum class PointClassification : uint8_t {
  InFront = 1,
  Behind = 2,
  Coplanar = 3,
};

enum class PolygonClassification : uint8_t {
  InFront = 1,
  Behind = 2,
  Coplanar = 3,
  Straddling = 4,
};

template<typename T = void>
struct Box;

template<typename T = void>
struct Option;

template<typename T = void>
struct Vec;

struct SoundDesc {
  uint32_t _refCount;
  FMOD_SOUND *handle;
  const char *name;
  const char *path;
};

using SoundState = uint8_t;

struct Sound {
  SoundDesc *desc;
  FMOD_CHANNEL *handle;
  SoundState state;
  const Vec3 *autoPos;
  const Vec3 *autoVel;
  bool freeOnFinish;
};

struct BSPNodeRef {
  int32_t index;
  uint8_t triangleCount;
};

struct Plane {
  Vec3 n;
  float d;
};

struct BSPNode {
  Plane plane;
  BSPNodeRef child[2];
};

struct Triangle {
  Vec3 vertices[3];
};

struct BSP {
  BSPNodeRef rootNode;
  BSPNodeRef emptyLeaf;
  Vec<BSPNode> nodes;
  Vec<Triangle> triangles;
};

struct Ray {
  Vec3 p;
  Vec3 dir;
  float tMin;
  float tMax;
};

struct LineSegment {
  Vec3 p0;
  Vec3 p1;
};

struct Sphere {
  Vec3 p;
  float r;
};

struct Box3 {
  Vec3 lower;
  Vec3 upper;
};

struct Computed {
  Box3 bound;
  float radius;
};

struct Vertex {
  Vec3 p;
  Vec3 n;
  Vec2 uv;
};

struct Mesh {
  uint32_t _refCount;
  uint32_t vbo;
  uint32_t ibo;
  uint64_t version;
  uint64_t versionBuffers;
  uint64_t versionInfo;
  Computed info;
  Vec<int32_t> index;
  Vec<Vertex> vertex;
};

using BSPNodeRel = uint8_t;

struct TriangleTest {
  Triangle *triangle;
  bool hit;
};

struct IntersectSphereProfiling {
  int32_t nodes;
  int32_t leaves;
  int32_t triangles;
  Vec<TriangleTest> triangleTests;
};

using BlendMode = int32_t;

struct Box_0 {
  Vec3 p;
  Vec3 s;
  Vec3 r;
  Vec3 b;
};

struct BoxMesh {
  Vec<Box_0> elem;
};

struct Node {
  Box3 box3;
  void *data;
  Node *sub[2];
};

struct BoxTree {
  Node *root;
};

struct Matrix {
  float m[16];
};

using DeviceType = int32_t;

using Button = int32_t;

struct Bytes {
  uint32_t size;
  uint32_t cursor;
  char data;
};

struct ClipRect {
  float x;
  float y;
  float sx;
  float sy;
  bool enabled;
};

using CubeFace = int32_t;

using CullFace = int32_t;

using DataFormat = int32_t;

struct Device {
  DeviceType ty;
  uint32_t id;
};

struct Directory {
  ReadDir iterator;
  Option<CString> lastEntry;
};

using Error = uint32_t;

struct File {
  File file;
};

struct HashMap {
  Node *elems;
  uint32_t size;
  uint32_t capacity;
  uint32_t mask;
  uint32_t keySize;
  uint32_t maxProbe;
};

using TexFormat = int32_t;

struct Tex2D {
  uint32_t _refCount;
  uint32_t handle;
  IVec2 size;
  TexFormat format;
};

struct Glyph {
  int32_t index;
  Tex2D *tex;
  int32_t x0;
  int32_t y0;
  int32_t x1;
  int32_t y1;
  int32_t sx;
  int32_t sy;
  int32_t advance;
};

struct Font {
  uint32_t _refCount;
  FT_Face handle;
  HashMap *glyphs;
  Glyph *glyphsAscii[256];
};

using TimeStamp = uint64_t;

struct Gamepad {
  Gamepad **gamepadList_prev;
  Gamepad *gamepadList_next;
  SDL_GameController *handle;
  TimeStamp lastActive;
  double axisState[GamepadAxis_SIZE];
  double axisLast[GamepadAxis_SIZE];
  double deadzone[GamepadAxis_SIZE];
  bool buttonState[GamepadButton_SIZE];
  bool buttonLast[GamepadButton_SIZE];
};

using GamepadAxis = SDL_GameControllerAxis;

using GamepadButton = SDL_GameControllerButton;

struct HashGridElem {
  uint64_t version;
  void *object;
  int32_t lower[3];
  int32_t upper[3];
};

struct HashGridCell {
  uint64_t version;
  Vec<HashGridElem*> elems;
};

struct MemPool {
  uint32_t size;
  uint32_t capacity;
  void *freeList;
  uint32_t cellSize;
  uint32_t blockSize;
  uint16_t blockCount;
  void **blocks;
};

struct HashGrid {
  uint64_t version;
  Vec<HashGridCell> cells;
  MemPool *elemPool;
  uint32_t cellCount;
  float cellSize;
  uint32_t mask;
  Vec<void*> results;
};

using ValueForeach = void(*)(void*, void*);

using Modifier = int32_t;

using State = int32_t;

struct InputEvent {
  uint32_t timestamp;
  Device device;
  Button button;
  float value;
  State state;
};

struct RawButton {
  Button button;
  float value;
};

using Lua = lua_State;

using lua_Integer = ptrdiff_t;

using LuaRef = lua_Integer;

struct AggregateButton {
  State state;
  LuaRef onPressed;
  LuaRef onDown;
  LuaRef onReleased;
};

struct AggregateAxis {
  float value;
  bool invert;
  LuaRef onChanged;
};

struct AggregateAxis2D {
  Vec2 value;
  LuaRef onChanged;
};

struct InputBinding {
  const char *name;
  RawButton rawButtons[BindCount][4];
  float pressThreshold;
  float releaseThreshold;
  float exponent;
  float deadzone;
  float minValue;
  float maxValue;
  Lua *luaInstance;
  AggregateButton buttons[4];
  AggregateAxis axes[2];
  AggregateAxis2D axis2D;
};

struct Joystick {
  SDL_Joystick *handle;
  const char *guid;
  int32_t axes;
  int32_t balls;
  int32_t buttons;
  int32_t hats;
  bool *buttonStates;
  bool *axisAlive;
  double *axisStates;
  TimeStamp lastUsed;
};

using HatDir = uint32_t;

struct KDTree {
  Box3 box_0;
  KDTree *back;
  KDTree *front;
  Node *elems;
};

using Key = uint8_t;

struct LodMeshEntry {
  LodMeshEntry *next;
  Mesh *mesh;
  float dMin;
  float dMax;
};

struct LodMesh {
  uint32_t _refCount;
  LodMeshEntry *head;
};

using lua_Number = double;

using lua_CFunction = int32_t(*)(lua_State*);

struct lua_Debug {
  int32_t event;
  const char *name;
  const char *namewhat;
  const char *what;
  const char *source;
  int32_t currentline;
  int32_t nups;
  int32_t linedefined;
  int32_t lastlinedefined;
  char short_src[60];
  int32_t i_ci;
};

using lua_Hook = void(*)(lua_State*, lua_Debug*);

using LuaFn = int32_t(*)(Lua*);

struct Quat {
  float x;
  float y;
  float z;
  float w;
};

struct MemStack {
  uint32_t size;
  uint32_t capacity;
  void *data;
};

struct Cell {
  float value;
  Vec3 normal;
};

struct SDF {
  IVec3 size;
  Cell *data;
};

struct Tex3D {
  uint32_t _refCount;
  uint32_t handle;
  IVec3 size;
  TexFormat format;
};

using Metric = int32_t;

struct MidiDevice {
  int32_t cursor;
  IVec2 buffer[512];
};

using MouseButton = int32_t;

struct Octree {
  Box3 box_0;
  Octree *child[8];
  Node *elems;
};

using PixelFormat = int32_t;

struct Polygon {
  Vec<Vec3> vertices;
};

struct RNG {
  uint64_t seed;
  uint64_t state[2];
};

struct TexCube {
  uint32_t _refCount;
  uint32_t handle;
  int32_t size;
  TexFormat format;
};

using ResourceType = int32_t;

using ShaderVarType = int32_t;

struct ShaderVar {
  ShaderVarType type_0;
  const char *name;
  int32_t index;
};

struct Shader {
  uint32_t _refCount;
  const char *name;
  uint32_t vs;
  uint32_t fs;
  uint32_t program;
  uint32_t texIndex;
  Vec<ShaderVar> vars;
};

struct Tex1D {
  uint32_t _refCount;
  uint32_t handle;
  int32_t size;
  TexFormat format;
};

union C2RustUnnamed {
  float asFloat;
  Vec2 asFloat2;
  Vec3 asFloat3;
  Vec4 asFloat4;
  int32_t asInt;
  Matrix *asMatrix;
  Tex1D *asTex1D;
  Tex2D *asTex2D;
  Tex3D *asTex3D;
  TexCube *asTexCube;
};

struct Elem {
  uint32_t type_0;
  int32_t index;
  C2RustUnnamed data;
};

struct ShaderState {
  uint32_t _refCount;
  Shader *shader;
  Vec<Elem> elems;
};

using Signal = int32_t;

using SignalHandler = void(*)(Signal);

struct StrMap {
  uint32_t capacity;
  uint32_t size;
  Node *data;
};

struct StrMapIter {
  StrMap *map;
  Node *node;
  uint32_t slot;
};

using TexFilter = int32_t;

using TexWrapMode = int32_t;

struct Thread {
  SDL_Thread *handle;
};

using ThreadFn = int32_t(*)(void*);

using ThreadPoolFn = int32_t(*)(int32_t, int32_t, void*);

struct ThreadData {
  SDL_Thread *handle;
  ThreadPoolFn fn_0;
  int32_t index;
  int32_t threads;
  void *data;
};

struct ThreadPool {
  int32_t threads;
  ThreadData *thread;
};

struct Time {
  int32_t second;
  int32_t minute;
  int32_t hour;
  int32_t dayOfWeek;
  int32_t dayOfMonth;
  int32_t dayOfYear;
  int32_t month;
  int32_t year;
};

struct Timer {
  uint64_t value;
};

using WindowMode = uint32_t;

struct Window {
  SDL_Window *handle;
  SDL_GLContext context;
  WindowMode mode;
};

using WindowPos = int32_t;

struct Delay {
  BSPNodeRef nodeRef;
  int32_t depth;
};

using PolygonFlag = uint8_t;

constexpr static const BlendMode BlendMode_Additive = 0;

constexpr static const BlendMode BlendMode_Alpha = 1;

constexpr static const BlendMode BlendMode_Disabled = 2;

constexpr static const BlendMode BlendMode_PreMultAlpha = 3;

constexpr static const CullFace CullFace_None = 0;

constexpr static const CullFace CullFace_Back = 1;

constexpr static const CullFace CullFace_Front = 2;

extern "C" {

extern const BSPNodeRel BSPNodeRel_Parent;

extern const BSPNodeRel BSPNodeRel_Back;

extern const BSPNodeRel BSPNodeRel_Front;

extern Vec<Delay> nodeStack;

extern const PolygonFlag PolygonFlag_None;

extern const PolygonFlag PolygonFlag_InvalidFaceSplit;

extern const PolygonFlag PolygonFlag_InvalidDecompose;

extern const PolygonFlag PolygonFlag_InvalidEdgeSplit;

extern const Button Button_Null;

extern const Button Button_First;

extern const Button Button_Keyboard_First;

extern const Button Button_Keyboard_A;

extern const Button Button_Keyboard_B;

extern const Button Button_Keyboard_C;

extern const Button Button_Keyboard_D;

extern const Button Button_Keyboard_E;

extern const Button Button_Keyboard_F;

extern const Button Button_Keyboard_G;

extern const Button Button_Keyboard_H;

extern const Button Button_Keyboard_I;

extern const Button Button_Keyboard_J;

extern const Button Button_Keyboard_K;

extern const Button Button_Keyboard_L;

extern const Button Button_Keyboard_M;

extern const Button Button_Keyboard_N;

extern const Button Button_Keyboard_O;

extern const Button Button_Keyboard_P;

extern const Button Button_Keyboard_Q;

extern const Button Button_Keyboard_R;

extern const Button Button_Keyboard_S;

extern const Button Button_Keyboard_T;

extern const Button Button_Keyboard_U;

extern const Button Button_Keyboard_V;

extern const Button Button_Keyboard_W;

extern const Button Button_Keyboard_X;

extern const Button Button_Keyboard_Y;

extern const Button Button_Keyboard_Z;

extern const Button Button_Keyboard_N0;

extern const Button Button_Keyboard_N1;

extern const Button Button_Keyboard_N2;

extern const Button Button_Keyboard_N3;

extern const Button Button_Keyboard_N4;

extern const Button Button_Keyboard_N5;

extern const Button Button_Keyboard_N6;

extern const Button Button_Keyboard_N7;

extern const Button Button_Keyboard_N8;

extern const Button Button_Keyboard_N9;

extern const Button Button_Keyboard_F1;

extern const Button Button_Keyboard_F2;

extern const Button Button_Keyboard_F3;

extern const Button Button_Keyboard_F4;

extern const Button Button_Keyboard_F5;

extern const Button Button_Keyboard_F6;

extern const Button Button_Keyboard_F7;

extern const Button Button_Keyboard_F8;

extern const Button Button_Keyboard_F9;

extern const Button Button_Keyboard_F10;

extern const Button Button_Keyboard_F11;

extern const Button Button_Keyboard_F12;

extern const Button Button_Keyboard_F13;

extern const Button Button_Keyboard_F14;

extern const Button Button_Keyboard_F15;

extern const Button Button_Keyboard_F16;

extern const Button Button_Keyboard_F17;

extern const Button Button_Keyboard_F18;

extern const Button Button_Keyboard_F19;

extern const Button Button_Keyboard_F20;

extern const Button Button_Keyboard_F21;

extern const Button Button_Keyboard_F22;

extern const Button Button_Keyboard_F23;

extern const Button Button_Keyboard_F24;

extern const Button Button_Keyboard_KP0;

extern const Button Button_Keyboard_KP1;

extern const Button Button_Keyboard_KP2;

extern const Button Button_Keyboard_KP3;

extern const Button Button_Keyboard_KP4;

extern const Button Button_Keyboard_KP5;

extern const Button Button_Keyboard_KP6;

extern const Button Button_Keyboard_KP7;

extern const Button Button_Keyboard_KP8;

extern const Button Button_Keyboard_KP9;

extern const Button Button_Keyboard_KPNumLock;

extern const Button Button_Keyboard_KPDivide;

extern const Button Button_Keyboard_KPMultiply;

extern const Button Button_Keyboard_KPSubtract;

extern const Button Button_Keyboard_KPAdd;

extern const Button Button_Keyboard_KPEnter;

extern const Button Button_Keyboard_KPDecimal;

extern const Button Button_Keyboard_Backspace;

extern const Button Button_Keyboard_Escape;

extern const Button Button_Keyboard_Return;

extern const Button Button_Keyboard_Space;

extern const Button Button_Keyboard_Tab;

extern const Button Button_Keyboard_Backtick;

extern const Button Button_Keyboard_CapsLock;

extern const Button Button_Keyboard_Minus;

extern const Button Button_Keyboard_Equals;

extern const Button Button_Keyboard_LBracket;

extern const Button Button_Keyboard_RBracket;

extern const Button Button_Keyboard_Backslash;

extern const Button Button_Keyboard_Semicolon;

extern const Button Button_Keyboard_Apostrophe;

extern const Button Button_Keyboard_Comma;

extern const Button Button_Keyboard_Period;

extern const Button Button_Keyboard_Slash;

extern const Button Button_Keyboard_PrintScreen;

extern const Button Button_Keyboard_ScrollLock;

extern const Button Button_Keyboard_Pause;

extern const Button Button_Keyboard_Insert;

extern const Button Button_Keyboard_Delete;

extern const Button Button_Keyboard_Home;

extern const Button Button_Keyboard_End;

extern const Button Button_Keyboard_PageUp;

extern const Button Button_Keyboard_PageDown;

extern const Button Button_Keyboard_Right;

extern const Button Button_Keyboard_Left;

extern const Button Button_Keyboard_Down;

extern const Button Button_Keyboard_Up;

extern const Button Button_Keyboard_LCtrl;

extern const Button Button_Keyboard_LShift;

extern const Button Button_Keyboard_LAlt;

extern const Button Button_Keyboard_LMeta;

extern const Button Button_Keyboard_RCtrl;

extern const Button Button_Keyboard_RShift;

extern const Button Button_Keyboard_RAlt;

extern const Button Button_Keyboard_RMeta;

extern const Button Button_Keyboard_Last;

extern const Button Button_Mouse_First;

extern const Button Button_Mouse_Left;

extern const Button Button_Mouse_Middle;

extern const Button Button_Mouse_Right;

extern const Button Button_Mouse_X1;

extern const Button Button_Mouse_X2;

extern const Button Button_Mouse_X;

extern const Button Button_Mouse_Y;

extern const Button Button_Mouse_ScrollX;

extern const Button Button_Mouse_ScrollY;

extern const Button Button_Mouse_Last;

extern const Button Button_Gamepad_First;

extern const Button Button_Gamepad_Button_First;

extern const Button Button_Gamepad_A;

extern const Button Button_Gamepad_B;

extern const Button Button_Gamepad_X;

extern const Button Button_Gamepad_Y;

extern const Button Button_Gamepad_Back;

extern const Button Button_Gamepad_Guide;

extern const Button Button_Gamepad_Start;

extern const Button Button_Gamepad_LStick;

extern const Button Button_Gamepad_RStick;

extern const Button Button_Gamepad_LBumper;

extern const Button Button_Gamepad_RBumper;

extern const Button Button_Gamepad_Up;

extern const Button Button_Gamepad_Down;

extern const Button Button_Gamepad_Left;

extern const Button Button_Gamepad_Right;

extern const Button Button_Gamepad_Button_Last;

extern const Button Button_Gamepad_Axis_First;

extern const Button Button_Gamepad_LTrigger;

extern const Button Button_Gamepad_RTrigger;

extern const Button Button_Gamepad_LStickX;

extern const Button Button_Gamepad_LStickY;

extern const Button Button_Gamepad_RStickX;

extern const Button Button_Gamepad_RStickY;

extern const Button Button_Gamepad_Axis_Last;

extern const Button Button_Gamepad_Last;

extern const Button Button_System_First;

extern const Button Button_System_Exit;

extern const Button Button_System_Win_Enter;

extern const Button Button_System_Win_Leave;

extern const Button Button_System_Last;

extern const Button Button_Last;

extern const CubeFace CubeFace_PX;

extern const CubeFace CubeFace_NX;

extern const CubeFace CubeFace_PY;

extern const CubeFace CubeFace_NY;

extern const CubeFace CubeFace_PZ;

extern const CubeFace CubeFace_NZ;

extern const DataFormat DataFormat_U8;

extern const DataFormat DataFormat_I8;

extern const DataFormat DataFormat_U16;

extern const DataFormat DataFormat_I16;

extern const DataFormat DataFormat_U32;

extern const DataFormat DataFormat_I32;

extern const DataFormat DataFormat_Float;

extern const DeviceType DeviceType_Null;

extern const DeviceType DeviceType_Mouse;

extern const DeviceType DeviceType_Keyboard;

extern const DeviceType DeviceType_Gamepad;

extern const uint32_t subsystems;

extern uint64_t nextID;

extern const GamepadAxis GamepadAxis_BEGIN;

extern const GamepadAxis GamepadAxis_LeftX;

extern const GamepadAxis GamepadAxis_LeftY;

extern const GamepadAxis GamepadAxis_RightX;

extern const GamepadAxis GamepadAxis_RightY;

extern const GamepadAxis GamepadAxis_LTrigger;

extern const GamepadAxis GamepadAxis_RTrigger;

extern const GamepadAxis GamepadAxis_END;

extern const GamepadButton GamepadButton_BEGIN;

extern const GamepadButton GamepadButton_A;

extern const GamepadButton GamepadButton_B;

extern const GamepadButton GamepadButton_X;

extern const GamepadButton GamepadButton_Y;

extern const GamepadButton GamepadButton_Back;

extern const GamepadButton GamepadButton_Guide;

extern const GamepadButton GamepadButton_Start;

extern const GamepadButton GamepadButton_LStick;

extern const GamepadButton GamepadButton_RStick;

extern const GamepadButton GamepadButton_LBumper;

extern const GamepadButton GamepadButton_RBumper;

extern const GamepadButton GamepadButton_Up;

extern const GamepadButton GamepadButton_Down;

extern const GamepadButton GamepadButton_Left;

extern const GamepadButton GamepadButton_Right;

extern const GamepadButton GamepadButton_END;

extern const HatDir HatDir_Centered;

extern const HatDir HatDir_Up;

extern const HatDir HatDir_Right;

extern const HatDir HatDir_Down;

extern const HatDir HatDir_Left;

extern const HatDir HatDir_RightUp;

extern const HatDir HatDir_RightDown;

extern const HatDir HatDir_LeftUp;

extern const HatDir HatDir_LeftDown;

extern const float InputBindings_DefaultMaxValue;

extern const float InputBindings_DefaultMinValue;

extern const float InputBindings_DefaultDeadzone;

extern const float InputBindings_DefaultExponent;

extern const float InputBindings_DefaultReleaseThreshold;

extern const float InputBindings_DefaultPressThreshold;

extern const Key Key_A;

extern const Key Key_B;

extern const Key Key_C;

extern const Key Key_D;

extern const Key Key_E;

extern const Key Key_F;

extern const Key Key_G;

extern const Key Key_H;

extern const Key Key_I;

extern const Key Key_J;

extern const Key Key_K;

extern const Key Key_L;

extern const Key Key_M;

extern const Key Key_N;

extern const Key Key_O;

extern const Key Key_P;

extern const Key Key_Q;

extern const Key Key_R;

extern const Key Key_S;

extern const Key Key_T;

extern const Key Key_U;

extern const Key Key_V;

extern const Key Key_W;

extern const Key Key_X;

extern const Key Key_Y;

extern const Key Key_Z;

extern const Key Key_N0;

extern const Key Key_N1;

extern const Key Key_N2;

extern const Key Key_N3;

extern const Key Key_N4;

extern const Key Key_N5;

extern const Key Key_N6;

extern const Key Key_N7;

extern const Key Key_N8;

extern const Key Key_N9;

extern const Key Key_F1;

extern const Key Key_F2;

extern const Key Key_F3;

extern const Key Key_F4;

extern const Key Key_F5;

extern const Key Key_F6;

extern const Key Key_F7;

extern const Key Key_F8;

extern const Key Key_F9;

extern const Key Key_F10;

extern const Key Key_F11;

extern const Key Key_F12;

extern const Key Key_F13;

extern const Key Key_F14;

extern const Key Key_F15;

extern const Key Key_F16;

extern const Key Key_F17;

extern const Key Key_F18;

extern const Key Key_F19;

extern const Key Key_F20;

extern const Key Key_F21;

extern const Key Key_F22;

extern const Key Key_F23;

extern const Key Key_F24;

extern const Key Key_KP0;

extern const Key Key_KP1;

extern const Key Key_KP2;

extern const Key Key_KP3;

extern const Key Key_KP4;

extern const Key Key_KP5;

extern const Key Key_KP6;

extern const Key Key_KP7;

extern const Key Key_KP8;

extern const Key Key_KP9;

extern const Key Key_KPNumLock;

extern const Key Key_KPDivide;

extern const Key Key_KPMultiply;

extern const Key Key_KPSubtract;

extern const Key Key_KPAdd;

extern const Key Key_KPEnter;

extern const Key Key_KPDecimal;

extern const Key Key_Backspace;

extern const Key Key_Escape;

extern const Key Key_Return;

extern const Key Key_Space;

extern const Key Key_Tab;

extern const Key Key_Backtick;

extern const Key Key_CapsLock;

extern const Key Key_Minus;

extern const Key Key_Equals;

extern const Key Key_LBracket;

extern const Key Key_RBracket;

extern const Key Key_Backslash;

extern const Key Key_Semicolon;

extern const Key Key_Apostrophe;

extern const Key Key_Comma;

extern const Key Key_Period;

extern const Key Key_Slash;

extern const Key Key_PrintScreen;

extern const Key Key_ScrollLock;

extern const Key Key_Pause;

extern const Key Key_Insert;

extern const Key Key_Home;

extern const Key Key_PageUp;

extern const Key Key_PageDown;

extern const Key Key_Delete;

extern const Key Key_Right;

extern const Key Key_Left;

extern const Key Key_Down;

extern const Key Key_Up;

extern const Key Key_LCtrl;

extern const Key Key_LShift;

extern const Key Key_LAlt;

extern const Key Key_LMeta;

extern const Key Key_RCtrl;

extern const Key Key_RShift;

extern const Key Key_RAlt;

extern const Key Key_RMeta;

extern const Modifier Modifier_Null;

extern const Modifier Modifier_Alt;

extern const Modifier Modifier_Ctrl;

extern const Modifier Modifier_Shift;

extern int32_t lastX;

extern int32_t lastY;

extern uint32_t lastState;

extern const MouseButton MouseButton_Left;

extern const MouseButton MouseButton_Middle;

extern const MouseButton MouseButton_Right;

extern const MouseButton MouseButton_X1;

extern const MouseButton MouseButton_X2;

extern const PixelFormat PixelFormat_Red;

extern const PixelFormat PixelFormat_RG;

extern const PixelFormat PixelFormat_RGB;

extern const PixelFormat PixelFormat_BGR;

extern const PixelFormat PixelFormat_RGBA;

extern const PixelFormat PixelFormat_BGRA;

extern const PixelFormat PixelFormat_Depth_Component;

extern const ResourceType ResourceType_Font;

extern const ResourceType ResourceType_Mesh;

extern const ResourceType ResourceType_Other;

extern const ResourceType ResourceType_Script;

extern const ResourceType ResourceType_Shader;

extern const ResourceType ResourceType_Sound;

extern const ResourceType ResourceType_Tex1D;

extern const ResourceType ResourceType_Tex2D;

extern const ResourceType ResourceType_Tex3D;

extern const ResourceType ResourceType_TexCube;

extern const uint32_t ElemType_Float;

extern const uint32_t ElemType_Float2;

extern const uint32_t ElemType_Float3;

extern const uint32_t ElemType_Float4;

extern const uint32_t ElemType_Int;

extern const uint32_t ElemType_Matrix;

extern const uint32_t ElemType_Tex1D;

extern const uint32_t ElemType_Tex2D;

extern const uint32_t ElemType_Tex3D;

extern const uint32_t ElemType_TexCube;

extern const Signal Signal_Ill;

extern const Signal Signal_Fpe;

extern const Signal Signal_Segv;

extern const Signal Signal_Term;

extern const Signal Signal_Abrt;

extern const Signal Signal_Int;

extern const State State_Null;

extern const State State_Changed;

extern const State State_Pressed;

extern const State State_Down;

extern const State State_Released;

extern const TexFilter TexFilter_Point;

extern const TexFilter TexFilter_PointMipPoint;

extern const TexFilter TexFilter_PointMipLinear;

extern const TexFilter TexFilter_Linear;

extern const TexFilter TexFilter_LinearMipPoint;

extern const TexFilter TexFilter_LinearMipLinear;

extern const TexFormat TexFormat_R8;

extern const TexFormat TexFormat_R16;

extern const TexFormat TexFormat_R16F;

extern const TexFormat TexFormat_R32F;

extern const TexFormat TexFormat_RG8;

extern const TexFormat TexFormat_RG16;

extern const TexFormat TexFormat_RG16F;

extern const TexFormat TexFormat_RG32F;

extern const TexFormat TexFormat_RGB8;

extern const TexFormat TexFormat_RGBA8;

extern const TexFormat TexFormat_RGBA16;

extern const TexFormat TexFormat_RGBA16F;

extern const TexFormat TexFormat_RGBA32F;

extern const TexFormat TexFormat_Depth16;

extern const TexFormat TexFormat_Depth24;

extern const TexFormat TexFormat_Depth32F;

extern const TexWrapMode TexWrapMode_Clamp;

extern const TexWrapMode TexWrapMode_MirrorClamp;

extern const TexWrapMode TexWrapMode_MirrorRepeat;

extern const TexWrapMode TexWrapMode_Repeat;

extern const WindowMode WindowMode_AlwaysOnTop;

extern const WindowMode WindowMode_Borderless;

extern const WindowMode WindowMode_Fullscreen;

extern const WindowMode WindowMode_Hidden;

extern const WindowMode WindowMode_Maximized;

extern const WindowMode WindowMode_Minimized;

extern const WindowMode WindowMode_Resizable;

extern const WindowMode WindowMode_Shown;

extern const WindowPos WindowPos_Centered;

extern const WindowPos WindowPos_Default;

void Audio_Init();

void Audio_Free();

void Audio_AttachListenerPos(const Vec3 *pos, const Vec3 *vel, const Vec3 *fwd, const Vec3 *up);

void Audio_Set3DSettings(float doppler, float scale, float rolloff);

void Audio_SetListenerPos(const Vec3 *pos, const Vec3 *vel, const Vec3 *fwd, const Vec3 *up);

void Audio_Update();

int32_t Audio_GetLoadedCount();

int32_t Audio_GetPlayingCount();

int32_t Audio_GetTotalCount();

void *Audio_GetHandle();

SoundDesc *Audio_AllocSoundDesc(const char *name);

void Audio_DeallocSoundDesc(SoundDesc *desc);

Sound *Audio_AllocSound();

void Audio_DeallocSound(Sound *sound);

void Audio_SoundStateChanged(Sound *sound);

bool BSP_IntersectRay(BSP *this_, const Ray *rayPtr, float *tHit);

bool BSP_IntersectLineSegment(BSP *this_, const LineSegment *lineSegment, Vec3 *pHit);

bool BSP_IntersectSphere(BSP *this_, const Sphere *sphere, Vec3 *pHit);

BSP *BSP_Create(Mesh *mesh);

void BSP_Free(BSP *this_);

BSPNodeRef BSPDebug_GetNode(BSP *this_, BSPNodeRef nodeRef, BSPNodeRel relationship);

void BSPDebug_DrawNode(BSP *this_, BSPNodeRef nodeRef);

void BSPDebug_DrawNodeSplit(BSP *this_, BSPNodeRef nodeRef);

void BSPDebug_DrawLineSegment(BSP *bsp, LineSegment *lineSegment);

void BSPDebug_DrawSphere(BSP *this_, Sphere *sphere);

void BSPDebug_PrintRayProfilingData(BSP *_this, double _totalTime);

void BSPDebug_PrintSphereProfilingData(BSP *_this, double _totalTime);

bool BSPDebug_GetIntersectSphereTriangles(BSP *this_,
                                          Sphere *sphere,
                                          IntersectSphereProfiling *sphereProf);

BSPNodeRef BSPDebug_GetLeaf(BSP *this_, int32_t leafIndex);

uint32_t Bit_And32(uint32_t x, uint32_t y);

uint32_t Bit_Or32(uint32_t x, uint32_t y);

uint32_t Bit_Xor32(uint32_t x, uint32_t y);

bool Bit_Has32(uint32_t x, uint32_t y);

uint64_t Bit_And64(uint64_t x, uint64_t y);

uint64_t Bit_Or64(uint64_t x, uint64_t y);

uint64_t Bit_Xor64(uint64_t x, uint64_t y);

bool Bit_Has64(uint64_t x, uint64_t y);

void BlendMode_Pop();

void BlendMode_Push(BlendMode blendMode);

void BlendMode_PushAdditive();

void BlendMode_PushAlpha();

void BlendMode_PushDisabled();

void BlendMode_PushPreMultAlpha();

BoxMesh *BoxMesh_Create();

void BoxMesh_Free(BoxMesh *this_);

void BoxMesh_Add(BoxMesh *this_, const Vec3 *p, const Vec3 *s, const Vec3 *r, const Vec3 *b);

Mesh *BoxMesh_GetMesh(BoxMesh *this_, int32_t res);

BoxTree *BoxTree_Create();

void BoxTree_Free(BoxTree *this_);

BoxTree *BoxTree_FromMesh(Mesh *mesh);

void BoxTree_Add(BoxTree *this_, Box3 box3, void *data);

int32_t BoxTree_GetMemory(BoxTree *this_);

bool BoxTree_IntersectRay(BoxTree *this_, Matrix *matrix, const Vec3 *ro, const Vec3 *rd);

void BoxTree_Draw(BoxTree *this_, int32_t maxDepth);

DeviceType Button_ToDeviceType(Button button);

const char *Button_ToString(Button button);

bool Button_IsAutoRelease(Button button);

Button Button_FromSDLScancode(SDL_Scancode scancode);

SDL_Scancode Button_ToSDLScancode(Button button);

Button Button_FromSDLMouseButton(uint8_t mouseButton);

uint8_t Button_ToSDLMouseButton(Button button);

Button Button_FromSDLControllerAxis(SDL_GameControllerAxis controllerAxis);

SDL_GameControllerAxis Button_ToSDLControllerAxis(Button button);

Button Button_FromSDLControllerButton(SDL_GameControllerButton controllerButton);

SDL_GameControllerButton Button_ToSDLControllerButton(Button button);

Bytes *Bytes_Create(uint32_t size);

Bytes *Bytes_FromData(const void *data, uint32_t len);

Bytes *Bytes_Load(const char *path);

void Bytes_Free(Bytes *this_);

void *Bytes_GetData(Bytes *this_);

uint32_t Bytes_GetSize(Bytes *this_);

Bytes *Bytes_Compress(Bytes *bytes);

Bytes *Bytes_Decompress(Bytes *bytes);

uint32_t Bytes_GetCursor(Bytes *this_);

void Bytes_Rewind(Bytes *this_);

void Bytes_SetCursor(Bytes *this_, uint32_t cursor);

void Bytes_Read(Bytes *this_, void *data, uint32_t len);

void Bytes_Write(Bytes *this_, const void *data, uint32_t len);

void Bytes_WriteStr(Bytes *this_, const char *data);

uint64_t Bytes_ReadU64(Bytes *this_);

int8_t Bytes_ReadI8(Bytes *this_);

void Bytes_WriteI8(Bytes *this_, int8_t value);

void Bytes_WriteI16(Bytes *this_, int16_t value);

uint8_t Bytes_ReadU8(Bytes *this_);

void Bytes_WriteI32(Bytes *this_, int32_t value);

void Bytes_WriteI64(Bytes *this_, int64_t value);

void Bytes_WriteF32(Bytes *this_, float value);

void Bytes_WriteU16(Bytes *this_, uint16_t value);

void Bytes_WriteU32(Bytes *this_, uint32_t value);

void Bytes_WriteU8(Bytes *this_, uint8_t value);

float Bytes_ReadF32(Bytes *this_);

uint16_t Bytes_ReadU16(Bytes *this_);

uint32_t Bytes_ReadU32(Bytes *this_);

int64_t Bytes_ReadI64(Bytes *this_);

double Bytes_ReadF64(Bytes *this_);

void Bytes_WriteU64(Bytes *this_, uint64_t value);

int16_t Bytes_ReadI16(Bytes *this_);

int32_t Bytes_ReadI32(Bytes *this_);

void Bytes_WriteF64(Bytes *this_, double value);

void Bytes_Print(const Bytes *this_);

void Bytes_Save(const Bytes *this_, const char *path);

void ClipRect_Activate(ClipRect *this_);

void ClipRect_Push(float x, float y, float sx, float sy);

void ClipRect_PushCombined(float x, float y, float sx, float sy);

void ClipRect_PushDisabled();

void ClipRect_PushTransform(float tx, float ty, float sx, float sy);

void ClipRect_Pop();

void ClipRect_PopTransform();

CubeFace CubeFace_Get(int32_t index);

void CullFace_Pop();

void CullFace_Push(CullFace cullFace);

void CullFace_PushNone();

void CullFace_PushBack();

void CullFace_PushFront();

int32_t DataFormat_GetSize(DataFormat this_);

void DepthTest_Pop();

void DepthTest_Push(bool depthTest);

void DepthTest_PushDisabled();

void DepthTest_PushEnabled();

bool Device_Equal(Device *a, Device *b);

const char *Device_ToString(Device *this_);

DeviceType DeviceType_FromButton(Button button);

const char *DeviceType_ToString(DeviceType deviceType);

Directory *Directory_Open(const char *path);

void Directory_Close(Directory *this_);

const char *Directory_GetNext(Directory *this_);

bool Directory_Change(const char *cwd);

bool Directory_Create(const char *path);

const char *Directory_GetCurrent();

bool Directory_Remove(const char *path);

void Draw_PushAlpha(float a);

void Draw_PopAlpha();

void Draw_Axes(const Vec3 *pos,
               const Vec3 *x,
               const Vec3 *y,
               const Vec3 *z,
               float scale,
               float _alpha);

void Draw_Border(float s, float x, float y, float w, float h);

void Draw_Box3(const Box3 *this_);

void Draw_Clear(float r, float g, float b, float a);

void Draw_ClearDepth(float d);

void Draw_Color(float r, float g, float b, float a);

void Draw_Flush();

void Draw_Line(float x1, float y1, float x2, float y2);

void Draw_Line3(const Vec3 *p1, const Vec3 *p2);

void Draw_LineWidth(float width);

void Draw_Plane(const Vec3 *p, const Vec3 *n, float scale);

void Draw_Point(float x, float y);

void Draw_Point3(float x, float y, float z);

void Draw_PointSize(float size);

void Draw_Poly(const Vec2 *points, int32_t count);

void Draw_Poly3(const Vec3 *points, int32_t count);

void Draw_Quad(const Vec2 *p1, const Vec2 *p2, const Vec2 *p3, const Vec2 *p4);

void Draw_Quad3(const Vec3 *p1, const Vec3 *p2, const Vec3 *p3, const Vec3 *p4);

void Draw_Rect(float x1, float y1, float xs, float ys);

void Draw_SmoothLines(bool enabled);

void Draw_SmoothPoints(bool enabled);

void Draw_Sphere(const Vec3 *p, float r);

void Draw_Tri(const Vec2 *v1, const Vec2 *v2, const Vec2 *v3);

void Draw_Tri3(const Vec3 *v1, const Vec3 *v2, const Vec3 *v3);

void Engine_Init(int32_t glVersionMajor, int32_t glVersionMinor);

void Engine_Free();

void Engine_Abort();

int32_t Engine_GetBits();

double Engine_GetTime();

const char *Engine_GetVersion();

bool Engine_IsInitialized();

void Engine_Terminate();

void Engine_Update();

void Error_Print(Error e);

bool File_Exists(const char *path);

bool File_IsDir(const char *path);

Option<Box<File>> File_Create(const char *path);

Option<Box<File>> File_Open(const char *path);

void File_Close(Option<Box<File>>);

Bytes *File_ReadBytes(const char *path);

const char *File_ReadCstr(const char *path);

int64_t File_Size(const char *path);

void File_Read(File *this_, void *data, uint32_t len);

void File_Write(File *this_, const void *data, uint32_t len);

void File_WriteStr(File *this_, const char *data);

uint8_t File_ReadU8(File *this_);

uint16_t File_ReadU16(File *this_);

uint32_t File_ReadU32(File *this_);

uint64_t File_ReadU64(File *this_);

int8_t File_ReadI8(File *this_);

int16_t File_ReadI16(File *this_);

int32_t File_ReadI32(File *this_);

int64_t File_ReadI64(File *this_);

float File_ReadF32(File *this_);

double File_ReadF64(File *this_);

void File_WriteU8(File *this_, uint8_t value);

void File_WriteU16(File *this_, uint16_t value);

void File_WriteU32(File *this_, uint32_t value);

void File_WriteU64(File *this_, uint64_t value);

void File_WriteI8(File *this_, int8_t value);

void File_WriteI16(File *this_, int16_t value);

void File_WriteI32(File *this_, int32_t value);

void File_WriteI64(File *this_, int64_t value);

void File_WriteF32(File *this_, float value);

void File_WriteF64(File *this_, double value);

Font *Font_Load(const char *name, int32_t size);

void Font_Acquire(Font *this_);

void Font_Free(Font *this_);

void Font_Draw(Font *this_, const char *text, float x, float y, float r, float g, float b, float a);

void Font_DrawShaded(Font *this_, const char *text, float x, float y);

int32_t Font_GetLineHeight(Font *this_);

void Font_GetSize(Font *this_, IVec4 *out, const char *text);

void Font_GetSize2(Font *this_, IVec2 *out, const char *text);

void GLMatrix_Clear();

void GLMatrix_Load(Matrix *matrix);

void GLMatrix_LookAt(const DVec3 *eye, const DVec3 *at, const DVec3 *up);

void GLMatrix_ModeP();

void GLMatrix_ModeWV();

void GLMatrix_Mult(Matrix *matrix);

void GLMatrix_Perspective(double fovy, double aspect, double z0, double z1);

void GLMatrix_Pop();

void GLMatrix_Push();

void GLMatrix_PushClear();

Option<Box<Matrix>> GLMatrix_Get();

void GLMatrix_RotateX(double angle);

void GLMatrix_RotateY(double angle);

void GLMatrix_RotateZ(double angle);

void GLMatrix_Scale(double x, double y, double z);

void GLMatrix_Translate(double x, double y, double z);

uint64_t GUID_Create();

bool GUID_Exists(uint64_t id);

void GUID_Reset();

bool Gamepad_CanOpen(int32_t index);

Gamepad *Gamepad_Open(int32_t index);

void Gamepad_Close(Gamepad *this_);

int32_t Gamepad_AddMappings(const char *file);

double Gamepad_GetAxis(Gamepad *this_, GamepadAxis axis);

double Gamepad_GetAxisDelta(Gamepad *this_, GamepadAxis axis);

bool Gamepad_GetButton(Gamepad *this_, GamepadButton button);

double Gamepad_GetButtonPressed(Gamepad *this_, GamepadButton button);

double Gamepad_GetButtonReleased(Gamepad *this_, GamepadButton button);

double Gamepad_GetIdleTime(Gamepad *this_);

int32_t Gamepad_GetID(Gamepad *this_);

const char *Gamepad_GetName(Gamepad *this_);

bool Gamepad_IsConnected(Gamepad *this_);

void Gamepad_SetDeadzone(Gamepad *this_, GamepadAxis axis, double deadzone);

void Gamepad_Update();

uint32_t Hash_FNV32(const void *buf, int32_t len);

uint64_t Hash_FNV64(const void *buf, int32_t len);

uint32_t Hash_FNVStr32(const char *s);

uint64_t Hash_FNVStr64(const char *s);

uint64_t Hash_FNV64_Init();

uint64_t Hash_FNV64_Incremental(uint64_t this_, const void *buf, int32_t len);

uint32_t Hash_Murmur3(const void *key, int32_t len);

uint64_t Hash_XX64(const void *buf, int32_t len, uint64_t seed);

HashGrid *HashGrid_Create(float cellSize, uint32_t cellCount);

void HashGrid_Free(HashGrid *this_);

HashGridElem *HashGrid_Add(HashGrid *this_, void *object, const Box3 *box_0);

void HashGrid_Clear(HashGrid *this_);

void HashGrid_Remove(HashGrid *this_, HashGridElem *elem);

void HashGrid_Update(HashGrid *this_, HashGridElem *elem, const Box3 *box_0);

void **HashGrid_GetResults(HashGrid *this_);

int32_t HashGrid_QueryBox(HashGrid *this_, const Box3 *box_0);

int32_t HashGrid_QueryPoint(HashGrid *this_, const Vec3 *p);

HashMap *HashMap_Create(uint32_t keySize, uint32_t capacity);

void HashMap_Free(HashMap *this_);

void HashMap_Foreach(HashMap *this_, ValueForeach fn_0, void *userData);

void *HashMap_Get(HashMap *this_, const void *key);

void *HashMap_GetRaw(HashMap *this_, uint64_t hash);

void HashMap_Resize(HashMap *this_, uint32_t capacity);

void HashMap_Set(HashMap *this_, const void *key, void *value);

void HashMap_SetRaw(HashMap *this_, uint64_t hash, void *value);

void HmGui_Begin(float sx, float sy);

void HmGui_End();

void HmGui_Draw();

void HmGui_BeginGroupX();

void HmGui_BeginGroupY();

void HmGui_BeginGroupStack();

void HmGui_EndGroup();

void HmGui_BeginScroll(float maxSize);

void HmGui_EndScroll();

void HmGui_BeginWindow(const char *_title);

void HmGui_EndWindow();

bool HmGui_Button(const char *label);

bool HmGui_Checkbox(const char *label, bool value);

float HmGui_Slider(float _lower, float _upper, float _value);

void HmGui_Image(Tex2D *image);

void HmGui_Rect(float sx, float sy, float r, float g, float b, float a);

void HmGui_Text(const char *text);

void HmGui_TextColored(const char *text, float r, float g, float b, float a);

void HmGui_TextEx(Font *font, const char *text, float r, float g, float b, float a);

void HmGui_SetAlign(float ax, float ay);

void HmGui_SetPadding(float px, float py);

void HmGui_SetPaddingEx(float left, float top, float right, float bottom);

void HmGui_SetPaddingLeft(float padding);

void HmGui_SetPaddingTop(float padding);

void HmGui_SetPaddingRight(float padding);

void HmGui_SetPaddingBottom(float padding);

void HmGui_SetSpacing(float spacing);

void HmGui_SetStretch(float x, float y);

bool HmGui_GroupHasFocus(int32_t ty);

void HmGui_PushStyle();

void HmGui_PushFont(Font *font);

void HmGui_PushTextColor(float r, float g, float b, float a);

void HmGui_PopStyle(int32_t depth);

void ImGui_Begin(float sx, float sy);

void ImGui_End();

void ImGui_Draw();

void ImGui_AlignCursor(float sx, float sy, float alignX, float alignY);

float ImGui_GetCursorX();

float ImGui_GetCursorY();

void ImGui_PushCursor();

void ImGui_PopCursor();

void ImGui_SetCursor(float cx, float cy);

void ImGui_SetCursorX(float x);

void ImGui_SetCursorY(float y);

void ImGui_Indent();

void ImGui_Undent();

void ImGui_BeginGroup(float sx, float sy, bool horizontal);

void ImGui_BeginGroupX(float sy);

void ImGui_BeginGroupY(float sx);

void ImGui_EndGroup();

void ImGui_BeginPanel(float sx, float sy);

void ImGui_EndPanel();

void ImGui_BeginWindow(const char *_title, float sx, float sy);

void ImGui_EndWindow();

void ImGui_BeginScrollFrame(float sx, float sy);

void ImGui_EndScrollFrame();

void ImGui_SetNextWidth(float sx);

void ImGui_SetNextHeight(float sy);

void ImGui_PushStyle();

void ImGui_PushStyleFont(Font *font);

void ImGui_PushStylePadding(float px, float py);

void ImGui_PushStyleSpacing(float x, float y);

void ImGui_PushStyleTextColor(float r, float g, float b, float a);

void ImGui_PopStyle();

void ImGui_SetFont(Font *font);

void ImGui_SetSpacing(float sx, float sy);

bool ImGui_Button(const char *label);

bool ImGui_ButtonEx(const char *label, float sx, float sy);

bool ImGui_Checkbox(bool value);

void ImGui_Divider();

bool ImGui_Selectable(const char *label);

void ImGui_Tex2D(Tex2D *tex);

void ImGui_Text(const char *text);

void ImGui_TextColored(const char *text, float r, float g, float b, float a);

void ImGui_TextEx(Font *font, const char *text, float r, float g, float b, float a);

void Input_Init();

void Input_Free();

void Input_Update();

void Input_LoadGamepadDatabase(const char *name);

bool Input_GetPressed(Button button);

bool Input_GetDown(Button button);

bool Input_GetReleased(Button button);

float Input_GetValue(Button button);

float Input_GetIdleTime();

void Input_GetActiveDevice(Device *device);

DeviceType Input_GetActiveDeviceType();

uint32_t Input_GetActiveDeviceID();

float Input_GetActiveDeviceIdleTime();

bool Input_GetDevicePressed(Device *device, Button button);

bool Input_GetDeviceDown(Device *device, Button button);

bool Input_GetDeviceReleased(Device *device, Button button);

float Input_GetDeviceValue(Device *device, Button button);

float Input_GetDeviceIdleTime(Device *device);

void Input_GetMouseDelta(IVec2 *delta);

float Input_GetMouseIdleTime();

void Input_GetMousePosition(IVec2 *position);

void Input_GetMouseScroll(IVec2 *scroll);

void Input_SetMousePosition(IVec2 *position);

void Input_SetMouseVisible(bool visible);

void Input_SetMouseVisibleAuto();

void Input_SetMouseScroll(IVec2 *scroll);

float Input_GetKeyboardIdleTime();

bool Input_GetKeyboardMod(Modifier modifier);

bool Input_GetKeyboardAlt();

bool Input_GetKeyboardCtrl();

bool Input_GetKeyboardShift();

float Input_GetGamepadIdleTime(uint32_t id);

bool Input_GetGamepadPressed(uint32_t id, Button button);

bool Input_GetGamepadDown(uint32_t id, Button button);

bool Input_GetGamepadReleased(uint32_t id, Button button);

float Input_GetGamepadValue(uint32_t id, Button button);

int32_t Input_GetEventCount();

bool Input_GetNextEvent(InputEvent *event);

void InputBindings_Init();

void InputBindings_Free();

void InputBindings_UpdateBinding(InputBinding *binding);

void InputBindings_Update();

bool InputBinding_GetPressed(InputBinding *binding);

bool InputBinding_GetDown(InputBinding *binding);

bool InputBinding_GetReleased(InputBinding *binding);

float InputBinding_GetValue(InputBinding *binding);

Vec2 InputBinding_GetVecValue(InputBinding *binding);

float InputBinding_GetXValue(InputBinding *binding);

float InputBinding_GetYValue(InputBinding *binding);

bool InputBinding_GetXPosPressed(InputBinding *binding);

bool InputBinding_GetXPosDown(InputBinding *binding);

bool InputBinding_GetXPosReleased(InputBinding *binding);

bool InputBinding_GetXNegPressed(InputBinding *binding);

bool InputBinding_GetXNegDown(InputBinding *binding);

bool InputBinding_GetXNegReleased(InputBinding *binding);

bool InputBinding_GetYPosPressed(InputBinding *binding);

bool InputBinding_GetYPosDown(InputBinding *binding);

bool InputBinding_GetYPosReleased(InputBinding *binding);

bool InputBinding_GetYNegPressed(InputBinding *binding);

bool InputBinding_GetYNegDown(InputBinding *binding);

bool InputBinding_GetYNegReleased(InputBinding *binding);

InputBinding *InputBinding_SetDeadzone(InputBinding *binding, float deadzone);

InputBinding *InputBinding_SetExponent(InputBinding *binding, float exponent);

InputBinding *InputBinding_SetInvertX(InputBinding *binding, bool invert);

InputBinding *InputBinding_SetInvertY(InputBinding *binding, bool invert);

InputBinding *InputBinding_SetRange(InputBinding *binding, float min, float max);

InputBinding *InputBinding_SetThresholds(InputBinding *binding, float press, float release);

const char *InputEvent_ToString(InputEvent *ie);

bool Intersect_PointBox(Matrix *src, Matrix *dst);

bool Intersect_PointTriangle_Barycentric(const Vec3 *p, const Triangle *tri);

bool Intersect_RayPlane(const Ray *ray, const Plane *plane, Vec3 *pHit);

bool Intersect_RayTriangle_Barycentric(const Ray *ray,
                                       const Triangle *tri,
                                       float tEpsilon,
                                       float *tHit);

bool Intersect_RayTriangle_Moller1(const Ray *ray, const Triangle *tri, float *tHit);

bool Intersect_RayTriangle_Moller2(const Ray *ray, const Triangle *tri, float *tHit);

bool Intersect_LineSegmentPlane(const LineSegment *lineSegment, const Plane *plane, Vec3 *pHit);

bool Intersect_RectRect(const Vec4 *a, const Vec4 *b);

bool Intersect_RectRectFast(const Vec4 *a, const Vec4 *b);

bool Intersect_SphereTriangle(const Sphere *sphere, const Triangle *triangle, Vec3 *pHit);

int32_t Joystick_GetCount();

Joystick *Joystick_Open(int32_t index);

void Joystick_Close(Joystick *this_);

const char *Joystick_GetGUID(Joystick *this_);

const char *Joystick_GetGUIDByIndex(int32_t index);

const char *Joystick_GetName(Joystick *this_);

const char *Joystick_GetNameByIndex(int32_t index);

int32_t Joystick_GetAxisCount(Joystick *this_);

int32_t Joystick_GetBallCount(Joystick *this_);

int32_t Joystick_GetButtonCount(Joystick *this_);

int32_t Joystick_GetHatCount(Joystick *this_);

double Joystick_GetIdleTime(Joystick *this_);

double Joystick_GetAxis(Joystick *this_, int32_t index);

bool Joystick_GetAxisAlive(Joystick *this_, int32_t index);

double Joystick_GetAxisDelta(Joystick *this_, int32_t index);

HatDir Joystick_GetHat(Joystick *this_, int32_t index);

bool Joystick_ButtonDown(Joystick *this_, int32_t index);

bool Joystick_ButtonPressed(Joystick *this_, int32_t index);

bool Joystick_ButtonReleased(Joystick *this_, int32_t index);

void Joystick_Update();

KDTree *KDTree_FromMesh(Mesh *mesh);

void KDTree_Free(KDTree *this_);

int32_t KDTree_GetMemory(KDTree *this_);

bool KDTree_IntersectRay(KDTree *_this, Matrix *_m, const Vec3 *_a, const Vec3 *_b);

void KDTree_Draw(KDTree *this_, int32_t maxDepth);

void Keyboard_Init();

void Keyboard_Free();

void Keyboard_UpdatePre();

void Keyboard_UpdatePost();

bool Keyboard_Down(Key key);

bool Keyboard_Pressed(Key key);

bool Keyboard_Released(Key key);

double Keyboard_GetIdleTime();

bool KeyMod_Alt();

bool KeyMod_Ctrl();

bool KeyMod_Shift();

void LineSegment_ToRay(const LineSegment *this_, Ray *out);

void LineSegment_FromRay(const Ray *ray, LineSegment *out);

const char *LineSegment_ToString(LineSegment *this_);

LodMesh *LodMesh_Create();

void LodMesh_Acquire(LodMesh *this_);

void LodMesh_Free(LodMesh *this_);

void LodMesh_Add(LodMesh *this_, Mesh *mesh, float dMin, float dMax);

void LodMesh_Draw(LodMesh *this_, float d2);

Mesh *LodMesh_Get(LodMesh *this_, float d2);

extern void lua_close(lua_State *L);

extern lua_State *lua_newthread(lua_State *L);

extern int32_t lua_gettop(lua_State *L);

extern void lua_settop(lua_State *L, int32_t idx);

extern void lua_xmove(lua_State *from, lua_State *to, int32_t n);

extern int32_t lua_type(lua_State *L, int32_t idx);

extern const char *lua_typename(lua_State *L, int32_t tp);

extern int32_t lua_toboolean(lua_State *L, int32_t idx);

extern const char *lua_tolstring(lua_State *L, int32_t idx, uintptr_t *len);

extern void *lua_touserdata(lua_State *L, int32_t idx);

extern const void *lua_topointer(lua_State *L, int32_t idx);

extern void lua_pushnumber(lua_State *L, lua_Number n);

extern void lua_pushstring(lua_State *L, const char *s);

extern void lua_pushcclosure(lua_State *L, lua_CFunction fn_0, int32_t n);

extern void lua_pushboolean(lua_State *L, int32_t b);

extern void lua_pushlightuserdata(lua_State *L, void *p);

extern int32_t lua_pushthread(lua_State *L);

extern void lua_getfield(lua_State *L, int32_t idx, const char *k);

extern void lua_rawgeti(lua_State *L, int32_t idx, int32_t n);

extern void lua_setfield(lua_State *L, int32_t idx, const char *k);

extern void lua_call(lua_State *L, int32_t nargs, int32_t nresults);

extern int32_t lua_pcall(lua_State *L, int32_t nargs, int32_t nresults, int32_t errfunc);

extern int32_t lua_gc(lua_State *L, int32_t what, int32_t data);

extern int32_t lua_error(lua_State *L);

extern int32_t lua_getstack(lua_State *L, int32_t level, lua_Debug *ar);

extern int32_t lua_getinfo(lua_State *L, const char *what, lua_Debug *ar);

extern const char *lua_getlocal(lua_State *L, const lua_Debug *ar, int32_t n);

extern const char *lua_getupvalue(lua_State *L, int32_t funcindex, int32_t n);

extern int32_t lua_sethook(lua_State *L, lua_Hook func, int32_t mask, int32_t count);

extern int32_t luaL_loadstring(lua_State *L, const char *s);

extern int32_t luaL_callmeta(lua_State *L, int32_t obj, const char *e);

extern void luaL_where(lua_State *L, int32_t lvl);

extern int32_t luaL_ref(lua_State *L, int32_t t);

extern void luaL_unref(lua_State *L, int32_t t, int32_t ref_0);

extern int32_t luaL_loadfile(lua_State *L, const char *filename);

extern void luaL_openlibs(lua_State *L);

extern lua_State *luaL_newstate();

Lua *Lua_Create();

Lua *Lua_CreateThread(Lua *this_);

void Lua_Free(Lua *this_);

Lua *Lua_GetActive();

void Lua_DoFile(Lua *this_, const char *name);

void Lua_DoString(Lua *this_, const char *code);

void Lua_LoadFile(Lua *this_, const char *name);

void Lua_LoadString(Lua *this_, const char *code);

void Lua_Call(Lua *this_, int32_t args, int32_t rets, int32_t errorHandler);

void Lua_PushGlobal(Lua *this_, const char *name);

void Lua_PushNumber(Lua *this_, double value);

void Lua_PushPtr(Lua *this_, void *value);

void Lua_PushStr(Lua *this_, const char *value);

void Lua_PushThread(Lua *this_, Lua *thread);

void Lua_SetBool(Lua *this_, const char *name, bool value);

void Lua_SetFn(Lua *this_, const char *name, LuaFn fn_0);

void Lua_SetNumber(Lua *this_, const char *name, double value);

void Lua_SetPtr(Lua *this_, const char *name, void *value);

void Lua_SetStr(Lua *this_, const char *name, const char *value);

void Lua_TransferStack(Lua *src, Lua *dst, int32_t count);

LuaRef Lua_GetRef(Lua *this_);

void Lua_ReleaseRef(Lua *this_, LuaRef ref_0);

void Lua_PushRef(Lua *this_, LuaRef ref_0);

void Lua_GCFull(Lua *this_);

void Lua_GCSetActive(Lua *this_, bool active);

void Lua_GCStep(Lua *this_);

int32_t Lua_GetMemory(Lua *this_);

void Lua_Backtrace();

extern int32_t lua_gettop(lua_State *L);

extern void lua_settop(lua_State *L, int32_t idx);

extern lua_Number lua_tonumber(lua_State *L, int32_t idx);

extern void lua_getfield(lua_State *L, int32_t idx, const char *k);

extern void luaL_unref(lua_State *L, int32_t t, int32_t ref_0);

extern void Lua_PushRef(Lua*, LuaRef);

extern void Lua_ReleaseRef(Lua*, LuaRef);

extern LuaRef Lua_GetRef(Lua*);

extern void Lua_Call(Lua*, int32_t args, int32_t rets, int32_t errorHandler);

extern void Lua_PushNumber(Lua*, double);

extern void Lua_SetFn(Lua*, const char *name, LuaFn);

extern TimeStamp TimeStamp_Get();

extern double TimeStamp_GetDifference(TimeStamp start, TimeStamp end);

extern TimeStamp TimeStamp_GetRelative(TimeStamp start, double seconds);

void LuaScheduler_Init(Lua *_L);

void LuaScheduler_Register(Lua *L);

double Math_Bezier3(double x, double y1, double y2, double y3);

double Math_Bezier4(double x, double y1, double y2, double y3, double y4);

double Math_Clamp(double x, double a, double b);

double Math_Clamp01(double x);

double Math_ClampSafe(double x, double a, double b);

double Math_ClampUnit(double x);

double Math_ExpMap(double x, double p);

double Math_ExpMapSigned(double x, double p);

double Math_ExpMap1(double x);

double Math_ExpMap1Signed(double x);

double Math_ExpMap2(double x);

double Math_ExpMap2Signed(double x);

double Math_PowSigned(double x, double p);

double Math_Round(double x);

double Math_Sign(double x);

Box<Matrix> Matrix_Clone(const Matrix *this_);

void Matrix_Free(Option<Box<Matrix>>);

bool Matrix_Equal(const Matrix *a, const Matrix *b);

bool Matrix_ApproximatelyEqual(const Matrix *a, const Matrix *b);

Box<Matrix> Matrix_Inverse(const Matrix *this_);

Box<Matrix> Matrix_InverseTranspose(const Matrix *this_);

Box<Matrix> Matrix_Sum(const Matrix *a, const Matrix *b);

Box<Matrix> Matrix_Transpose(const Matrix *this_);

void Matrix_IInverse(Matrix *this_);

void Matrix_IScale(Matrix *this_, float scale);

void Matrix_ITranspose(Matrix *this_);

Box<Matrix> Matrix_Identity();

Box<Matrix> Matrix_LookAt(const Vec3 *pos, const Vec3 *at, const Vec3 *up);

Box<Matrix> Matrix_LookUp(const Vec3 *pos, const Vec3 *look, const Vec3 *up);

Box<Matrix> Matrix_Perspective(float degreesFovy, float aspect, float N, float F);

Box<Matrix> Matrix_Product(const Matrix *a, const Matrix *b);

Box<Matrix> Matrix_RotationX(float rads);

Box<Matrix> Matrix_RotationY(float rads);

Box<Matrix> Matrix_RotationZ(float rads);

Box<Matrix> Matrix_Scaling(float sx, float sy, float sz);

Box<Matrix> Matrix_SRT(float sx,
                       float sy,
                       float sz,
                       float ry,
                       float rp,
                       float rr,
                       float tx,
                       float ty,
                       float tz);

Box<Matrix> Matrix_Translation(float tx, float ty, float tz);

Box<Matrix> Matrix_YawPitchRoll(float yaw, float pitch, float roll);

void Matrix_MulBox(const Matrix *this_, Box3 *out, const Box3 *in_0);

void Matrix_MulDir(const Matrix *this_, Vec3 *out, float x, float y, float z);

void Matrix_MulPoint(const Matrix *this_, Vec3 *out, float x, float y, float z);

void Matrix_MulVec(const Matrix *this_, Vec4 *out, float x, float y, float z, float w);

void Matrix_GetForward(const Matrix *this_, Vec3 *out);

void Matrix_GetRight(const Matrix *this_, Vec3 *out);

void Matrix_GetUp(const Matrix *this_, Vec3 *out);

void Matrix_GetPos(const Matrix *this_, Vec3 *out);

void Matrix_GetRow(const Matrix *this_, Vec4 *out, int32_t row);

Box<Matrix> Matrix_FromBasis(const Vec3 *x, const Vec3 *y, const Vec3 *z);

Box<Matrix> Matrix_FromPosRot(const Vec3 *pos, const Quat *rot);

Box<Matrix> Matrix_FromPosRotScale(const Vec3 *pos, const Quat *rot, float scale);

Box<Matrix> Matrix_FromPosBasis(const Vec3 *pos, const Vec3 *x, const Vec3 *y, const Vec3 *z);

Box<Matrix> Matrix_FromQuat(const Quat *q);

void Matrix_ToQuat(const Matrix *this_, Quat *q);

void Matrix_Print(const Matrix *this_);

const char *Matrix_ToString(const Matrix *this_);

MemPool *MemPool_Create(uint32_t cellSize, uint32_t blockSize);

MemPool *MemPool_CreateAuto(uint32_t elemSize);

void MemPool_Free(MemPool *this_);

void *MemPool_Alloc(MemPool *this_);

void MemPool_Clear(MemPool *this_);

void MemPool_Dealloc(MemPool *this_, void *ptr);

uint32_t MemPool_GetCapacity(MemPool *this_);

uint32_t MemPool_GetSize(MemPool *this_);

MemStack *MemStack_Create(uint32_t capacity);

void MemStack_Free(MemStack *this_);

void *MemStack_Alloc(MemStack *this_, uint32_t size);

void MemStack_Clear(MemStack *this_);

void MemStack_Dealloc(MemStack *this_, uint32_t size);

bool MemStack_CanAlloc(MemStack *this_, uint32_t size);

uint32_t MemStack_GetSize(MemStack *this_);

uint32_t MemStack_GetCapacity(MemStack *this_);

uint32_t MemStack_GetRemaining(MemStack *this_);

void *Memory_Alloc(uintptr_t size);

void *Memory_Calloc(uintptr_t n, uintptr_t size);

void Memory_Free(void *ptr);

void Memory_MemCopy(void *dst, const void *src, uintptr_t size);

void Memory_MemMove(void *dst, const void *src, uintptr_t size);

void *Memory_Realloc(void *ptr, uintptr_t newSize);

Mesh *Mesh_Create();

Mesh *Mesh_Clone(Mesh *other);

Mesh *Mesh_Load(const char *name);

void Mesh_Acquire(Mesh *this_);

void Mesh_Free(Mesh *this_);

Bytes *Mesh_ToBytes(Mesh *mesh);

Mesh *Mesh_FromBytes(Bytes *buf);

Mesh *Mesh_FromSDF(SDF *sdf);

void Mesh_AddIndex(Mesh *this_, int32_t newIndex);

void Mesh_AddMesh(Mesh *this_, Mesh *other);

void Mesh_AddQuad(Mesh *this_, int32_t i1, int32_t i2, int32_t i3, int32_t i4);

void Mesh_AddTri(Mesh *this_, int32_t i1, int32_t i2, int32_t i3);

void Mesh_AddVertex(Mesh *this_,
                    float px,
                    float py,
                    float pz,
                    float nx,
                    float ny,
                    float nz,
                    float u,
                    float v);

void Mesh_AddVertexRaw(Mesh *this_, const Vertex *vertex);

void Mesh_DrawBind(Mesh *this_);

void Mesh_DrawBound(Mesh *this_);

void Mesh_DrawUnbind(Mesh *_this);

void Mesh_Draw(Mesh *this_);

void Mesh_DrawNormals(Mesh *this_, float scale);

void Mesh_GetBound(Mesh *this_, Box3 *out);

void Mesh_GetCenter(Mesh *this_, Vec3 *out);

int32_t Mesh_GetIndexCount(Mesh *this_);

int32_t *Mesh_GetIndexData(Mesh *this_);

float Mesh_GetRadius(Mesh *this_);

uint64_t Mesh_GetVersion(Mesh *this_);

void Mesh_IncVersion(Mesh *this_);

Error Mesh_Validate(Mesh *this_);

Vertex *Mesh_GetVertex(Mesh *this_, int32_t index);

int32_t Mesh_GetVertexCount(Mesh *this_);

Vertex *Mesh_GetVertexData(Mesh *this_);

void Mesh_ReserveIndexData(Mesh *this_, int32_t capacity);

void Mesh_ReserveVertexData(Mesh *this_, int32_t capacity);

Mesh *Mesh_Center(Mesh *this_);

Mesh *Mesh_Invert(Mesh *this_);

Mesh *Mesh_RotateX(Mesh *this_, float rads);

Mesh *Mesh_RotateY(Mesh *this_, float rads);

Mesh *Mesh_RotateZ(Mesh *this_, float rads);

Mesh *Mesh_RotateYPR(Mesh *this_, float yaw, float pitch, float roll);

Mesh *Mesh_Scale(Mesh *this_, float x, float y, float z);

Mesh *Mesh_ScaleUniform(Mesh *this_, float s);

Mesh *Mesh_Translate(Mesh *this_, float x, float y, float z);

Mesh *Mesh_Transform(Mesh *this_, const Matrix *matrix);

void Mesh_ComputeNormals(Mesh *this_);

void Mesh_SplitNormals(Mesh *this_, float minDot);

void Mesh_ComputeAO(Mesh *this_, float radius);

void Mesh_ComputeOcclusion(Mesh *this_, Tex3D *sdf, float radius);

Mesh *Mesh_FromObj(const char *bytes);

Mesh *Mesh_Box(int32_t res);

Mesh *Mesh_BoxSphere(int32_t res);

Mesh *Mesh_Plane(Vec3 origin, Vec3 du, Vec3 dv, int32_t resU, int32_t resV);

int32_t Metric_Get(Metric this_);

const char *Metric_GetName(Metric this_);

void Metric_AddDraw(int32_t polys, int32_t tris, int32_t verts);

void Metric_AddDrawImm(int32_t polys, int32_t tris, int32_t verts);

void Metric_Inc(Metric this_);

void Metric_Mod(Metric this_, int32_t delta);

void Metric_Reset();

int32_t MidiDevice_GetCount();

MidiDevice *MidiDevice_Open(int32_t _index);

void MidiDevice_Close(MidiDevice *_this);

const char *MidiDevice_GetNameByIndex(int32_t _index);

bool MidiDevice_HasMessage(MidiDevice *this_);

IVec2 MidiDevice_PopMessage(MidiDevice *this_);

const char *Modifier_ToString(Modifier modifier);

void Mouse_Init();

void Mouse_Free();

void Mouse_SetScroll(int32_t amount);

void Mouse_Update();

void Mouse_GetDelta(IVec2 *out);

double Mouse_GetIdleTime();

void Mouse_GetPosition(IVec2 *out);

void Mouse_GetPositionGlobal(IVec2 *out);

int32_t Mouse_GetScroll();

void Mouse_SetPosition(int32_t x, int32_t y);

void Mouse_SetVisible(bool visible);

bool Mouse_Down(MouseButton button);

bool Mouse_Pressed(MouseButton button);

bool Mouse_Released(MouseButton button);

const char *OS_GetClipboard();

int32_t OS_GetCPUCount();

const char *OS_GetVideoDriver();

void OS_SetClipboard(const char *text);

Octree *Octree_Create(Box3 box_0);

void Octree_Free(Octree *this_);

Octree *Octree_FromMesh(Mesh *mesh);

double Octree_GetAvgLoad(Octree *this_);

int32_t Octree_GetMaxLoad(Octree *this_);

int32_t Octree_GetMemory(Octree *this_);

bool Octree_IntersectRay(Octree *this_, Matrix *matrix, const Vec3 *ro, const Vec3 *rd);

void Octree_Add(Octree *this_, Box3 box_0, uint32_t id);

void Octree_Draw(Octree *this_);

void OpenGL_Init();

void OpenGL_CheckError(const char *file, int32_t line);

extern Physics *_cppPhysics_Create();

extern void _cppPhysics_Free(Physics*);

extern void _cppPhysics_AddRigidBody(Physics*, RigidBody*);

extern void _cppPhysics_RemoveRigidBody(Physics*, RigidBody*);

extern void _cppPhysics_AddTrigger(Physics*, Trigger*);

extern void _cppPhysics_RemoveTrigger(Physics*, Trigger*);

extern bool _cppPhysics_GetNextCollision(Physics*, Collision*);

extern void _cppPhysics_Update(Physics*, float dt);

extern void _cppPhysics_RayCast(Physics*, Ray*, RayCastResult*);

extern void _cppPhysics_SphereCast(Physics*, Sphere*, ShapeCastResult*);

extern void _cppPhysics_BoxCast(Physics*,
                                Vec3 *pos,
                                Quat *rot,
                                Vec3 *halfExtents,
                                ShapeCastResult*);

extern bool _cppPhysics_SphereOverlap(Physics*, Sphere*);

extern bool _cppPhysics_BoxOverlap(Physics*, Vec3 *pos, Quat *rot, Vec3 *halfExtents);

extern void _cppPhysics_PrintProfiling(Physics*);

extern void _cppPhysics_DrawBoundingBoxesLocal(Physics*);

extern void _cppPhysics_DrawBoundingBoxesWorld(Physics*);

extern void _cppPhysics_DrawTriggers(Physics*);

extern void _cppPhysics_DrawWireframes(Physics*);

Physics *Physics_Create();

void Physics_Free(Physics *this_);

void Physics_AddRigidBody(Physics *this_, RigidBody *rb);

void Physics_RemoveRigidBody(Physics *this_, RigidBody *rb);

void Physics_AddTrigger(Physics *this_, Trigger *t);

void Physics_RemoveTrigger(Physics *this_, Trigger *t);

bool Physics_GetNextCollision(Physics *this_, Collision *c);

void Physics_Update(Physics *this_, float dt);

void Physics_RayCast(Physics *this_, Ray *ray, RayCastResult *result);

void Physics_SphereCast(Physics *this_, Sphere *sphere, ShapeCastResult *result);

void Physics_BoxCast(Physics *this_,
                     Vec3 *pos,
                     Quat *rot,
                     Vec3 *halfExtents,
                     ShapeCastResult *result);

bool Physics_SphereOverlap(Physics *this_, Sphere *sphere);

bool Physics_BoxOverlap(Physics *this_, Vec3 *pos, Quat *rot, Vec3 *halfExtents);

void Physics_PrintProfiling(Physics *this_);

void Physics_DrawBoundingBoxesLocal(Physics *this_);

void Physics_DrawBoundingBoxesWorld(Physics *this_);

void Physics_DrawTriggers(Physics *this_);

void Physics_DrawWireframes(Physics *this_);

int32_t PixelFormat_Components(PixelFormat this_);

PointClassification Plane_ClassifyPoint(const Plane *plane, const Vec3 *p);

PolygonClassification Plane_ClassifyPolygon(const Plane *plane, const Polygon *polygon);

Error Plane_Validate(const Plane *plane);

void Plane_FromPolygon(const Polygon *polygon, Plane *plane);

void Plane_FromPolygonFast(const Polygon *polygon, Plane *plane);

void Polygon_ToPlane(const Polygon *polygon, Plane *out);

void Polygon_ToPlaneFast(const Polygon *polygon, Plane *out);

void Polygon_SplitSafe(const Polygon *polygon, Plane splitPlane, Polygon *back, Polygon *front);

void Polygon_Split(Polygon *polygon, Plane splitPlane, Polygon *back, Polygon *front);

void Polygon_GetCentroid(Polygon *polygon, Vec3 *out);

Error Polygon_Validate(Polygon *polygon);

void Profiler_Enable();

void Profiler_Disable();

void Profiler_Begin(const char *name);

void Profiler_End();

void Profiler_SetValue(const char *_name, int32_t _value);

void Profiler_LoopMarker();

void Profiler_Backtrace();

Quat Quat_Create(float x, float y, float z, float w);

void Quat_GetAxisX(const Quat *q, Vec3 *out);

void Quat_GetAxisY(const Quat *q, Vec3 *out);

void Quat_GetAxisZ(const Quat *q, Vec3 *out);

void Quat_GetForward(const Quat *q, Vec3 *out);

void Quat_GetRight(const Quat *q, Vec3 *out);

void Quat_GetUp(const Quat *q, Vec3 *out);

void Quat_Identity(Quat *out);

void Quat_Canonicalize(const Quat *q, Quat *out);

void Quat_ICanonicalize(Quat *q);

float Quat_Dot(const Quat *q, const Quat *p);

bool Quat_Equal(const Quat *q, const Quat *p);

bool Quat_ApproximatelyEqual(const Quat *q, const Quat *p);

void Quat_Inverse(const Quat *q, Quat *out);

void Quat_IInverse(Quat *q);

void Quat_Lerp(const Quat *q, const Quat *p, float t, Quat *out);

void Quat_ILerp(Quat *q, const Quat *p, float t);

void Quat_Mul(const Quat *q, const Quat *p, Quat *out);

void Quat_IMul(Quat *q, const Quat *p);

void Quat_MulV(const Quat *q, const Vec3 *v, Vec3 *out);

void Quat_Normalize(const Quat *q, Quat *out);

void Quat_INormalize(Quat *q);

void Quat_Scale(const Quat *q, float scale, Quat *out);

void Quat_IScale(Quat *q, float scale);

void Quat_Slerp(const Quat *q, const Quat *p, float t, Quat *out);

void Quat_ISlerp(Quat *q, const Quat *p, float t);

const char *Quat_ToString(const Quat *q);

Error Quat_Validate(const Quat *q);

void Quat_FromAxisAngle(const Vec3 *axis, float radians, Quat *out);

void Quat_FromBasis(const Vec3 *x, const Vec3 *y, const Vec3 *z, Quat *out);

void Quat_FromLookUp(const Vec3 *look, const Vec3 *up, Quat *out);

void Quat_FromRotateTo(const Vec3 *from, const Vec3 *to, Quat *out);

Box<RNG> RNG_Create(uint64_t seed);

Box<RNG> RNG_FromStr(const char *s);

Box<RNG> RNG_FromTime();

void RNG_Free(Option<Box<RNG>>);

void RNG_Rewind(RNG *this_);

bool RNG_Chance(RNG *this_, double probability);

int32_t RNG_Get31(RNG *this_);

uint32_t RNG_Get32(RNG *this_);

uint64_t RNG_Get64(RNG *this_);

double RNG_GetAngle(RNG *this_);

int32_t RNG_GetInt(RNG *this_, int32_t lower, int32_t upper);

Box<RNG> RNG_GetRNG(RNG *this_);

double RNG_GetUniform(RNG *this_);

double RNG_GetUniformRange(RNG *this_, double lower, double upper);

double RNG_GetErlang(RNG *this_, int32_t k);

double RNG_GetExp(RNG *this_);

double RNG_GetGaussian(RNG *this_);

void RNG_GetAxis2(RNG *this_, Vec2 *out);

void RNG_GetAxis3(RNG *this_, Vec3 *out);

void RNG_GetDir2(RNG *this_, Vec2 *out);

void RNG_GetDir3(RNG *this_, Vec3 *out);

void RNG_GetDisc(RNG *this_, Vec2 *out);

double RNG_GetSign(RNG *this_);

void RNG_GetSphere(RNG *this_, Vec3 *out);

void RNG_GetVec2(RNG *this_, Vec2 *out, double lower, double upper);

void RNG_GetVec3(RNG *this_, Vec3 *out, double lower, double upper);

void RNG_GetVec4(RNG *this_, Vec4 *out, double lower, double upper);

void RNG_GetQuat(RNG *this_, Quat *out);

void Ray_GetPoint(const Ray *this_, float t, Vec3 *out);

bool Ray_IntersectPlane(const Ray *this_, const Plane *plane, Vec3 *pHit);

bool Ray_IntersectTriangle_Barycentric(const Ray *this_,
                                       const Triangle *tri,
                                       float tEpsilon,
                                       float *tHit);

bool Ray_IntersectTriangle_Moller1(const Ray *this_, const Triangle *tri, float *tHit);

bool Ray_IntersectTriangle_Moller2(const Ray *this_, const Triangle *tri, float *tHit);

void Ray_ToLineSegment(const Ray *this_, LineSegment *lineSegment);

void Ray_FromLineSegment(const LineSegment *lineSegment, Ray *this_);

void RenderState_PushAllDefaults();

void RenderState_PopAll();

void RenderState_PopBlendMode();

void RenderState_PopWireframe();

void RenderState_PushBlendMode(BlendMode value);

void RenderState_PopDepthTest();

void RenderState_PopCullFace();

void RenderState_PopDepthWritable();

void RenderState_PushCullFace(CullFace value);

void RenderState_PushDepthTest(bool value);

void RenderState_PushDepthWritable(bool value);

void RenderState_PushWireframe(bool value);

void RenderTarget_Push(int32_t sx, int32_t sy);

void RenderTarget_Pop();

void RenderTarget_BindTex2D(Tex2D *this_);

void RenderTarget_BindTex2DLevel(Tex2D *tex, int32_t level);

void RenderTarget_BindTex3D(Tex3D *this_, int32_t layer);

void RenderTarget_BindTex3DLevel(Tex3D *tex, int32_t layer, int32_t level);

void RenderTarget_BindTexCube(TexCube *this_, CubeFace face);

void RenderTarget_BindTexCubeLevel(TexCube *tex, CubeFace face, int32_t level);

void RenderTarget_PushTex2D(Tex2D *this_);

void RenderTarget_PushTex2DLevel(Tex2D *this_, int32_t level);

void RenderTarget_PushTex3D(Tex3D *this_, int32_t layer);

void RenderTarget_PushTex3DLevel(Tex3D *this_, int32_t layer, int32_t level);

bool Resource_Exists(ResourceType ty, const char *name);

const char *Resource_GetPath(ResourceType ty, const char *name);

Bytes *Resource_LoadBytes(ResourceType ty, const char *name);

const char *Resource_LoadCstr(ResourceType ty, const char *name);

void Resource_Init();

const char *ResourceType_ToString(ResourceType this_);

extern RigidBody *_cppRigidBody_CreateBox();

extern RigidBody *_cppRigidBody_CreateBoxFromMesh(Mesh *mesh);

extern RigidBody *_cppRigidBody_CreateSphere();

extern RigidBody *_cppRigidBody_CreateSphereFromMesh(Mesh *mesh);

extern RigidBody *_cppRigidBody_CreateHullFromMesh(Mesh *mesh);

extern void _cppRigidBody_Free(RigidBody *this_);

extern void _cppRigidBody_ApplyForce(RigidBody *this_, Vec3 *force);

extern void _cppRigidBody_ApplyTorque(RigidBody *this_, Vec3 *torque);

extern void _cppRigidBody_Attach(RigidBody *this_, RigidBody *other, Vec3 *offset, Quat *rot);

extern void _cppRigidBody_Detach(RigidBody *this_, RigidBody *other);

extern void _cppRigidBody_GetBoundingBox(RigidBody *this_, Box3 *out);

extern void _cppRigidBody_GetBoundingBoxCompound(RigidBody *this_, Box3 *out);

extern void _cppRigidBody_GetBoundingBoxLocal(RigidBody *this_, Box3 *out);

extern void _cppRigidBody_GetBoundingBoxLocalCompound(RigidBody *this_, Box3 *out);

extern float _cppRigidBody_GetBoundingRadius(RigidBody *this_);

extern float _cppRigidBody_GetBoundingRadiusCompound(RigidBody *this_);

extern RigidBody *_cppRigidBody_GetParentBody(RigidBody *this_);

extern float _cppRigidBody_GetSpeed(RigidBody *this_);

extern Matrix *_cppRigidBody_GetToLocalMatrix(RigidBody *this_);

extern Matrix *_cppRigidBody_GetToWorldMatrix(RigidBody *this_);

extern void _cppRigidBody_GetVelocity(RigidBody *this_, Vec3 *out);

extern void _cppRigidBody_GetVelocityA(RigidBody *this_, Vec3 *out);

extern void _cppRigidBody_SetCollidable(RigidBody *this_, bool collidable);

extern void _cppRigidBody_SetCollisionGroup(RigidBody *this_, int32_t group);

extern void _cppRigidBody_SetCollisionMask(RigidBody *this_, int32_t mask);

extern void _cppRigidBody_SetDrag(RigidBody *this_, float linear, float angular);

extern void _cppRigidBody_SetFriction(RigidBody *this_, float friction);

extern void _cppRigidBody_SetKinematic(RigidBody *this_, bool kinematic);

extern void _cppRigidBody_SetRestitution(RigidBody *this_, float restitution);

extern void _cppRigidBody_SetSleepThreshold(RigidBody *this_, float linear, float angular);

extern float _cppRigidBody_GetMass(RigidBody *this_);

extern void _cppRigidBody_SetMass(RigidBody *this_, float mass);

extern void _cppRigidBody_GetPos(RigidBody *this_, Vec3 *out);

extern void _cppRigidBody_GetPosLocal(RigidBody *this_, Vec3 *out);

extern void _cppRigidBody_SetPos(RigidBody *this_, Vec3 *pos);

extern void _cppRigidBody_SetPosLocal(RigidBody *this_, Vec3 *pos);

extern void _cppRigidBody_GetRot(RigidBody *this_, Quat *out);

extern void _cppRigidBody_GetRotLocal(RigidBody *this_, Quat *out);

extern void _cppRigidBody_SetRot(RigidBody *this_, Quat *rot);

extern void _cppRigidBody_SetRotLocal(RigidBody *this_, Quat *rot);

extern float _cppRigidBody_GetScale(RigidBody *this_);

extern void _cppRigidBody_SetScale(RigidBody *this_, float scale);

RigidBody *RigidBody_CreateBox();

RigidBody *RigidBody_CreateBoxFromMesh(Mesh *mesh);

RigidBody *RigidBody_CreateSphere();

RigidBody *RigidBody_CreateSphereFromMesh(Mesh *mesh);

RigidBody *RigidBody_CreateHullFromMesh(Mesh *mesh);

void RigidBody_Free(RigidBody *this_);

void RigidBody_ApplyForce(RigidBody *this_, Vec3 *force);

void RigidBody_ApplyTorque(RigidBody *this_, Vec3 *torque);

void RigidBody_Attach(RigidBody *this_, RigidBody *other, Vec3 *offset, Quat *rot);

void RigidBody_Detach(RigidBody *this_, RigidBody *other);

void RigidBody_GetBoundingBox(RigidBody *this_, Box3 *out);

void RigidBody_GetBoundingBoxCompound(RigidBody *this_, Box3 *out);

void RigidBody_GetBoundingBoxLocal(RigidBody *this_, Box3 *out);

void RigidBody_GetBoundingBoxLocalCompound(RigidBody *this_, Box3 *out);

float RigidBody_GetBoundingRadius(RigidBody *this_);

float RigidBody_GetBoundingRadiusCompound(RigidBody *this_);

RigidBody *RigidBody_GetParentBody(RigidBody *this_);

float RigidBody_GetSpeed(RigidBody *this_);

Matrix *RigidBody_GetToLocalMatrix(RigidBody *this_);

Matrix *RigidBody_GetToWorldMatrix(RigidBody *this_);

void RigidBody_GetVelocity(RigidBody *this_, Vec3 *out);

void RigidBody_GetVelocityA(RigidBody *this_, Vec3 *out);

void RigidBody_SetCollidable(RigidBody *this_, bool collidable);

void RigidBody_SetCollisionGroup(RigidBody *this_, int32_t group);

void RigidBody_SetCollisionMask(RigidBody *this_, int32_t mask);

void RigidBody_SetDrag(RigidBody *this_, float linear, float angular);

void RigidBody_SetFriction(RigidBody *this_, float friction);

void RigidBody_SetKinematic(RigidBody *this_, bool kinematic);

void RigidBody_SetRestitution(RigidBody *this_, float restitution);

void RigidBody_SetSleepThreshold(RigidBody *this_, float linear, float angular);

float RigidBody_GetMass(RigidBody *this_);

void RigidBody_SetMass(RigidBody *this_, float mass);

void RigidBody_GetPos(RigidBody *this_, Vec3 *out);

void RigidBody_GetPosLocal(RigidBody *this_, Vec3 *out);

void RigidBody_SetPos(RigidBody *this_, Vec3 *pos);

void RigidBody_SetPosLocal(RigidBody *this_, Vec3 *pos);

void RigidBody_GetRot(RigidBody *this_, Quat *out);

void RigidBody_GetRotLocal(RigidBody *this_, Quat *out);

void RigidBody_SetRot(RigidBody *this_, Quat *rot);

void RigidBody_SetRotLocal(RigidBody *this_, Quat *rot);

float RigidBody_GetScale(RigidBody *this_);

void RigidBody_SetScale(RigidBody *this_, float scale);

SDF *SDF_Create(int32_t sx, int32_t sy, int32_t sz);

SDF *SDF_FromTex3D(Tex3D *tex);

void SDF_Free(SDF *this_);

Mesh *SDF_ToMesh(SDF *this_);

void SDF_Clear(SDF *this_, float value);

void SDF_ComputeNormals(SDF *this_);

void SDF_Set(SDF *this_, int32_t x, int32_t y, int32_t z, float value);

void SDF_SetNormal(SDF *this_, int32_t x, int32_t y, int32_t z, const Vec3 *normal);

Shader *Shader_Create(const char *vs, const char *fs);

Shader *Shader_Load(const char *vName, const char *fName);

void Shader_Acquire(Shader *this_);

void Shader_Free(Shader *this_);

ShaderState *Shader_ToShaderState(Shader *this_);

void Shader_Start(Shader *this_);

void Shader_Stop(Shader *_s);

void Shader_ClearCache();

uint32_t Shader_GetHandle(Shader *this_);

int32_t Shader_GetVariable(Shader *this_, const char *name);

bool Shader_HasVariable(Shader *this_, const char *name);

void Shader_ResetTexIndex();

void Shader_SetFloat(const char *name, float value);

void Shader_ISetFloat(int32_t index, float value);

void Shader_SetFloat2(const char *name, float x, float y);

void Shader_ISetFloat2(int32_t index, float x, float y);

void Shader_SetFloat3(const char *name, float x, float y, float z);

void Shader_ISetFloat3(int32_t index, float x, float y, float z);

void Shader_SetFloat4(const char *name, float x, float y, float z, float w);

void Shader_ISetFloat4(int32_t index, float x, float y, float z, float w);

void Shader_SetInt(const char *name, int32_t value);

void Shader_ISetInt(int32_t index, int32_t value);

void Shader_SetMatrix(const char *name, Matrix *value);

void Shader_SetMatrixT(const char *name, Matrix *value);

void Shader_ISetMatrix(int32_t index, Matrix *value);

void Shader_ISetMatrixT(int32_t index, Matrix *value);

void Shader_SetTex1D(const char *name, Tex1D *value);

void Shader_ISetTex1D(int32_t index, Tex1D *value);

void Shader_SetTex2D(const char *name, Tex2D *value);

void Shader_ISetTex2D(int32_t index, Tex2D *value);

void Shader_SetTex3D(const char *name, Tex3D *value);

void Shader_ISetTex3D(int32_t index, Tex3D *value);

void Shader_SetTexCube(const char *name, TexCube *value);

void Shader_ISetTexCube(int32_t index, TexCube *value);

ShaderState *ShaderState_Create(Shader *shader);

void ShaderState_Acquire(ShaderState *this_);

void ShaderState_Free(ShaderState *this_);

ShaderState *ShaderState_FromShaderLoad(const char *vertName, const char *fragName);

void ShaderState_SetFloat(ShaderState *this_, const char *name, float x);

void ShaderState_SetFloat2(ShaderState *this_, const char *name, float x, float y);

void ShaderState_SetFloat3(ShaderState *this_, const char *name, float x, float y, float z);

void ShaderState_SetFloat4(ShaderState *this_,
                           const char *name,
                           float x,
                           float y,
                           float z,
                           float w);

void ShaderState_SetInt(ShaderState *this_, const char *name, int32_t x);

void ShaderState_SetMatrix(ShaderState *this_, const char *name, Matrix *x);

void ShaderState_SetTex1D(ShaderState *this_, const char *name, Tex1D *x);

void ShaderState_SetTex2D(ShaderState *this_, const char *name, Tex2D *x);

void ShaderState_SetTex3D(ShaderState *this_, const char *name, Tex3D *x);

void ShaderState_SetTexCube(ShaderState *this_, const char *name, TexCube *x);

void ShaderState_Start(ShaderState *this_);

void ShaderState_Stop(ShaderState *this_);

void ShaderVar_Init();

void ShaderVar_Free();

void *ShaderVar_Get(const char *name, ShaderVarType type_0);

void ShaderVar_PushFloat(const char *name, float x);

void ShaderVar_PushFloat2(const char *name, float x, float y);

void ShaderVar_PushFloat3(const char *name, float x, float y, float z);

void ShaderVar_PushFloat4(const char *name, float x, float y, float z, float w);

void ShaderVar_PushInt(const char *name, int32_t x);

void ShaderVar_PushMatrix(const char *name, Matrix *x);

void ShaderVar_PushTex1D(const char *name, Tex1D *x);

void ShaderVar_PushTex2D(const char *name, Tex2D *x);

void ShaderVar_PushTex3D(const char *name, Tex3D *x);

void ShaderVar_PushTexCube(const char *name, TexCube *x);

void ShaderVar_Pop(const char *name);

ShaderVarType ShaderVarType_FromStr(const char *s);

const char *ShaderVarType_GetGLSLName(ShaderVarType this_);

const char *ShaderVarType_GetName(ShaderVarType this_);

int32_t ShaderVarType_GetSize(ShaderVarType this_);

extern void (*signal(int32_t, void(*)(int32_t)))(int32_t);

void Signal_Init();

void Signal_Free();

void Signal_AddHandler(Signal sig, SignalHandler fn_0);

void Signal_AddHandlerAll(SignalHandler fn_0);

void Signal_RemoveHandler(Signal sig, SignalHandler fn_0);

void Signal_RemoveHandlerAll(SignalHandler fn_0);

const char *Signal_ToString(Signal this_);

void Signal_IgnoreDefault();

Sound *Sound_Load(const char *name, bool isLooped, bool is3D);

Sound *Sound_LoadAsync(const char *name, bool isLooped, bool is3D);

Sound *Sound_Clone(Sound *this_);

void Sound_ToFile(Sound *this_, const char *name);

void Sound_Acquire(Sound *this_);

void Sound_Free(Sound *this_);

void Sound_Play(Sound *this_);

void Sound_Pause(Sound *this_);

void Sound_Rewind(Sound *this_);

bool Sound_Get3D(Sound *this_);

float Sound_GetDuration(Sound *this_);

bool Sound_GetLooped(Sound *this_);

const char *Sound_GetName(Sound *this_);

const char *Sound_GetPath(Sound *this_);

bool Sound_IsFinished(Sound *this_);

bool Sound_IsPlaying(Sound *this_);

bool Sound_IsAudible(Sound *this_);

void Sound_Attach3DPos(Sound *this_, const Vec3 *pos, const Vec3 *vel);

void Sound_Set3DLevel(Sound *this_, float level);

void Sound_Set3DPos(Sound *this_, const Vec3 *pos, const Vec3 *vel);

void Sound_SetFreeOnFinish(Sound *this_, bool freeOnFinish);

void Sound_SetPan(Sound *this_, float pan);

void Sound_SetPitch(Sound *this_, float pitch);

void Sound_SetPlayPos(Sound *this_, float seconds);

void Sound_SetVolume(Sound *this_, float volume);

void Sound_FadeIn(Sound *this_, float seconds);

void Sound_FadeOut(Sound *this_, float seconds);

Sound *Sound_LoadPlay(const char *name, bool isLooped, bool is3D);

Sound *Sound_LoadPlayAttached(const char *name,
                              bool isLooped,
                              bool is3D,
                              const Vec3 *pos,
                              const Vec3 *vel);

void Sound_LoadPlayFree(const char *name, bool isLooped, bool is3D);

void Sound_LoadPlayFreeAttached(const char *name,
                                bool isLooped,
                                bool is3D,
                                const Vec3 *pos,
                                const Vec3 *vel);

Sound *Sound_ClonePlay(Sound *this_);

Sound *Sound_ClonePlayAttached(Sound *this_, const Vec3 *pos, const Vec3 *vel);

void Sound_ClonePlayFree(Sound *this_);

void Sound_ClonePlayFreeAttached(Sound *this_, const Vec3 *pos, const Vec3 *vel);

void Sound_Update(Sound *this_);

bool Sound_IsFreed(Sound *this_);

void SoundDesc_FinishLoad(SoundDesc *this_, const char *func);

SoundDesc *SoundDesc_Load(const char *name, bool immediate, bool isLooped, bool is3D);

void SoundDesc_Acquire(SoundDesc *this_);

void SoundDesc_Free(SoundDesc *this_);

float SoundDesc_GetDuration(SoundDesc *this_);

const char *SoundDesc_GetName(SoundDesc *this_);

const char *SoundDesc_GetPath(SoundDesc *this_);

void SoundDesc_ToFile(SoundDesc *this_, const char *name);

const char *State_ToString(State state);

StrMap *StrMap_Create(uint32_t capacity);

void StrMap_Free(StrMap *this_);

void StrMap_FreeEx(StrMap *this_, void (*freeFn)(const char*, void*));

void *StrMap_Get(StrMap *this_, const char *key);

uint32_t StrMap_GetSize(StrMap *this_);

void StrMap_Remove(StrMap *this_, const char *key);

void StrMap_Set(StrMap *this_, const char *key, void *value);

void StrMap_Dump(StrMap *this_);

StrMapIter *StrMap_Iterate(StrMap *this_);

void StrMapIter_Free(StrMapIter *this_);

void StrMapIter_Advance(StrMapIter *it);

bool StrMapIter_HasMore(StrMapIter *it);

const char *StrMapIter_GetKey(StrMapIter *it);

void *StrMapIter_GetValue(StrMapIter *it);

Tex1D *Tex1D_Create(int32_t size, TexFormat format);

void Tex1D_Acquire(Tex1D *this_);

void Tex1D_Free(Tex1D *this_);

void Tex1D_Draw(Tex1D *this_, float x, float y, float xs, float ys);

void Tex1D_GenMipmap(Tex1D *this_);

TexFormat Tex1D_GetFormat(Tex1D *this_);

void Tex1D_GetData(Tex1D *this_, void *data, PixelFormat pf, DataFormat df);

Bytes *Tex1D_GetDataBytes(Tex1D *this_, PixelFormat pf, DataFormat df);

uint32_t Tex1D_GetHandle(Tex1D *this_);

uint32_t Tex1D_GetSize(Tex1D *this_);

void Tex1D_SetData(Tex1D *this_, const void *data, PixelFormat pf, DataFormat df);

void Tex1D_SetDataBytes(Tex1D *this_, Bytes *data, PixelFormat pf, DataFormat df);

void Tex1D_SetMagFilter(Tex1D *this_, TexFilter filter);

void Tex1D_SetMinFilter(Tex1D *this_, TexFilter filter);

void Tex1D_SetTexel(Tex1D *this_, int32_t x, float r, float g, float b, float a);

void Tex1D_SetWrapMode(Tex1D *this_, TexWrapMode mode);

Tex2D *Tex2D_Create(int32_t sx, int32_t sy, TexFormat format);

Tex2D *Tex2D_ScreenCapture();

void Tex2D_Acquire(Tex2D *this_);

void Tex2D_Free(Tex2D *this_);

void Tex2D_Pop(Tex2D*);

void Tex2D_Push(Tex2D *this_);

void Tex2D_PushLevel(Tex2D *this_, int32_t level);

void Tex2D_Clear(Tex2D *this_, float r, float g, float b, float a);

Tex2D *Tex2D_Clone(Tex2D *this_);

void Tex2D_Draw(Tex2D *this_, float x, float y, float sx, float sy);

void Tex2D_DrawEx(Tex2D *this_,
                  float x0,
                  float y0,
                  float x1,
                  float y1,
                  float u0,
                  float v0,
                  float u1,
                  float v1);

void Tex2D_GenMipmap(Tex2D *this_);

void Tex2D_GetData(Tex2D *this_, void *data, PixelFormat pf, DataFormat df);

Bytes *Tex2D_GetDataBytes(Tex2D *this_, PixelFormat pf, DataFormat df);

TexFormat Tex2D_GetFormat(Tex2D *this_);

uint32_t Tex2D_GetHandle(Tex2D *this_);

void Tex2D_GetSize(Tex2D *this_, IVec2 *out);

void Tex2D_GetSizeLevel(Tex2D *this_, IVec2 *out, int32_t level);

Tex2D *Tex2D_Load(const char *name);

void Tex2D_SetAnisotropy(Tex2D *this_, float factor);

void Tex2D_SetData(Tex2D *this_, const void *data, PixelFormat pf, DataFormat df);

void Tex2D_SetDataBytes(Tex2D *this_, Bytes *data, PixelFormat pf, DataFormat df);

void Tex2D_SetMagFilter(Tex2D *this_, TexFilter filter);

void Tex2D_SetMinFilter(Tex2D *this_, TexFilter filter);

void Tex2D_SetMipRange(Tex2D *this_, int32_t minLevel, int32_t maxLevel);

void Tex2D_SetTexel(Tex2D *this_, int32_t x, int32_t y, float r, float g, float b, float a);

void Tex2D_SetWrapMode(Tex2D *this_, TexWrapMode mode);

void Tex2D_Save(Tex2D *this_, const char *path);

unsigned char *Tex2D_LoadRaw(const char *path, int32_t *sx, int32_t *sy, int32_t *components);

bool Tex2D_Save_Png(const char *path, int32_t sx, int32_t sy, int32_t components, uint8_t *data);

Tex3D *Tex3D_Create(int32_t sx, int32_t sy, int32_t sz, TexFormat format);

void Tex3D_Acquire(Tex3D *this_);

void Tex3D_Free(Tex3D *this_);

void Tex3D_Pop(Tex3D *_this);

void Tex3D_Push(Tex3D *this_, int32_t layer);

void Tex3D_PushLevel(Tex3D *this_, int32_t layer, int32_t level);

void Tex3D_Draw(Tex3D *this_, int32_t layer, float x, float y, float xs, float ys);

void Tex3D_GenMipmap(Tex3D *this_);

void Tex3D_GetData(Tex3D *this_, void *data, PixelFormat pf, DataFormat df);

Bytes *Tex3D_GetDataBytes(Tex3D *this_, PixelFormat pf, DataFormat df);

TexFormat Tex3D_GetFormat(Tex3D *this_);

uint32_t Tex3D_GetHandle(Tex3D *this_);

void Tex3D_GetSize(Tex3D *this_, IVec3 *out);

void Tex3D_GetSizeLevel(Tex3D *this_, IVec3 *out, int32_t level);

void Tex3D_SetData(Tex3D *this_, const void *data, PixelFormat pf, DataFormat df);

void Tex3D_SetDataBytes(Tex3D *this_, Bytes *data, PixelFormat pf, DataFormat df);

void Tex3D_SetMagFilter(Tex3D *this_, TexFilter filter);

void Tex3D_SetMinFilter(Tex3D *this_, TexFilter filter);

void Tex3D_SetWrapMode(Tex3D *this_, TexWrapMode mode);

TexCube *TexCube_Create(int32_t size, TexFormat format);

void TexCube_Acquire(TexCube *this_);

void TexCube_Clear(TexCube *this_, float r, float g, float b, float a);

void TexCube_Free(TexCube *this_);

TexCube *TexCube_Load(const char *path);

void TexCube_GetData(TexCube *this_,
                     void *data,
                     CubeFace face,
                     int32_t level,
                     PixelFormat pf,
                     DataFormat df);

Bytes *TexCube_GetDataBytes(TexCube *this_,
                            CubeFace face,
                            int32_t level,
                            PixelFormat pf,
                            DataFormat df);

TexFormat TexCube_GetFormat(TexCube *this_);

uint32_t TexCube_GetHandle(TexCube *this_);

int32_t TexCube_GetSize(TexCube *this_);

void TexCube_Generate(TexCube *this_, ShaderState *state);

void TexCube_GenMipmap(TexCube *this_);

void TexCube_SetData(TexCube *this_,
                     const void *data,
                     CubeFace face,
                     int32_t level,
                     PixelFormat pf,
                     DataFormat df);

void TexCube_SetDataBytes(TexCube *this_,
                          Bytes *data,
                          CubeFace face,
                          int32_t level,
                          PixelFormat pf,
                          DataFormat df);

void TexCube_SetMagFilter(TexCube *this_, TexFilter filter);

void TexCube_SetMinFilter(TexCube *this_, TexFilter filter);

void TexCube_Save(TexCube *this_, const char *path);

void TexCube_SaveLevel(TexCube *this_, const char *path, int32_t level);

TexCube *TexCube_GenIRMap(TexCube *this_, int32_t sampleCount);

int32_t TexFormat_Components(TexFormat this_);

int32_t TexFormat_GetSize(TexFormat this_);

bool TexFormat_IsColor(TexFormat this_);

bool TexFormat_IsDepth(TexFormat this_);

bool TexFormat_IsValid(TexFormat this_);

Thread *Thread_Create(const char *name, ThreadFn fn_0, void *data);

void Thread_Detach(Thread *this_);

void Thread_Sleep(uint32_t ms);

int32_t Thread_Wait(Thread *this_);

ThreadPool *ThreadPool_Create(int32_t threads);

void ThreadPool_Free(ThreadPool *this_);

void ThreadPool_Launch(ThreadPool *this_, ThreadPoolFn fn_0, void *data);

void ThreadPool_Wait(ThreadPool *this_);

Time Time_GetLocal();

Time Time_GetUTC();

uint32_t Time_GetRaw();

TimeStamp TimeStamp_Get();

double TimeStamp_GetDifference(TimeStamp start, TimeStamp end);

double TimeStamp_GetElapsed(TimeStamp then);

double TimeStamp_GetElapsedMs(TimeStamp then);

TimeStamp TimeStamp_GetFuture(double seconds);

TimeStamp TimeStamp_GetRelative(TimeStamp start, double seconds);

double TimeStamp_ToDouble(TimeStamp this_);

Timer *Timer_Create();

void Timer_Free(Timer *this_);

double Timer_GetAndReset(Timer *this_);

double Timer_GetElapsed(Timer *this_);

void Timer_Reset(Timer *this_);

void Triangle_ToPlane(const Triangle *tri, Plane *plane);

void Triangle_ToPlaneFast(const Triangle *triangle, Plane *plane);

float Triangle_GetArea(const Triangle *tri);

Error Triangle_Validate(const Triangle *tri);

extern Trigger *_cppTrigger_CreateBox(Vec3 *halfExtents);

extern void _cppTrigger_Free(Trigger *this_);

extern void _cppTrigger_Attach(Trigger *this_, RigidBody *rb, Vec3 *offset);

extern void _cppTrigger_Detach(Trigger *this_, RigidBody *rb);

extern void _cppTrigger_GetBoundingBox(Trigger *this_, Box3 *out);

extern int32_t _cppTrigger_GetContentsCount(Trigger *this_);

extern RigidBody *_cppTrigger_GetContents(Trigger *this_, int32_t i);

extern void _cppTrigger_SetCollisionMask(Trigger *this_, int32_t i);

extern void _cppTrigger_SetPos(Trigger *this_, Vec3 *pos);

extern void _cppTrigger_SetPosLocal(Trigger *this_, Vec3 *pos);

Trigger *Trigger_CreateBox(Vec3 *halfExtents);

void Trigger_Free(Trigger *this_);

void Trigger_Attach(Trigger *this_, RigidBody *rb, Vec3 *offset);

void Trigger_Detach(Trigger *this_, RigidBody *rb);

void Trigger_GetBoundingBox(Trigger *this_, Box3 *out);

int32_t Trigger_GetContentsCount(Trigger *this_);

RigidBody *Trigger_GetContents(Trigger *this_, int32_t i);

void Trigger_SetCollisionMask(Trigger *this_, int32_t i);

void Trigger_SetPos(Trigger *this_, Vec3 *pos);

void Trigger_SetPosLocal(Trigger *this_, Vec3 *pos);

void UIRenderer_Begin();

void UIRenderer_End();

void UIRenderer_Draw();

void UIRenderer_BeginLayer(float x, float y, float sx, float sy, bool clip);

void UIRenderer_EndLayer();

void UIRenderer_Image(Tex2D *image, float x, float y, float sx, float sy);

void UIRenderer_Panel(float x,
                      float y,
                      float sx,
                      float sy,
                      float r,
                      float g,
                      float b,
                      float a,
                      float bevel,
                      float innerAlpha);

void UIRenderer_Rect(float x,
                     float y,
                     float sx,
                     float sy,
                     float r,
                     float g,
                     float b,
                     float a,
                     bool outline);

void UIRenderer_Text(Font *font,
                     const char *text,
                     float x,
                     float y,
                     float r,
                     float g,
                     float b,
                     float a);

float Viewport_GetAspect();

void Viewport_GetSize(IVec2 *out);

void Viewport_Push(int32_t x, int32_t y, int32_t sx, int32_t sy, bool isWindow);

void Viewport_Pop();

Box<Window> Window_Create(const char *title,
                          int32_t x,
                          int32_t y,
                          int32_t sx,
                          int32_t sy,
                          WindowMode mode);

void Window_Free(Option<Box<Window>>);

void Window_BeginDraw(const Window *w);

void Window_EndDraw(const Window *w);

void Window_GetSize(const Window *w, IVec2 *out);

void Window_GetPosition(const Window *w, IVec2 *out);

const char *Window_GetTitle(const Window *w);

void Window_SetFullscreen(const Window *w, bool fs);

void Window_SetPosition(const Window *w, WindowPos x, WindowPos y);

void Window_SetSize(const Window *w, int32_t sx, int32_t sy);

void Window_SetTitle(const Window *w, const char *title);

void Window_SetVsync(const Window*, bool vsync);

void Window_ToggleFullscreen(Window *w);

void Window_Hide(const Window *w);

void Window_Show(const Window *w);

} // extern "C"
