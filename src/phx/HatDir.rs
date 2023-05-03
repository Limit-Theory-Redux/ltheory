use crate::phx::internal::Memory::*;
use crate::phx::Common::*;
use crate::phx::Math::Vec3;
use sdl2_sys::*;

pub type HatDir = u32;

#[no_mangle]
pub static HatDir_Centered: HatDir = SDL_HAT_CENTERED;

#[no_mangle]
pub static HatDir_Up: HatDir = SDL_HAT_UP;

#[no_mangle]
pub static HatDir_Right: HatDir = SDL_HAT_RIGHT;

#[no_mangle]
pub static HatDir_Down: HatDir = SDL_HAT_DOWN;

#[no_mangle]
pub static HatDir_Left: HatDir = SDL_HAT_LEFT;

#[no_mangle]
pub static HatDir_RightUp: HatDir = SDL_HAT_RIGHTUP;

#[no_mangle]
pub static HatDir_RightDown: HatDir = SDL_HAT_RIGHTDOWN;

#[no_mangle]
pub static HatDir_LeftUp: HatDir = SDL_HAT_LEFTUP;

#[no_mangle]
pub static HatDir_LeftDown: HatDir = SDL_HAT_LEFTDOWN;
