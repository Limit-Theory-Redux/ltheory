#![allow(dead_code)]
#![allow(mutable_transmutes)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(unused_assignments)]
#![allow(unused_mut)]
#![allow(unused_imports)]
#![allow(unused_unsafe)]
#![feature(c_variadic)]
#![feature(extern_types)]
#![feature(thread_local)]

extern crate chrono;
extern crate core;
extern crate fmod_sys;
extern crate freetype_sys;
extern crate glam;
extern crate image;
extern crate libc;
extern crate memoffset;
extern crate mlua;
extern crate printf_compat;
extern crate rodio;
extern crate sdl2_sys;

pub mod internal;

pub mod audio;
pub mod bit;
pub mod blend_mode;
pub mod box_mesh;
pub mod box_tree;
pub mod bsp;
pub mod button;
pub mod bytes;
pub mod clip_rect;
pub mod collision_shape;
pub mod common;
pub mod cube_face;
pub mod cull_face;
pub mod data_format;
pub mod depth_test;
pub mod device;
pub mod device_type;
pub mod directory;
pub mod draw;
pub mod engine;
pub mod error;
pub mod file;
pub mod font;
pub mod gamepad;
pub mod gamepad_axis;
pub mod gamepad_button;
pub mod gl;
pub mod gl_matrix;
pub mod guid;
pub mod hash;
pub mod hash_grid;
pub mod hash_map;
pub mod hat_dir;
pub mod hmgui;
pub mod imgui;
pub mod input;
pub mod input_bindings;
pub mod input_event;
pub mod intersect;
pub mod joystick;
pub mod kd_tree;
pub mod key;
pub mod keyboard;
pub mod line_segment;
pub mod lod_mesh;
pub mod lua;
pub mod lua_scheduler;
pub mod math;
pub mod matrix;
pub mod mem_pool;
pub mod mem_stack;
pub mod memory;
pub mod mesh;
pub mod mesh_compute_ao;
pub mod mesh_from_obj;
pub mod meshes;
pub mod metric;
pub mod midi;
pub mod modifier;
pub mod mouse;
pub mod mouse_button;
pub mod octree;
pub mod open_gl;
pub mod os;
pub mod physics;
pub mod pixel_format;
pub mod plane;
pub mod polygon;
pub mod profiler;
pub mod quat;
pub mod ray;
pub mod render_state;
pub mod render_target;
pub mod resource;
pub mod resource_type;
pub mod rigid_body;
pub mod rng;
pub mod sdf;
pub mod shader;
pub mod shader_state;
pub mod shader_var;
pub mod shader_var_type;
pub mod signal;
pub mod sound;
pub mod sound_desc;
pub mod state;
pub mod str_map;
pub mod tex1d;
pub mod tex2d;
pub mod tex2d_load;
pub mod tex2d_save;
pub mod tex3d;
pub mod tex_cube_gen_ir_map;
pub mod tex_filter;
pub mod tex_format;
pub mod tex_wrap_mode;
pub mod texcube;
pub mod thread;
pub mod thread_pool;
pub mod time;
pub mod time_stamp;
pub mod timer;
pub mod triangle;
pub mod trigger;
pub mod ui_renderer;
pub mod viewport;
pub mod window;
pub mod window_mode;
pub mod window_pos;
