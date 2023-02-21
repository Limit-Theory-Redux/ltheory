#![allow(dead_code)]
#![allow(mutable_transmutes)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(unused_assignments)]
#![allow(unused_mut)]
#![allow(unused_imports)]
#![feature(c_variadic)]
#![feature(extern_types)]
#![feature(thread_local)]

extern crate glam;
extern crate libc;
extern crate memoffset;
extern crate printf_compat;
extern crate rodio;
extern crate sdl2_sys;
extern crate stb;

pub mod internal {
    pub mod Memory;
}
pub mod Audio;
pub mod BSP;
pub mod Bit;
pub mod BlendMode;
pub mod BoxMesh;
pub mod BoxTree;
pub mod Button;
pub mod Bytes;
pub mod ClipRect;
pub mod CollisionShape;
pub mod Common;
pub mod CubeFace;
pub mod CullFace;
pub mod DataFormat;
pub mod DepthTest;
pub mod Device;
pub mod DeviceType;
pub mod Directory;
pub mod Draw;
pub mod Engine;
pub mod Error;
pub mod File;
pub mod Font;
pub mod GLMatrix;
pub mod GUID;
pub mod Gamepad;
pub mod GamepadAxis;
pub mod GamepadButton;
pub mod Hash;
pub mod HashGrid;
pub mod HashMap;
pub mod HatDir;
pub mod HmGui;
pub mod ImGui;
pub mod Input;
pub mod InputBindings;
pub mod InputEvent;
pub mod Intersect;
pub mod Joystick;
pub mod KDTree;
pub mod Key;
pub mod Keyboard;
pub mod LineSegment;
pub mod LodMesh;
pub mod Lua;
pub mod LuaScheduler;
pub mod Matrix;
pub mod MemPool;
pub mod MemStack;
pub mod Mesh;
pub mod Mesh_ComputeAO;
pub mod Mesh_FromObj;
pub mod Meshes;
pub mod Metric;
pub mod Midi;
pub mod Modifier;
pub mod Mouse;
pub mod MouseButton;
pub mod OS;
pub mod Octree;
pub mod OpenGL;
pub mod PhxMath;
pub mod PhxMemory;
pub mod PhxSignal;
pub mod PhxTime;
pub mod Physics;
pub mod PixelFormat;
pub mod Plane;
pub mod Polygon;
pub mod Profiler;
pub mod Quat;
pub mod RNG;
pub mod Ray;
pub mod RenderState;
pub mod RenderTarget;
pub mod Resource;
pub mod ResourceType;
pub mod RigidBody;
pub mod SDF;
pub mod Shader;
pub mod ShaderState;
pub mod ShaderVar;
pub mod ShaderVarType;
pub mod Socket;
pub mod Sound;
pub mod SoundDesc;
pub mod State;
pub mod StrBuffer;
pub mod StrMap;
pub mod Tex1D;
pub mod Tex2D;
pub mod Tex2D_Load;
pub mod Tex2D_Save;
pub mod Tex3D;
pub mod TexCube;
pub mod TexCube_GenIRMap;
pub mod TexFilter;
pub mod TexFormat;
pub mod TexWrapMode;
pub mod Thread;
pub mod ThreadPool;
pub mod TimeStamp;
pub mod Timer;
pub mod Triangle;
pub mod Trigger;
pub mod UIRenderer;
pub mod Viewport;
pub mod Window;
pub mod WindowMode;
pub mod WindowPos;
