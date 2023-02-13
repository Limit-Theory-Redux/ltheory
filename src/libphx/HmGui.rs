use ::libc;
use super::internal::Memory::*;
extern "C" {
    pub type Font;
    pub type HashMap;
    pub type Tex2D;
    fn memcpy(
        _: *mut libc::c_void,
        _: *const libc::c_void,
        _: libc::c_ulong,
    ) -> *mut libc::c_void;
    fn malloc(_: libc::c_ulong) -> *mut libc::c_void;
    fn free(_: *mut libc::c_void);
    fn Font_Load(name: cstr, size: libc::c_int) -> *mut Font;
    fn Font_GetSize2(_: *mut Font, out: *mut Vec2i, text: cstr);
    fn Hash_FNV64_Init() -> uint64;
    fn Hash_FNV64_Incremental(
        _: uint64,
        buf: *const libc::c_void,
        len: libc::c_int,
    ) -> uint64;
    fn HashMap_Create(keySize: uint32, capacity: uint32) -> *mut HashMap;
    fn HashMap_GetRaw(_: *mut HashMap, keyHash: uint64) -> *mut libc::c_void;
    fn HashMap_SetRaw(_: *mut HashMap, keyHash: uint64, value: *mut libc::c_void);
    fn Input_GetPressed(_: Button) -> bool;
    fn Input_GetDown(_: Button) -> bool;
    fn Input_GetMouseDelta(_: *mut Vec2i);
    fn Input_GetMousePosition(_: *mut Vec2i);
    fn Input_GetMouseScroll(_: *mut Vec2i);
    fn Profiler_Begin(_: cstr);
    fn Profiler_End();
    fn RenderState_PushBlendMode(_: BlendMode);
    fn RenderState_PopBlendMode();
    fn UIRenderer_Begin();
    fn UIRenderer_End();
    fn UIRenderer_Draw();
    fn UIRenderer_BeginLayer(
        x: libc::c_float,
        y: libc::c_float,
        sx: libc::c_float,
        sy: libc::c_float,
        clip: bool,
    );
    fn UIRenderer_EndLayer();
    fn UIRenderer_Image(
        _: *mut Tex2D,
        x: libc::c_float,
        y: libc::c_float,
        sx: libc::c_float,
        sy: libc::c_float,
    );
    fn UIRenderer_Panel(
        x: libc::c_float,
        y: libc::c_float,
        sx: libc::c_float,
        sy: libc::c_float,
        r: libc::c_float,
        g: libc::c_float,
        b: libc::c_float,
        a: libc::c_float,
        bevel: libc::c_float,
        innerAlpha: libc::c_float,
    );
    fn UIRenderer_Rect(
        x: libc::c_float,
        y: libc::c_float,
        sx: libc::c_float,
        sy: libc::c_float,
        r: libc::c_float,
        g: libc::c_float,
        b: libc::c_float,
        a: libc::c_float,
        outline: bool,
    );
    fn UIRenderer_Text(
        font: *mut Font,
        text: cstr,
        x: libc::c_float,
        y: libc::c_float,
        r: libc::c_float,
        g: libc::c_float,
        b: libc::c_float,
        a: libc::c_float,
    );
}
pub type int32_t = libc::c_int;
pub type uint32_t = libc::c_uint;
pub type uint64_t = libc::c_ulonglong;
pub type __darwin_size_t = libc::c_ulong;
pub type size_t = __darwin_size_t;
pub type cstr = *const libc::c_char;
pub type int32 = int32_t;
pub type uint32 = uint32_t;
pub type uint64 = uint64_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Vec2i {
    pub x: libc::c_int,
    pub y: libc::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Vec2f {
    pub x: libc::c_float,
    pub y: libc::c_float,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Vec4f {
    pub x: libc::c_float,
    pub y: libc::c_float,
    pub z: libc::c_float,
    pub w: libc::c_float,
}
pub type BlendMode = int32;
pub type Button = int32;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct HmGuiGroup {
    pub widget: HmGuiWidget,
    pub head: *mut HmGuiWidget,
    pub tail: *mut HmGuiWidget,
    pub layout: uint32,
    pub children: uint32,
    pub focusStyle: uint32,
    pub paddingLower: Vec2f,
    pub paddingUpper: Vec2f,
    pub offset: Vec2f,
    pub maxSize: Vec2f,
    pub totalStretch: Vec2f,
    pub spacing: libc::c_float,
    pub frameOpacity: libc::c_float,
    pub clip: bool,
    pub expand: bool,
    pub focusable: [bool; 2],
    pub storeSize: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct HmGuiWidget {
    pub parent: *mut HmGuiGroup,
    pub next: *mut HmGuiWidget,
    pub prev: *mut HmGuiWidget,
    pub hash: uint64,
    pub type_0: uint32,
    pub pos: Vec2f,
    pub size: Vec2f,
    pub minSize: Vec2f,
    pub align: Vec2f,
    pub stretch: Vec2f,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct HmGui {
    pub group: *mut HmGuiGroup,
    pub root: *mut HmGuiGroup,
    pub last: *mut HmGuiWidget,
    pub style: *mut HmGuiStyle,
    pub clipRect: *mut HmGuiClipRect,
    pub data: *mut HashMap,
    pub focus: [uint64; 2],
    pub focusPos: Vec2f,
    pub activate: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct HmGuiClipRect {
    pub prev: *mut HmGuiClipRect,
    pub lower: Vec2f,
    pub upper: Vec2f,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct HmGuiStyle {
    pub prev: *mut HmGuiStyle,
    pub font: *mut Font,
    pub spacing: libc::c_float,
    pub colorPrimary: Vec4f,
    pub colorFrame: Vec4f,
    pub colorText: Vec4f,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct HmGuiText {
    pub widget: HmGuiWidget,
    pub font: *mut Font,
    pub text: cstr,
    pub color: Vec4f,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct HmGuiData {
    pub offset: Vec2f,
    pub minSize: Vec2f,
    pub size: Vec2f,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct HmGuiImage {
    pub widget: HmGuiWidget,
    pub image: *mut Tex2D,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct HmGuiRect {
    pub widget: HmGuiWidget,
    pub color: Vec4f,
}
#[no_mangle]
pub static mut Button_Last: Button = 0;
#[no_mangle]
pub static mut Button_System_Last: Button = 0;
#[no_mangle]
pub static mut Button_System_Exit: Button = 0;
#[no_mangle]
pub static mut Button_System_First: Button = 0;
#[no_mangle]
pub static mut Button_Gamepad_Last: Button = 0;
#[no_mangle]
pub static mut Button_Gamepad_Axis_Last: Button = 0;
#[no_mangle]
pub static mut Button_Gamepad_RStickY: Button = 0;
#[no_mangle]
pub static mut Button_Gamepad_RStickX: Button = 0;
#[no_mangle]
pub static mut Button_Gamepad_LStickY: Button = 0;
#[no_mangle]
pub static mut Button_Gamepad_LStickX: Button = 0;
#[no_mangle]
pub static mut Button_Gamepad_RTrigger: Button = 0;
#[no_mangle]
pub static mut Button_Gamepad_LTrigger: Button = 0;
#[no_mangle]
pub static mut Button_Gamepad_Axis_First: Button = 0;
#[no_mangle]
pub static mut Button_Gamepad_Button_Last: Button = 0;
#[no_mangle]
pub static mut Button_Gamepad_Right: Button = 0;
#[no_mangle]
pub static mut Button_Gamepad_Left: Button = 0;
#[no_mangle]
pub static mut Button_Gamepad_Down: Button = 0;
#[no_mangle]
pub static mut Button_Gamepad_Up: Button = 0;
#[no_mangle]
pub static mut Button_Gamepad_RBumper: Button = 0;
#[no_mangle]
pub static mut Button_Gamepad_LBumper: Button = 0;
#[no_mangle]
pub static mut Button_Gamepad_RStick: Button = 0;
#[no_mangle]
pub static mut Button_Gamepad_LStick: Button = 0;
#[no_mangle]
pub static mut Button_Gamepad_Start: Button = 0;
#[no_mangle]
pub static mut Button_Gamepad_Guide: Button = 0;
#[no_mangle]
pub static mut Button_Gamepad_Back: Button = 0;
#[no_mangle]
pub static mut Button_Gamepad_Y: Button = 0;
#[no_mangle]
pub static mut Button_Gamepad_X: Button = 0;
#[no_mangle]
pub static mut Button_Gamepad_B: Button = 0;
#[no_mangle]
pub static mut Button_Gamepad_A: Button = 0;
#[no_mangle]
pub static mut Button_Gamepad_Button_First: Button = 0;
#[no_mangle]
pub static mut Button_Gamepad_First: Button = 0;
#[no_mangle]
pub static mut Button_Mouse_Last: Button = 0;
#[no_mangle]
pub static mut Button_Mouse_ScrollY: Button = 0;
#[no_mangle]
pub static mut Button_Mouse_ScrollX: Button = 0;
#[no_mangle]
pub static mut Button_Mouse_Y: Button = 0;
#[no_mangle]
pub static mut Button_Mouse_X: Button = 0;
#[no_mangle]
pub static mut Button_Mouse_X2: Button = 0;
#[no_mangle]
pub static mut Button_Mouse_X1: Button = 0;
#[no_mangle]
pub static mut Button_Mouse_Right: Button = 0;
#[no_mangle]
pub static mut Button_Mouse_Middle: Button = 0;
#[no_mangle]
pub static mut Button_Mouse_Left: Button = 0;
#[no_mangle]
pub static mut Button_Mouse_First: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_Last: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_RMeta: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_RAlt: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_RShift: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_RCtrl: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_LMeta: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_LAlt: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_LShift: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_LCtrl: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_Up: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_Down: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_Left: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_Right: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_PageDown: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_PageUp: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_End: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_Home: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_Delete: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_Insert: Button = 0;
#[no_mangle]
pub static mut Button_Null: Button = 0;
#[no_mangle]
pub static mut Button_First: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_First: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_A: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_B: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_C: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_D: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_E: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_F: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_G: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_H: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_I: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_J: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_K: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_L: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_M: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_N: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_O: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_P: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_Q: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_R: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_S: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_T: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_U: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_V: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_W: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_X: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_Y: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_Z: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_N0: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_N1: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_N2: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_N3: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_N4: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_N5: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_N6: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_N7: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_N8: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_N9: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_F1: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_F2: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_F3: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_F4: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_F5: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_F6: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_F7: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_F8: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_F9: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_F10: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_F11: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_F12: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_F13: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_F14: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_F15: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_F16: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_F17: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_F18: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_F19: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_F20: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_F21: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_F22: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_F23: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_F24: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_KP0: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_KP1: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_KP2: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_KP3: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_KP4: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_KP5: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_KP6: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_KP7: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_KP8: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_KP9: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_KPNumLock: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_KPDivide: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_KPMultiply: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_KPSubtract: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_KPAdd: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_KPEnter: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_KPDecimal: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_Backspace: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_Escape: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_Return: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_Space: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_Tab: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_Backtick: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_CapsLock: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_Minus: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_Equals: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_LBracket: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_RBracket: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_Backslash: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_Semicolon: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_Apostrophe: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_Comma: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_Period: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_Slash: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_PrintScreen: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_ScrollLock: Button = 0;
#[no_mangle]
pub static mut Button_Keyboard_Pause: Button = 0;
#[inline]
unsafe extern "C" fn Vec2f_Create(mut x: libc::c_float, mut y: libc::c_float) -> Vec2f {
    let mut self_1: Vec2f = {
        let mut init = Vec2f { x: x, y: y };
        init
    };
    return self_1;
}
#[inline]
unsafe extern "C" fn Clamp(
    mut t: libc::c_double,
    mut lower: libc::c_double,
    mut upper: libc::c_double,
) -> libc::c_double {
    t = if t > upper { upper } else { t };
    t = if t < lower { lower } else { t };
    return t;
}
#[inline]
unsafe extern "C" fn Lerp(
    mut a: libc::c_double,
    mut b: libc::c_double,
    mut t: libc::c_double,
) -> libc::c_double {
    return a + t * (b - a);
}
#[inline]
unsafe extern "C" fn Max(
    mut a: libc::c_double,
    mut b: libc::c_double,
) -> libc::c_double {
    return if a > b { a } else { b };
}
#[inline]
unsafe extern "C" fn Min(
    mut a: libc::c_double,
    mut b: libc::c_double,
) -> libc::c_double {
    return if a < b { a } else { b };
}
#[inline]
unsafe extern "C" fn Vec4f_Create(
    mut x: libc::c_float,
    mut y: libc::c_float,
    mut z: libc::c_float,
    mut w: libc::c_float,
) -> Vec4f {
    let mut self_1: Vec4f = {
        let mut init = Vec4f { x: x, y: y, z: z, w: w };
        init
    };
    return self_1;
}

#[inline]
unsafe extern "C" fn StrAlloc(mut len: size_t) -> *mut libc::c_char {
    return malloc(len) as *mut libc::c_char;
}
#[inline]
unsafe extern "C" fn StrFree(mut s: cstr) {
    free(s as *mut libc::c_void);
}
#[inline]
unsafe extern "C" fn StrDup(mut s: cstr) -> cstr {
    if s.is_null() {
        return 0 as cstr;
    }
    let mut len: size_t = (StrLen(s)).wrapping_add(1 as libc::c_int as libc::c_ulong);
    let mut buf: *mut libc::c_char = StrAlloc(len);
    memcpy(buf as *mut libc::c_void, s as *const libc::c_void, len);
    return buf as cstr;
}
#[inline]
unsafe extern "C" fn StrLen(mut s: cstr) -> size_t {
    if s.is_null() {
        return 0 as libc::c_int as size_t;
    }
    let mut begin: cstr = s;
    while *s != 0 {
        s = s.offset(1);
    }
    return s.offset_from(begin) as libc::c_long as size_t;
}
static mut self_0: HmGui = {
    let mut init = HmGui {
        group: 0 as *const HmGuiGroup as *mut HmGuiGroup,
        root: 0 as *const HmGuiGroup as *mut HmGuiGroup,
        last: 0 as *const HmGuiWidget as *mut HmGuiWidget,
        style: 0 as *const HmGuiStyle as *mut HmGuiStyle,
        clipRect: 0 as *const HmGuiClipRect as *mut HmGuiClipRect,
        data: 0 as *const HashMap as *mut HashMap,
        focus: [0; 2],
        focusPos: Vec2f { x: 0., y: 0. },
        activate: false,
    };
    init
};
static mut init: bool = 0 as libc::c_int != 0;
unsafe extern "C" fn HmGui_InitWidget(mut e: *mut HmGuiWidget, mut type_0: uint32) {
    (*e).parent = self_0.group;
    (*e).next = 0 as *mut HmGuiWidget;
    (*e)
        .prev = if !(self_0.group).is_null() {
        (*self_0.group).tail
    } else {
        0 as *mut HmGuiWidget
    };
    if !((*e).parent).is_null() {
        (*(*e).parent).children = ((*(*e).parent).children).wrapping_add(1);
        (*e)
            .hash = Hash_FNV64_Incremental(
            (*(*e).parent).widget.hash,
            &mut (*(*e).parent).children as *mut uint32 as *const libc::c_void,
            ::core::mem::size_of::<uint32>() as libc::c_ulong as libc::c_int,
        );
        if !((*e).next).is_null() {
            (*(*e).next).prev = e;
        } else {
            (*(*e).parent).tail = e;
        }
        if !((*e).prev).is_null() {
            (*(*e).prev).next = e;
        } else {
            (*(*e).parent).head = e;
        }
    } else {
        (*e).hash = Hash_FNV64_Init();
    }
    (*e).type_0 = type_0;
    (*e)
        .pos = Vec2f_Create(
        0 as libc::c_int as libc::c_float,
        0 as libc::c_int as libc::c_float,
    );
    (*e)
        .size = Vec2f_Create(
        0 as libc::c_int as libc::c_float,
        0 as libc::c_int as libc::c_float,
    );
    (*e)
        .minSize = Vec2f_Create(
        0 as libc::c_int as libc::c_float,
        0 as libc::c_int as libc::c_float,
    );
    (*e)
        .align = Vec2f_Create(
        0 as libc::c_int as libc::c_float,
        0 as libc::c_int as libc::c_float,
    );
    (*e)
        .stretch = Vec2f_Create(
        0 as libc::c_int as libc::c_float,
        0 as libc::c_int as libc::c_float,
    );
    self_0.last = e;
}
unsafe extern "C" fn HmGui_BeginGroup(mut layout: uint32) {
    let mut e: *mut HmGuiGroup = MemAlloc(
        ::core::mem::size_of::<HmGuiGroup>() as libc::c_ulong,
    ) as *mut HmGuiGroup;
    HmGui_InitWidget(&mut (*e).widget, 0 as libc::c_int as uint32);
    (*e).head = 0 as *mut HmGuiWidget;
    (*e).tail = 0 as *mut HmGuiWidget;
    (*e).layout = layout;
    (*e).children = 0 as libc::c_int as uint32;
    (*e).focusStyle = 0 as libc::c_int as uint32;
    (*e)
        .paddingLower = Vec2f_Create(
        0 as libc::c_int as libc::c_float,
        0 as libc::c_int as libc::c_float,
    );
    (*e)
        .paddingUpper = Vec2f_Create(
        0 as libc::c_int as libc::c_float,
        0 as libc::c_int as libc::c_float,
    );
    (*e)
        .offset = Vec2f_Create(
        0 as libc::c_int as libc::c_float,
        0 as libc::c_int as libc::c_float,
    );
    (*e).maxSize = Vec2f_Create(1e30f32, 1e30f32);
    (*e).spacing = (*self_0.style).spacing;
    (*e).frameOpacity = 0.0f32;
    (*e).clip = 0 as libc::c_int != 0;
    (*e).expand = 1 as libc::c_int != 0;
    let mut i: libc::c_int = 0 as libc::c_int;
    while i < 2 as libc::c_int {
        (*e).focusable[i as usize] = 0 as libc::c_int != 0;
        i += 1;
    }
    (*e).storeSize = 0 as libc::c_int != 0;
    self_0.group = e;
    match layout {
        1 => {
            (*e)
                .widget
                .stretch = Vec2f_Create(
                1 as libc::c_int as libc::c_float,
                1 as libc::c_int as libc::c_float,
            );
        }
        2 => {
            (*e)
                .widget
                .stretch = Vec2f_Create(
                1 as libc::c_int as libc::c_float,
                0 as libc::c_int as libc::c_float,
            );
        }
        3 => {
            (*e)
                .widget
                .stretch = Vec2f_Create(
                0 as libc::c_int as libc::c_float,
                1 as libc::c_int as libc::c_float,
            );
        }
        _ => {}
    };
}
unsafe extern "C" fn HmGui_FreeText(mut e: *mut HmGuiText) {
    StrFree((*e).text);
    MemFree(e as *const libc::c_void);
}
unsafe extern "C" fn HmGui_FreeGroup(mut g: *mut HmGuiGroup) {
    let mut e: *mut HmGuiWidget = (*g).head;
    while !e.is_null() {
        let mut next: *mut HmGuiWidget = (*e).next;
        match (*e).type_0 {
            0 => {
                HmGui_FreeGroup(e as *mut HmGuiGroup);
            }
            1 => {
                HmGui_FreeText(e as *mut HmGuiText);
            }
            _ => {
                MemFree(e as *const libc::c_void);
            }
        }
        e = next;
    }
    MemFree(g as *const libc::c_void);
}
unsafe extern "C" fn HmGui_GetData(mut g: *mut HmGuiGroup) -> *mut HmGuiData {
    let mut data: *mut HmGuiData = HashMap_GetRaw(self_0.data, (*g).widget.hash)
        as *mut HmGuiData;
    if data.is_null() {
        data = MemAlloc(::core::mem::size_of::<HmGuiData>())
            as *mut HmGuiData;
        (*data)
            .offset = Vec2f_Create(
            0 as libc::c_int as libc::c_float,
            0 as libc::c_int as libc::c_float,
        );
        (*data)
            .minSize = Vec2f_Create(
            0 as libc::c_int as libc::c_float,
            0 as libc::c_int as libc::c_float,
        );
        (*data)
            .size = Vec2f_Create(
            0 as libc::c_int as libc::c_float,
            0 as libc::c_int as libc::c_float,
        );
        HashMap_SetRaw(self_0.data, (*g).widget.hash, data as *mut libc::c_void);
    }
    return data;
}
unsafe extern "C" fn HmGui_ComputeSize(mut g: *mut HmGuiGroup) {
    let mut e: *mut HmGuiWidget = (*g).head;
    while !e.is_null() {
        if (*e).type_0 == 0 as libc::c_int as libc::c_uint {
            HmGui_ComputeSize(e as *mut HmGuiGroup);
        }
        e = (*e).next;
    }
    (*g)
        .widget
        .minSize = Vec2f_Create(
        0 as libc::c_int as libc::c_float,
        0 as libc::c_int as libc::c_float,
    );
    let mut e_0: *mut HmGuiWidget = (*g).head;
    while !e_0.is_null() {
        match (*g).layout {
            1 => {
                (*g)
                    .widget
                    .minSize
                    .x = Max(
                    (*g).widget.minSize.x as libc::c_double,
                    (*e_0).minSize.x as libc::c_double,
                ) as libc::c_float;
                (*g)
                    .widget
                    .minSize
                    .y = Max(
                    (*g).widget.minSize.y as libc::c_double,
                    (*e_0).minSize.y as libc::c_double,
                ) as libc::c_float;
            }
            2 => {
                (*g)
                    .widget
                    .minSize
                    .x = Max(
                    (*g).widget.minSize.x as libc::c_double,
                    (*e_0).minSize.x as libc::c_double,
                ) as libc::c_float;
                (*g).widget.minSize.y += (*e_0).minSize.y;
                if e_0 != (*g).head {
                    (*g).widget.minSize.y += (*g).spacing;
                }
            }
            3 => {
                (*g).widget.minSize.x += (*e_0).minSize.x;
                (*g)
                    .widget
                    .minSize
                    .y = Max(
                    (*g).widget.minSize.y as libc::c_double,
                    (*e_0).minSize.y as libc::c_double,
                ) as libc::c_float;
                if e_0 != (*g).head {
                    (*g).widget.minSize.x += (*g).spacing;
                }
            }
            _ => {}
        }
        e_0 = (*e_0).next;
    }
    (*g).widget.minSize.x += (*g).paddingLower.x + (*g).paddingUpper.x;
    (*g).widget.minSize.y += (*g).paddingLower.y + (*g).paddingUpper.y;
    if (*g).storeSize {
        let mut data: *mut HmGuiData = HmGui_GetData(g);
        (*data).minSize = (*g).widget.minSize;
    }
    (*g)
        .widget
        .minSize
        .x = Min(
        (*g).widget.minSize.x as libc::c_double,
        (*g).maxSize.x as libc::c_double,
    ) as libc::c_float;
    (*g)
        .widget
        .minSize
        .y = Min(
        (*g).widget.minSize.y as libc::c_double,
        (*g).maxSize.y as libc::c_double,
    ) as libc::c_float;
}
unsafe extern "C" fn HmGui_LayoutWidget(
    mut e: *mut HmGuiWidget,
    mut pos: Vec2f,
    mut sx: libc::c_float,
    mut sy: libc::c_float,
) {
    (*e).pos = pos;
    (*e).size = (*e).minSize;
    (*e).size.x += (*e).stretch.x * (sx - (*e).minSize.x);
    (*e).size.y += (*e).stretch.y * (sy - (*e).minSize.y);
    (*e).pos.x += (*e).align.x * (sx - (*e).size.x);
    (*e).pos.y += (*e).align.y * (sy - (*e).size.y);
}
unsafe extern "C" fn HmGui_LayoutGroup(mut g: *mut HmGuiGroup) {
    let mut pos: Vec2f = (*g).widget.pos;
    let mut size: Vec2f = (*g).widget.size;
    let mut extra: libc::c_float = 0 as libc::c_int as libc::c_float;
    let mut totalStretch: libc::c_float = 0 as libc::c_int as libc::c_float;
    pos.x += (*g).paddingLower.x + (*g).offset.x;
    pos.y += (*g).paddingLower.y + (*g).offset.y;
    size.x -= (*g).paddingLower.x + (*g).paddingUpper.x;
    size.y -= (*g).paddingLower.y + (*g).paddingUpper.y;
    if (*g).expand {
        if (*g).layout == 2 as libc::c_int as libc::c_uint {
            extra = (*g).widget.size.y - (*g).widget.minSize.y;
            let mut e: *mut HmGuiWidget = (*g).head;
            while !e.is_null() {
                totalStretch += (*e).stretch.y;
                e = (*e).next;
            }
        } else if (*g).layout == 3 as libc::c_int as libc::c_uint {
            extra = (*g).widget.size.x - (*g).widget.minSize.x;
            let mut e_0: *mut HmGuiWidget = (*g).head;
            while !e_0.is_null() {
                totalStretch += (*e_0).stretch.x;
                e_0 = (*e_0).next;
            }
        }
        if totalStretch > 0 as libc::c_int as libc::c_float {
            extra /= totalStretch;
        }
    }
    let mut s: libc::c_float = 0.;
    let mut e_1: *mut HmGuiWidget = (*g).head;
    while !e_1.is_null() {
        match (*g).layout {
            0 => {
                HmGui_LayoutWidget(e_1, (*e_1).pos, size.x, size.y);
            }
            1 => {
                HmGui_LayoutWidget(e_1, pos, size.x, size.y);
            }
            2 => {
                s = (*e_1).minSize.y;
                if extra > 0 as libc::c_int as libc::c_float {
                    s += (*e_1).stretch.y * extra;
                }
                HmGui_LayoutWidget(e_1, pos, size.x, s);
                pos.y += (*e_1).size.y + (*g).spacing;
            }
            3 => {
                s = (*e_1).minSize.x;
                if extra > 0 as libc::c_int as libc::c_float {
                    s += (*e_1).stretch.x * extra;
                }
                HmGui_LayoutWidget(e_1, pos, s, size.y);
                pos.x += (*e_1).size.x + (*g).spacing;
            }
            _ => {}
        }
        if (*e_1).type_0 == 0 as libc::c_int as libc::c_uint {
            HmGui_LayoutGroup(e_1 as *mut HmGuiGroup);
        }
        e_1 = (*e_1).next;
    }
    if (*g).storeSize {
        let mut data: *mut HmGuiData = HmGui_GetData(g);
        (*data).size = (*g).widget.size;
    }
}
#[inline]
unsafe extern "C" fn IsClipped(mut g: *mut HmGuiGroup, mut p: Vec2f) -> bool {
    return p.x < (*g).widget.pos.x || p.y < (*g).widget.pos.y
        || (*g).widget.pos.x + (*g).widget.size.x < p.x
        || (*g).widget.pos.y + (*g).widget.size.y < p.y;
}
unsafe extern "C" fn HmGui_CheckFocus(mut g: *mut HmGuiGroup) {
    if (*g).clip as libc::c_int != 0 && IsClipped(g, self_0.focusPos) as libc::c_int != 0
    {
        return;
    }
    let mut e: *mut HmGuiWidget = (*g).tail;
    while !e.is_null() {
        if (*e).type_0 == 0 as libc::c_int as libc::c_uint {
            HmGui_CheckFocus(e as *mut HmGuiGroup);
        }
        e = (*e).prev;
    }
    let mut i: libc::c_int = 0 as libc::c_int;
    while i < 2 as libc::c_int {
        if self_0.focus[i as usize] == 0 as libc::c_int as libc::c_ulonglong
            && (*g).focusable[i as usize] as libc::c_int != 0
        {
            if (*g).widget.pos.x <= self_0.focusPos.x
                && (*g).widget.pos.y <= self_0.focusPos.y
                && self_0.focusPos.x <= (*g).widget.pos.x + (*g).widget.size.x
                && self_0.focusPos.y <= (*g).widget.pos.y + (*g).widget.size.y
            {
                self_0.focus[i as usize] = (*g).widget.hash;
            }
        }
        i += 1;
    }
}
unsafe extern "C" fn HmGui_DrawText(mut e: *mut HmGuiText) {
    UIRenderer_Text(
        (*e).font,
        (*e).text,
        (*e).widget.pos.x,
        (*e).widget.pos.y + (*e).widget.minSize.y,
        (*e).color.x,
        (*e).color.y,
        (*e).color.z,
        (*e).color.w,
    );
}
unsafe extern "C" fn HmGui_DrawRect(mut e: *mut HmGuiRect) {
    UIRenderer_Rect(
        (*e).widget.pos.x,
        (*e).widget.pos.y,
        (*e).widget.size.x,
        (*e).widget.size.y,
        (*e).color.x,
        (*e).color.y,
        (*e).color.z,
        (*e).color.w,
        0 as libc::c_int != 0,
    );
}
unsafe extern "C" fn HmGui_DrawImage(mut e: *mut HmGuiImage) {
    UIRenderer_Image(
        (*e).image,
        (*e).widget.pos.x,
        (*e).widget.pos.y,
        (*e).widget.size.x,
        (*e).widget.size.y,
    );
}
unsafe extern "C" fn HmGui_DrawGroup(mut g: *mut HmGuiGroup) {
    UIRenderer_BeginLayer(
        (*g).widget.pos.x,
        (*g).widget.pos.y,
        (*g).widget.size.x,
        (*g).widget.size.y,
        (*g).clip,
    );
    let mut e: *mut HmGuiWidget = (*g).tail;
    while !e.is_null() {
        match (*e).type_0 {
            0 => {
                HmGui_DrawGroup(e as *mut HmGuiGroup);
            }
            1 => {
                HmGui_DrawText(e as *mut HmGuiText);
            }
            2 => {
                HmGui_DrawRect(e as *mut HmGuiRect);
            }
            3 => {
                HmGui_DrawImage(e as *mut HmGuiImage);
            }
            _ => {}
        }
        e = (*e).prev;
    }
    if (*g).focusable[0 as libc::c_int as usize] {
        let mut focus: bool = self_0.focus[0 as libc::c_int as usize]
            == (*g).widget.hash;
        if (*g).focusStyle == 0 as libc::c_int as libc::c_uint {
            UIRenderer_Panel(
                (*g).widget.pos.x,
                (*g).widget.pos.y,
                (*g).widget.size.x,
                (*g).widget.size.y,
                0.1f32,
                0.12f32,
                0.13f32,
                1.0f32,
                8.0f32,
                (*g).frameOpacity,
            );
        } else if (*g).focusStyle == 1 as libc::c_int as libc::c_uint {
            if focus {
                UIRenderer_Panel(
                    (*g).widget.pos.x,
                    (*g).widget.pos.y,
                    (*g).widget.size.x,
                    (*g).widget.size.y,
                    0.1f32,
                    0.5f32,
                    1.0f32,
                    1.0f32,
                    0.0f32,
                    1.0f32,
                );
            } else {
                UIRenderer_Panel(
                    (*g).widget.pos.x,
                    (*g).widget.pos.y,
                    (*g).widget.size.x,
                    (*g).widget.size.y,
                    0.15f32,
                    0.15f32,
                    0.15f32,
                    0.8f32,
                    0.0f32,
                    (*g).frameOpacity,
                );
            }
        } else if (*g).focusStyle == 2 as libc::c_int as libc::c_uint {
            if focus {
                UIRenderer_Rect(
                    (*g).widget.pos.x,
                    (*g).widget.pos.y,
                    (*g).widget.size.x,
                    (*g).widget.size.y,
                    0.1f32,
                    0.5f32,
                    1.0f32,
                    1.0f32,
                    1 as libc::c_int != 0,
                );
            }
        } else if (*g).focusStyle == 3 as libc::c_int as libc::c_uint {
            UIRenderer_Rect(
                (*g).widget.pos.x,
                (*g).widget.pos.y,
                (*g).widget.size.x,
                (*g).widget.size.y,
                0.3f32,
                0.3f32,
                0.3f32,
                if focus as libc::c_int != 0 { 0.5f32 } else { (*g).frameOpacity },
                0 as libc::c_int != 0,
            );
        }
    }
    UIRenderer_EndLayer();
}
#[no_mangle]
pub unsafe extern "C" fn HmGui_Begin(mut sx: libc::c_float, mut sy: libc::c_float) {
    if !init {
        init = 1 as libc::c_int != 0;
        self_0.group = 0 as *mut HmGuiGroup;
        self_0.root = 0 as *mut HmGuiGroup;
        self_0
            .style = MemAlloc(::core::mem::size_of::<HmGuiStyle>())
            as *mut HmGuiStyle;
        (*self_0.style).prev = 0 as *mut HmGuiStyle;
        (*self_0.style)
            .font = Font_Load(
            b"Rajdhani\0" as *const u8 as *const libc::c_char,
            14 as libc::c_int,
        );
        (*self_0.style).spacing = 6 as libc::c_int as libc::c_float;
        (*self_0.style).colorPrimary = Vec4f_Create(0.1f32, 0.5f32, 1.0f32, 1.0f32);
        (*self_0.style).colorFrame = Vec4f_Create(0.1f32, 0.1f32, 0.1f32, 0.5f32);
        (*self_0.style)
            .colorText = Vec4f_Create(
            1 as libc::c_int as libc::c_float,
            1 as libc::c_int as libc::c_float,
            1 as libc::c_int as libc::c_float,
            1 as libc::c_int as libc::c_float,
        );
        self_0.clipRect = 0 as *mut HmGuiClipRect;
        self_0
            .data = HashMap_Create(
            0 as libc::c_int as uint32,
            128 as libc::c_int as uint32,
        );
        let mut i: libc::c_int = 0 as libc::c_int;
        while i < 2 as libc::c_int {
            self_0.focus[i as usize] = 0 as libc::c_int as uint64;
            i += 1;
        }
        self_0.activate = 0 as libc::c_int != 0;
    }
    if !(self_0.root).is_null() {
        HmGui_FreeGroup(self_0.root);
        self_0.root = 0 as *mut HmGuiGroup;
    }
    self_0.last = 0 as *mut HmGuiWidget;
    self_0.activate = Input_GetPressed(Button_Mouse_Left);
    HmGui_BeginGroup(0 as libc::c_int as uint32);
    (*self_0.group).clip = 1 as libc::c_int != 0;
    (*self_0.group)
        .widget
        .pos = Vec2f_Create(
        0 as libc::c_int as libc::c_float,
        0 as libc::c_int as libc::c_float,
    );
    (*self_0.group).widget.size = Vec2f_Create(sx, sy);
    self_0.root = self_0.group;
}
#[no_mangle]
pub unsafe extern "C" fn HmGui_End() {
    Profiler_Begin(
        (*::core::mem::transmute::<&[u8; 10], &[libc::c_char; 10]>(b"HmGui_End\0"))
            .as_ptr(),
    );
    HmGui_EndGroup();
    HmGui_ComputeSize(self_0.root);
    HmGui_LayoutGroup(self_0.root);
    let mut i: libc::c_int = 0 as libc::c_int;
    while i < 2 as libc::c_int {
        self_0.focus[i as usize] = 0 as libc::c_int as uint64;
        i += 1;
    }
    let mut mouse: Vec2i = Vec2i { x: 0, y: 0 };
    Input_GetMousePosition(&mut mouse);
    self_0.focusPos = Vec2f_Create(mouse.x as libc::c_float, mouse.y as libc::c_float);
    HmGui_CheckFocus(self_0.root);
    Profiler_End();
}
#[no_mangle]
pub unsafe extern "C" fn HmGui_Draw() {
    Profiler_Begin(
        (*::core::mem::transmute::<&[u8; 11], &[libc::c_char; 11]>(b"HmGui_Draw\0"))
            .as_ptr(),
    );
    RenderState_PushBlendMode(1 as libc::c_int);
    UIRenderer_Begin();
    HmGui_DrawGroup(self_0.root);
    UIRenderer_End();
    RenderState_PopBlendMode();
    UIRenderer_Draw();
    Profiler_End();
}
#[no_mangle]
pub unsafe extern "C" fn HmGui_BeginGroupX() {
    HmGui_BeginGroup(3 as libc::c_int as uint32);
}
#[no_mangle]
pub unsafe extern "C" fn HmGui_BeginGroupY() {
    HmGui_BeginGroup(2 as libc::c_int as uint32);
}
#[no_mangle]
pub unsafe extern "C" fn HmGui_BeginGroupStack() {
    HmGui_BeginGroup(1 as libc::c_int as uint32);
}
#[no_mangle]
pub unsafe extern "C" fn HmGui_EndGroup() {
    self_0.last = &mut (*self_0.group).widget;
    self_0.group = (*self_0.group).widget.parent;
}
#[no_mangle]
pub unsafe extern "C" fn HmGui_BeginScroll(mut maxSize: libc::c_float) {
    HmGui_BeginGroupX();
    HmGui_SetStretch(
        1 as libc::c_int as libc::c_float,
        1 as libc::c_int as libc::c_float,
    );
    (*self_0.group).clip = 1 as libc::c_int != 0;
    HmGui_SetSpacing(2 as libc::c_int as libc::c_float);
    HmGui_BeginGroupY();
    HmGui_SetPadding(
        6 as libc::c_int as libc::c_float,
        6 as libc::c_int as libc::c_float,
    );
    HmGui_SetStretch(
        1 as libc::c_int as libc::c_float,
        1 as libc::c_int as libc::c_float,
    );
    (*self_0.group).expand = 0 as libc::c_int != 0;
    (*self_0.group).storeSize = 1 as libc::c_int != 0;
    (*self_0.group).maxSize.y = maxSize;
    let mut data: *mut HmGuiData = HmGui_GetData(self_0.group);
    (*self_0.group).offset.y = -(*data).offset.y;
}
#[no_mangle]
pub unsafe extern "C" fn HmGui_EndScroll() {
    let mut data: *mut HmGuiData = HmGui_GetData(self_0.group);
    if HmGui_GroupHasFocus(1 as libc::c_int) {
        let mut scroll: Vec2i = Vec2i { x: 0, y: 0 };
        Input_GetMouseScroll(&mut scroll);
        (*data).offset.y -= 10.0f32 * scroll.y as libc::c_float;
    }
    let mut maxScroll: libc::c_float = Max(
        0.0f32 as libc::c_double,
        ((*data).minSize.y - (*data).size.y) as libc::c_double,
    ) as libc::c_float;
    (*data)
        .offset
        .y = Clamp(
        (*data).offset.y as libc::c_double,
        0.0f32 as libc::c_double,
        maxScroll as libc::c_double,
    ) as libc::c_float;
    HmGui_EndGroup();
    HmGui_BeginGroupY();
    HmGui_SetStretch(
        0 as libc::c_int as libc::c_float,
        1 as libc::c_int as libc::c_float,
    );
    HmGui_SetSpacing(0 as libc::c_int as libc::c_float);
    if maxScroll > 0 as libc::c_int as libc::c_float {
        let mut handleSize: libc::c_float = (*data).size.y
            * ((*data).size.y / (*data).minSize.y);
        let mut handlePos: libc::c_float = Lerp(
            0.0f32 as libc::c_double,
            ((*data).size.y - handleSize) as libc::c_double,
            ((*data).offset.y / maxScroll) as libc::c_double,
        ) as libc::c_float;
        HmGui_Rect(4.0f32, handlePos, 0.0f32, 0.0f32, 0.0f32, 0.0f32);
        HmGui_Rect(
            4.0f32,
            handleSize,
            (*self_0.style).colorFrame.x,
            (*self_0.style).colorFrame.y,
            (*self_0.style).colorFrame.z,
            (*self_0.style).colorFrame.w,
        );
    } else {
        HmGui_Rect(4.0f32, 16.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32);
    }
    HmGui_EndGroup();
    HmGui_EndGroup();
}
#[no_mangle]
pub unsafe extern "C" fn HmGui_BeginWindow(mut title: cstr) {
    HmGui_BeginGroupStack();
    HmGui_SetStretch(
        0 as libc::c_int as libc::c_float,
        0 as libc::c_int as libc::c_float,
    );
    (*self_0.group).focusStyle = 0 as libc::c_int as uint32;
    (*self_0.group).frameOpacity = 0.95f32;
    let mut data: *mut HmGuiData = HmGui_GetData(self_0.group);
    if HmGui_GroupHasFocus(0 as libc::c_int) {
        if Input_GetDown(Button_Mouse_Left) {
            let mut md: Vec2i = Vec2i { x: 0, y: 0 };
            Input_GetMouseDelta(&mut md);
            (*data).offset.x += md.x as libc::c_float;
            (*data).offset.y += md.y as libc::c_float;
        }
    }
    (*self_0.group).widget.pos.x += (*data).offset.x;
    (*self_0.group).widget.pos.y += (*data).offset.y;
    HmGui_BeginGroupY();
    (*self_0.group).clip = 1 as libc::c_int != 0;
    HmGui_SetPadding(
        8 as libc::c_int as libc::c_float,
        8 as libc::c_int as libc::c_float,
    );
    HmGui_SetStretch(
        1 as libc::c_int as libc::c_float,
        1 as libc::c_int as libc::c_float,
    );
}
#[no_mangle]
pub unsafe extern "C" fn HmGui_EndWindow() {
    HmGui_EndGroup();
    HmGui_EndGroup();
}
#[no_mangle]
pub unsafe extern "C" fn HmGui_Button(mut label: cstr) -> bool {
    HmGui_BeginGroupStack();
    (*self_0.group).focusStyle = 1 as libc::c_int as uint32;
    (*self_0.group).frameOpacity = 0.5f32;
    let mut focus: bool = HmGui_GroupHasFocus(0 as libc::c_int);
    HmGui_SetPadding(
        8 as libc::c_int as libc::c_float,
        8 as libc::c_int as libc::c_float,
    );
    HmGui_Text(label);
    HmGui_SetAlign(0.5f32, 0.5f32);
    HmGui_EndGroup();
    return focus as libc::c_int != 0 && self_0.activate as libc::c_int != 0;
}
#[no_mangle]
pub unsafe extern "C" fn HmGui_Checkbox(mut label: cstr, mut value: bool) -> bool {
    HmGui_BeginGroupX();
    (*self_0.group).focusStyle = 3 as libc::c_int as uint32;
    if HmGui_GroupHasFocus(0 as libc::c_int) as libc::c_int != 0
        && self_0.activate as libc::c_int != 0
    {
        value = !value;
    }
    HmGui_SetPadding(
        4 as libc::c_int as libc::c_float,
        4 as libc::c_int as libc::c_float,
    );
    HmGui_SetSpacing(8 as libc::c_int as libc::c_float);
    HmGui_SetStretch(
        1 as libc::c_int as libc::c_float,
        0 as libc::c_int as libc::c_float,
    );
    HmGui_Text(label);
    HmGui_SetAlign(0.0f32, 0.5f32);
    HmGui_SetStretch(
        1 as libc::c_int as libc::c_float,
        0 as libc::c_int as libc::c_float,
    );
    HmGui_BeginGroupStack();
    HmGui_Rect(
        16 as libc::c_int as libc::c_float,
        16 as libc::c_int as libc::c_float,
        (*self_0.style).colorFrame.x,
        (*self_0.style).colorFrame.y,
        (*self_0.style).colorFrame.z,
        (*self_0.style).colorFrame.w,
    );
    if value {
        HmGui_Rect(
            10 as libc::c_int as libc::c_float,
            10 as libc::c_int as libc::c_float,
            (*self_0.style).colorPrimary.x,
            (*self_0.style).colorPrimary.y,
            (*self_0.style).colorPrimary.z,
            (*self_0.style).colorPrimary.w,
        );
        HmGui_SetAlign(0.5f32, 0.5f32);
    }
    HmGui_EndGroup();
    HmGui_SetStretch(
        0 as libc::c_int as libc::c_float,
        0 as libc::c_int as libc::c_float,
    );
    HmGui_EndGroup();
    return value;
}
#[no_mangle]
pub unsafe extern "C" fn HmGui_Slider(
    mut lower: libc::c_float,
    mut upper: libc::c_float,
    mut value: libc::c_float,
) -> libc::c_float {
    HmGui_BeginGroupStack();
    HmGui_Rect(
        0 as libc::c_int as libc::c_float,
        2 as libc::c_int as libc::c_float,
        0.5f32,
        0.5f32,
        0.5f32,
        1.0f32,
    );
    HmGui_SetAlign(0.5f32, 0.5f32);
    HmGui_SetStretch(
        1 as libc::c_int as libc::c_float,
        0 as libc::c_int as libc::c_float,
    );
    HmGui_EndGroup();
    return 0.0f32;
}
#[no_mangle]
pub unsafe extern "C" fn HmGui_Image(mut image: *mut Tex2D) {
    let mut e: *mut HmGuiImage = MemAlloc(
        ::core::mem::size_of::<HmGuiImage>() as libc::c_ulong,
    ) as *mut HmGuiImage;
    HmGui_InitWidget(&mut (*e).widget, 3 as libc::c_int as uint32);
    (*e).image = image;
    (*e)
        .widget
        .stretch = Vec2f_Create(
        1 as libc::c_int as libc::c_float,
        1 as libc::c_int as libc::c_float,
    );
}
#[no_mangle]
pub unsafe extern "C" fn HmGui_Rect(
    mut sx: libc::c_float,
    mut sy: libc::c_float,
    mut r: libc::c_float,
    mut g: libc::c_float,
    mut b: libc::c_float,
    mut a: libc::c_float,
) {
    let mut e: *mut HmGuiRect = MemAlloc(
        ::core::mem::size_of::<HmGuiRect>() as libc::c_ulong,
    ) as *mut HmGuiRect;
    HmGui_InitWidget(&mut (*e).widget, 2 as libc::c_int as uint32);
    (*e).color = Vec4f_Create(r, g, b, a);
    (*e).widget.minSize = Vec2f_Create(sx, sy);
}
#[no_mangle]
pub unsafe extern "C" fn HmGui_Text(mut text: cstr) {
    HmGui_TextEx(
        (*self_0.style).font,
        text,
        (*self_0.style).colorText.x,
        (*self_0.style).colorText.y,
        (*self_0.style).colorText.z,
        (*self_0.style).colorText.w,
    );
}
#[no_mangle]
pub unsafe extern "C" fn HmGui_TextColored(
    mut text: cstr,
    mut r: libc::c_float,
    mut g: libc::c_float,
    mut b: libc::c_float,
    mut a: libc::c_float,
) {
    HmGui_TextEx((*self_0.style).font, text, r, g, b, a);
}
#[no_mangle]
pub unsafe extern "C" fn HmGui_TextEx(
    mut font: *mut Font,
    mut text: cstr,
    mut r: libc::c_float,
    mut g: libc::c_float,
    mut b: libc::c_float,
    mut a: libc::c_float,
) {
    let mut e: *mut HmGuiText = MemAlloc(
        ::core::mem::size_of::<HmGuiText>() as libc::c_ulong,
    ) as *mut HmGuiText;
    HmGui_InitWidget(&mut (*e).widget, 1 as libc::c_int as uint32);
    (*e).font = font;
    (*e).text = StrDup(text);
    (*e).color = Vec4f_Create(r, g, b, a);
    let mut size: Vec2i = Vec2i { x: 0, y: 0 };
    Font_GetSize2((*e).font, &mut size, (*e).text);
    (*e).widget.minSize = Vec2f_Create(size.x as libc::c_float, size.y as libc::c_float);
    HmGui_SetAlign(0.0f32, 1.0f32);
}
#[no_mangle]
pub unsafe extern "C" fn HmGui_SetAlign(mut ax: libc::c_float, mut ay: libc::c_float) {
    (*self_0.last).align = Vec2f_Create(ax, ay);
}
#[no_mangle]
pub unsafe extern "C" fn HmGui_SetPadding(mut px: libc::c_float, mut py: libc::c_float) {
    (*self_0.group).paddingLower = Vec2f_Create(px, py);
    (*self_0.group).paddingUpper = Vec2f_Create(px, py);
}
#[no_mangle]
pub unsafe extern "C" fn HmGui_SetPaddingEx(
    mut left: libc::c_float,
    mut top: libc::c_float,
    mut right: libc::c_float,
    mut bottom: libc::c_float,
) {
    (*self_0.group).paddingLower = Vec2f_Create(left, top);
    (*self_0.group).paddingUpper = Vec2f_Create(right, bottom);
}
#[no_mangle]
pub unsafe extern "C" fn HmGui_SetPaddingLeft(mut padding: libc::c_float) {
    (*self_0.group).paddingLower.x = padding;
}
#[no_mangle]
pub unsafe extern "C" fn HmGui_SetPaddingTop(mut padding: libc::c_float) {
    (*self_0.group).paddingLower.y = padding;
}
#[no_mangle]
pub unsafe extern "C" fn HmGui_SetPaddingRight(mut padding: libc::c_float) {
    (*self_0.group).paddingUpper.x = padding;
}
#[no_mangle]
pub unsafe extern "C" fn HmGui_SetPaddingBottom(mut padding: libc::c_float) {
    (*self_0.group).paddingUpper.y = padding;
}
#[no_mangle]
pub unsafe extern "C" fn HmGui_SetSpacing(mut spacing: libc::c_float) {
    (*self_0.group).spacing = spacing;
}
#[no_mangle]
pub unsafe extern "C" fn HmGui_SetStretch(mut x: libc::c_float, mut y: libc::c_float) {
    (*self_0.last).stretch = Vec2f_Create(x, y);
}
#[no_mangle]
pub unsafe extern "C" fn HmGui_GroupHasFocus(mut type_0: libc::c_int) -> bool {
    (*self_0.group).focusable[type_0 as usize] = 1 as libc::c_int != 0;
    return self_0.focus[type_0 as usize] == (*self_0.group).widget.hash;
}
#[no_mangle]
pub unsafe extern "C" fn HmGui_PushStyle() {
    let mut style: *mut HmGuiStyle = MemAlloc(
        ::core::mem::size_of::<HmGuiStyle>() as libc::c_ulong,
    ) as *mut HmGuiStyle;
    *style = *self_0.style;
    (*style).prev = self_0.style;
    self_0.style = style;
}
#[no_mangle]
pub unsafe extern "C" fn HmGui_PushFont(mut font: *mut Font) {
    HmGui_PushStyle();
    (*self_0.style).font = font;
}
#[no_mangle]
pub unsafe extern "C" fn HmGui_PushTextColor(
    mut r: libc::c_float,
    mut g: libc::c_float,
    mut b: libc::c_float,
    mut a: libc::c_float,
) {
    HmGui_PushStyle();
    (*self_0.style).colorText = Vec4f_Create(r, g, b, a);
}
#[no_mangle]
pub unsafe extern "C" fn HmGui_PopStyle(mut depth: libc::c_int) {
    let mut i: libc::c_int = 0 as libc::c_int;
    while i < depth {
        let mut style: *mut HmGuiStyle = self_0.style;
        self_0.style = (*style).prev;
        MemFree(style as *const libc::c_void);
        i += 1;
    }
}
