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
pub mod common;
pub mod engine;
pub mod error;
pub mod input;
pub mod lua;
pub mod math;
pub mod physics;
pub mod render;
pub mod system;
pub mod ui;
