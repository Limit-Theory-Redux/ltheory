use ::libc;
use glam::Vec3;
use crate::internal::Memory::*;
use crate::State::*;
use glam::Vec2;

extern "C" {
    pub type lua_State;
    fn Input_GetNextEvent(_: *mut InputEvent) -> bool;
    fn pow(_: f64, _: f64) -> f64;
    fn sqrt(_: f64) -> f64;
    fn printf(_: *const libc::c_char, _: ...) -> libc::c_int;
}
pub type __darwin_ptrdiff_t = libc::c_long;
pub type cstr = *const libc::c_char;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct InputBinding {
    pub name: cstr,
    pub rawButtons: [[RawButton; 4]; 4],
    pub pressThreshold: f32,
    pub releaseThreshold: f32,
    pub exponent: f32,
    pub deadzone: f32,
    pub minValue: f32,
    pub maxValue: f32,
    pub luaInstance: *mut Lua,
    pub buttons: [AggregateButton; 4],
    pub axes: [AggregateAxis; 2],
    pub axis2D: AggregateAxis2D,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct AggregateAxis2D {
    pub value: Vec2,
    pub onChanged: LuaRef,
}
pub type LuaRef = lua_Integer;
pub type lua_Integer = ptrdiff_t;
pub type ptrdiff_t = __darwin_ptrdiff_t;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct AggregateAxis {
    pub value: f32,
    pub invert: bool,
    pub onChanged: LuaRef,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct AggregateButton {
    pub state: State,
    pub onPressed: LuaRef,
    pub onDown: LuaRef,
    pub onReleased: LuaRef,
}
pub type State = i32;
pub type Lua = lua_State;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct RawButton {
    pub button: Button,
    pub value: f32,
}
pub type Button = i32;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Device {
    pub type_0: DeviceType,
    pub id: u32,
}
pub type DeviceType = i32;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct InputEvent {
    pub timestamp: u32,
    pub device: Device,
    pub button: Button,
    pub value: f32,
    pub state: State,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct DownBinding {
    pub binding: *mut InputBinding,
    pub button: *mut AggregateButton,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct InputBindings {
    pub activeBindings_size: i32,
    pub activeBindings_capacity: i32,
    pub activeBindings_data: *mut InputBinding,
    pub downBindings_size: i32,
    pub downBindings_capacity: i32,
    pub downBindings_data: *mut DownBinding,
}


#[inline]
unsafe extern "C" fn Pow(
    mut t: f64,
    mut p: f64,
) -> f64 {
    return pow(t, p);
}
#[inline]
unsafe extern "C" fn Sqrtf(mut t: f32) -> f32 {
    return sqrt(t as f64) as f32;
}
#[inline]
unsafe extern "C" fn Clamp(
    mut t: f64,
    mut lower: f64,
    mut upper: f64,
) -> f64 {
    t = if t > upper { upper } else { t };
    t = if t < lower { lower } else { t };
    return t;
}
#[no_mangle]
pub static mut InputBindings_DefaultMaxValue: f32 = 0.;
#[no_mangle]
pub static mut InputBindings_DefaultMinValue: f32 = 0.;
#[no_mangle]
pub static mut InputBindings_DefaultDeadzone: f32 = 0.;
#[no_mangle]
pub static mut InputBindings_DefaultExponent: f32 = 0.;
#[no_mangle]
pub static mut InputBindings_DefaultReleaseThreshold: f32 = 0.;
#[no_mangle]
pub static mut InputBindings_DefaultPressThreshold: f32 = 0.;

static mut BindCount: libc::c_int = 4 as libc::c_int;
static mut this: InputBindings =  InputBindings {
        activeBindings_size: 0 as libc::c_int,
        activeBindings_capacity: 0,
        activeBindings_data: 0 as *const InputBinding as *mut InputBinding,
        downBindings_size: 0,
        downBindings_capacity: 0,
        downBindings_data: 0 as *const DownBinding as *mut DownBinding,
    };
#[no_mangle]
pub unsafe extern "C" fn InputBindings_Init() {
    if (this.activeBindings_capacity < 64 as libc::c_int) as libc::c_int
        as libc::c_long != 0
    {
        this.activeBindings_capacity = 64 as libc::c_int;
        let mut elemSize: usize = ::core::mem::size_of::<InputBinding>();
        let mut pData: *mut *mut libc::c_void = &mut this.activeBindings_data
            as *mut *mut InputBinding as *mut *mut libc::c_void;
        *pData = MemRealloc(
            this.activeBindings_data as *mut libc::c_void,
            (this.activeBindings_capacity as usize).wrapping_mul(elemSize as usize),
        );
    }
    if (this.downBindings_capacity < 8 as libc::c_int) as libc::c_long
        != 0
    {
        this.downBindings_capacity = 8 as libc::c_int;
        let mut elemSize_0: usize = ::core::mem::size_of::<DownBinding>();
        let mut pData_0: *mut *mut libc::c_void = &mut this.downBindings_data
            as *mut *mut DownBinding as *mut *mut libc::c_void;
        *pData_0 = MemRealloc(
            this.downBindings_data as *mut libc::c_void,
            (this.downBindings_capacity as usize).wrapping_mul(elemSize_0 as usize),
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn InputBindings_Free() {
    MemFree(this.activeBindings_data as *const libc::c_void);
    MemFree(this.downBindings_data as *const libc::c_void);
}
unsafe extern "C" fn InputBindings_RaiseCallback(
    mut event: cstr,
    mut binding: *mut InputBinding,
    mut callback: LuaRef,
) {
    printf(b"%s - %s\n\0" as *const u8 as *const libc::c_char, event, (*binding).name);
}
#[no_mangle]
pub unsafe extern "C" fn InputBindings_UpdateBinding(mut binding: *mut InputBinding) {
    let mut value =  Vec2::ZERO;
    let mut axisValues: [*mut f32; 2] = [&mut value.x, &mut value.y];
    let mut iAxis: libc::c_int = 0 as libc::c_int;
    while iAxis
        < (::core::mem::size_of::<[AggregateAxis; 2]>())
            .wrapping_div(::core::mem::size_of::<AggregateAxis>())
            as libc::c_int
    {
        let mut axisValue: *mut f32 = axisValues[iAxis as usize];
        let mut iBind: libc::c_int = 0 as libc::c_int;
        while iBind < BindCount {
            *axisValue
                += (*binding)
                    .rawButtons[(2 as libc::c_int * iAxis + 0 as libc::c_int)
                        as usize][iBind as usize]
                    .value;
            *axisValue
                -= (*binding)
                    .rawButtons[(2 as libc::c_int * iAxis + 1 as libc::c_int)
                        as usize][iBind as usize]
                    .value;
            iBind += 1;
        }
        *axisValue = (*axisValue - (*binding).deadzone) / (1.0f32 - (*binding).deadzone);
        *axisValue = Pow(
            *axisValue as f64,
            (*binding).exponent as f64,
        ) as f32;
        *axisValue = Clamp(
            *axisValue as f64,
            (*binding).minValue as f64,
            (*binding).maxValue as f64,
        ) as f32;
        iAxis += 1;
    }
    let mut len: f32 = value.length();
    if len > 1.0f32 {
        value /= 1.0f32 / len;
    }
    let mut axis2D: *mut AggregateAxis2D = &mut (*binding).axis2D;
    if value != (*axis2D).value {
        (*axis2D).value = value;
        InputBindings_RaiseCallback(
            b"Changed\0" as *const u8 as *const libc::c_char,
            binding,
            (*axis2D).onChanged,
        );
    }
    let mut iAxis_0: libc::c_int = 0 as libc::c_int;
    while iAxis_0
        < (::core::mem::size_of::<[AggregateAxis; 2]>())
            .wrapping_div(::core::mem::size_of::<AggregateAxis>())
            as libc::c_int
    {
        let mut axis: *mut AggregateAxis = &mut *((*binding).axes)
            .as_mut_ptr()
            .offset(iAxis_0 as isize) as *mut AggregateAxis;
        if *axisValues[iAxis_0 as usize] != (*axis).value {
            (*axis).value = *axisValues[iAxis_0 as usize];
            InputBindings_RaiseCallback(
                if iAxis_0 == 0 as libc::c_int {
                    b"Changed X\0" as *const u8 as *const libc::c_char
                } else {
                    b"Changed Y\0" as *const u8 as *const libc::c_char
                },
                binding,
                (*axis).onChanged,
            );
        }
        iAxis_0 += 1;
    }
    let mut iBtn: libc::c_int = 0 as libc::c_int;
    while iBtn
        < (::core::mem::size_of::<[AggregateButton; 4]>())
            .wrapping_div(::core::mem::size_of::<AggregateButton>())
            as libc::c_int
    {
        let mut button: *mut AggregateButton = &mut *((*binding).buttons)
            .as_mut_ptr()
            .offset(iBtn as isize) as *mut AggregateButton;
        let mut axisValue_0: f32 = (*binding)
            .axes[(iBtn / 2 as libc::c_int) as usize]
            .value;
        let mut isPos: bool = iBtn & 1 as libc::c_int == 0;
        if !((*button).state & State_Down == State_Down) {
            if if isPos as libc::c_int != 0 {
                (axisValue_0 > (*binding).pressThreshold) as libc::c_int
            } else {
                (axisValue_0 < -(*binding).pressThreshold) as libc::c_int
            } != 0
            {
                (*button).state |= State_Pressed;
                (*button).state |= State_Down;
                InputBindings_RaiseCallback(
                    b"Pressed\0" as *const u8 as *const libc::c_char,
                    binding,
                    (*button).onPressed,
                );
                let mut downBinding: DownBinding =  DownBinding {
                        binding: 0 as *mut InputBinding,
                        button: 0 as *mut AggregateButton,
                    };
                downBinding.binding = binding;
                downBinding.button = button;
                if (this.downBindings_capacity == this.downBindings_size)
                    as libc::c_long != 0
                {
                    this
                        .downBindings_capacity = if this.downBindings_capacity != 0 {
                        this.downBindings_capacity * 2 as libc::c_int
                    } else {
                        1 as libc::c_int
                    };
                    let mut elemSize: usize = ::core::mem::size_of::<DownBinding>();
                    let mut pData: *mut *mut libc::c_void = &mut this.downBindings_data
                        as *mut *mut DownBinding as *mut *mut libc::c_void;
                    *pData = MemRealloc(
                        this.downBindings_data as *mut libc::c_void,
                        (this.downBindings_capacity as usize)
                            .wrapping_mul(elemSize),
                    );
                }
                let fresh0 = this.downBindings_size;
                this.downBindings_size = this.downBindings_size + 1;
                *(this.downBindings_data).offset(fresh0 as isize) = downBinding;
            }
        } else if if isPos as libc::c_int != 0 {
            (axisValue_0 < (*binding).releaseThreshold) as libc::c_int
        } else {
            (axisValue_0 > -(*binding).releaseThreshold) as libc::c_int
        } != 0
        {
            (*button).state |= State_Released;
            (*button).state &= !State_Down;
            InputBindings_RaiseCallback(
                b"Released\0" as *const u8 as *const libc::c_char,
                binding,
                (*button).onReleased,
            );
            let mut _i: i32 = 0 as libc::c_int;
            while _i < this.downBindings_size {
                let mut x: *mut DownBinding = &mut *(this.downBindings_data)
                    .offset(_i as isize) as *mut DownBinding;
                if ((*x).binding == binding && (*x).button == button) as libc::c_int
                    as libc::c_long != 0
                {
                    let mut _j: i32 = _i;
                    while _j < this.downBindings_size - 1 as libc::c_int {
                        *(this.downBindings_data)
                            .offset(
                                _j as isize,
                            ) = *(this.downBindings_data)
                            .offset((_j + 1 as libc::c_int) as isize);
                        _j += 1;
                    }
                    this.downBindings_size -= 1;
                    break;
                } else {
                    _i += 1;
                }
            }
        }
        iBtn += 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn InputBindings_Update() {
    let mut i: libc::c_int = 0 as libc::c_int;
    while i < this.downBindings_size {
        let mut down: DownBinding = *(this.downBindings_data).offset(i as isize);
        InputBindings_RaiseCallback(
            b"Down\0" as *const u8 as *const libc::c_char,
            down.binding,
            (*down.button).onDown,
        );
        i += 1;
    }
    let mut event: InputEvent =  InputEvent {
            timestamp: 0,
            device: Device { type_0: 0, id: 0 },
            button: 0,
            value: 0.,
            state: 0,
        };
    while Input_GetNextEvent(&mut event) {
        let mut binding: *mut InputBinding = (this.activeBindings_data)
            .offset(this.activeBindings_size as isize)
            .offset(-(1));
        let mut __iterbegin: *mut InputBinding = this.activeBindings_data;
        while binding >= __iterbegin {
            let mut iBtn: libc::c_int = 0 as libc::c_int;
            while iBtn
                < (::core::mem::size_of::<[[RawButton; 4]; 4]>())
                    .wrapping_div(
                        ::core::mem::size_of::<[RawButton; 4]>(),
                    ) as libc::c_int
            {
                let mut iBind: libc::c_int = 0 as libc::c_int;
                while iBind
                    < (::core::mem::size_of::<[RawButton; 4]>())
                        .wrapping_div(
                            ::core::mem::size_of::<RawButton>() as usize,
                        ) as libc::c_int
                {
                    let mut button: *mut RawButton = &mut *(*((*binding).rawButtons)
                        .as_mut_ptr()
                        .offset(iBtn as isize))
                        .as_mut_ptr()
                        .offset(iBind as isize) as *mut RawButton;
                    if event.button == (*button).button {
                        (*button).value = event.value;
                        InputBindings_UpdateBinding(binding);
                    }
                    iBind += 1;
                }
                iBtn += 1;
            }
            binding = binding.offset(-1);
        }
    }
}
static mut iXPos: libc::c_int = 0 as libc::c_int;
static mut iXNeg: libc::c_int = 1 as libc::c_int;
static mut iYPos: libc::c_int = 2 as libc::c_int;
static mut iYNeg: libc::c_int = 3 as libc::c_int;
static mut iX: libc::c_int = 0 as libc::c_int;
static mut iY: libc::c_int = 1 as libc::c_int;
#[inline]
unsafe extern "C" fn InputBinding_GetButtonState(
    mut binding: *mut InputBinding,
    mut iBtn: libc::c_int,
    mut state: State,
) -> bool {
    return (*binding).buttons[iBtn as usize].state & state == state;
}
#[no_mangle]
pub unsafe extern "C" fn InputBinding_GetPressed(
    mut binding: *mut InputBinding,
) -> bool {
    return InputBinding_GetButtonState(binding, iXPos, State_Pressed);
}
#[no_mangle]
pub unsafe extern "C" fn InputBinding_GetDown(mut binding: *mut InputBinding) -> bool {
    return InputBinding_GetButtonState(binding, iXPos, State_Down);
}
#[no_mangle]
pub unsafe extern "C" fn InputBinding_GetReleased(
    mut binding: *mut InputBinding,
) -> bool {
    return InputBinding_GetButtonState(binding, iXPos, State_Released);
}
#[no_mangle]
pub unsafe extern "C" fn InputBinding_GetValue(
    mut binding: *mut InputBinding,
) -> f32 {
    return (*binding).axes[iX as usize].value;
}
#[no_mangle]
pub unsafe extern "C" fn InputBinding_GetVecValue(
    mut binding: *mut InputBinding,
) -> Vec2 {
    return (*binding).axis2D.value;
}
#[no_mangle]
pub unsafe extern "C" fn InputBinding_GetXValue(
    mut binding: *mut InputBinding,
) -> f32 {
    return (*binding).axes[iX as usize].value;
}
#[no_mangle]
pub unsafe extern "C" fn InputBinding_GetYValue(
    mut binding: *mut InputBinding,
) -> f32 {
    return (*binding).axes[iY as usize].value;
}
#[no_mangle]
pub unsafe extern "C" fn InputBinding_GetXPosPressed(
    mut binding: *mut InputBinding,
) -> bool {
    return InputBinding_GetButtonState(binding, iXPos, State_Pressed);
}
#[no_mangle]
pub unsafe extern "C" fn InputBinding_GetXPosDown(
    mut binding: *mut InputBinding,
) -> bool {
    return InputBinding_GetButtonState(binding, iXPos, State_Down);
}
#[no_mangle]
pub unsafe extern "C" fn InputBinding_GetXPosReleased(
    mut binding: *mut InputBinding,
) -> bool {
    return InputBinding_GetButtonState(binding, iXPos, State_Released);
}
#[no_mangle]
pub unsafe extern "C" fn InputBinding_GetXNegPressed(
    mut binding: *mut InputBinding,
) -> bool {
    return InputBinding_GetButtonState(binding, iXNeg, State_Pressed);
}
#[no_mangle]
pub unsafe extern "C" fn InputBinding_GetXNegDown(
    mut binding: *mut InputBinding,
) -> bool {
    return InputBinding_GetButtonState(binding, iXNeg, State_Down);
}
#[no_mangle]
pub unsafe extern "C" fn InputBinding_GetXNegReleased(
    mut binding: *mut InputBinding,
) -> bool {
    return InputBinding_GetButtonState(binding, iXNeg, State_Released);
}
#[no_mangle]
pub unsafe extern "C" fn InputBinding_GetYPosPressed(
    mut binding: *mut InputBinding,
) -> bool {
    return InputBinding_GetButtonState(binding, iYPos, State_Pressed);
}
#[no_mangle]
pub unsafe extern "C" fn InputBinding_GetYPosDown(
    mut binding: *mut InputBinding,
) -> bool {
    return InputBinding_GetButtonState(binding, iYPos, State_Down);
}
#[no_mangle]
pub unsafe extern "C" fn InputBinding_GetYPosReleased(
    mut binding: *mut InputBinding,
) -> bool {
    return InputBinding_GetButtonState(binding, iYPos, State_Released);
}
#[no_mangle]
pub unsafe extern "C" fn InputBinding_GetYNegPressed(
    mut binding: *mut InputBinding,
) -> bool {
    return InputBinding_GetButtonState(binding, iYNeg, State_Pressed);
}
#[no_mangle]
pub unsafe extern "C" fn InputBinding_GetYNegDown(
    mut binding: *mut InputBinding,
) -> bool {
    return InputBinding_GetButtonState(binding, iYNeg, State_Down);
}
#[no_mangle]
pub unsafe extern "C" fn InputBinding_GetYNegReleased(
    mut binding: *mut InputBinding,
) -> bool {
    return InputBinding_GetButtonState(binding, iYNeg, State_Released);
}
#[no_mangle]
pub unsafe extern "C" fn InputBinding_SetDeadzone(
    mut binding: *mut InputBinding,
    mut deadzone: f32,
) -> *mut InputBinding {
    (*binding).deadzone = deadzone;
    return binding;
}
#[no_mangle]
pub unsafe extern "C" fn InputBinding_SetExponent(
    mut binding: *mut InputBinding,
    mut exponent: f32,
) -> *mut InputBinding {
    (*binding).exponent = exponent;
    return binding;
}
#[inline]
unsafe extern "C" fn InputBinding_SetInvert(
    mut binding: *mut InputBinding,
    mut iAxis: libc::c_int,
    mut invert: bool,
) {
    let mut axis: *mut AggregateAxis = &mut *((*binding).axes)
        .as_mut_ptr()
        .offset(iAxis as isize) as *mut AggregateAxis;
    if invert as libc::c_int != (*axis).invert as libc::c_int {
        (*axis).invert = invert;
        let mut iBind: libc::c_int = 0 as libc::c_int;
        while iBind < BindCount {
            let mut btnPos: *mut RawButton = &mut *(*((*binding).rawButtons)
                .as_mut_ptr()
                .offset((2 as libc::c_int * iAxis + 0 as libc::c_int) as isize))
                .as_mut_ptr()
                .offset(iBind as isize) as *mut RawButton;
            let mut btnNeg: *mut RawButton = &mut *(*((*binding).rawButtons)
                .as_mut_ptr()
                .offset((2 as libc::c_int * iAxis + 1 as libc::c_int) as isize))
                .as_mut_ptr()
                .offset(iBind as isize) as *mut RawButton;
            let mut temp: Button = (*btnPos).button;
            (*btnPos).button = (*btnNeg).button;
            (*btnNeg).button = temp;
            iBind += 1;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn InputBinding_SetInvertX(
    mut binding: *mut InputBinding,
    mut invert: bool,
) -> *mut InputBinding {
    InputBinding_SetInvert(binding, 0 as libc::c_int, invert);
    return binding;
}
#[no_mangle]
pub unsafe extern "C" fn InputBinding_SetInvertY(
    mut binding: *mut InputBinding,
    mut invert: bool,
) -> *mut InputBinding {
    InputBinding_SetInvert(binding, 1 as libc::c_int, invert);
    return binding;
}
#[no_mangle]
pub unsafe extern "C" fn InputBinding_SetRange(
    mut binding: *mut InputBinding,
    mut min: f32,
    mut max: f32,
) -> *mut InputBinding {
    (*binding).minValue = min;
    (*binding).maxValue = max;
    return binding;
}
#[no_mangle]
pub unsafe extern "C" fn InputBinding_SetThresholds(
    mut binding: *mut InputBinding,
    mut press: f32,
    mut release: f32,
) -> *mut InputBinding {
    (*binding).pressThreshold = press;
    (*binding).releaseThreshold = release;
    return binding;
}
