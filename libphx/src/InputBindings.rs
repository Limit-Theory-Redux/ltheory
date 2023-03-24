use crate::internal::Memory::*;
use crate::Button::*;
use crate::Common::*;
use crate::Device::*;
use crate::Input::*;
use crate::InputEvent::*;
use crate::Math::Vec2;
use crate::Math::Vec3;
use crate::State::*;
use libc;

extern "C" {
    pub type lua_State;
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct InputBinding {
    pub name: *const libc::c_char,
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
pub type lua_Integer = libc::ptrdiff_t;

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

#[derive(Copy, Clone)]
#[repr(C)]
pub struct DownBinding {
    pub binding: *mut InputBinding,
    pub button: *mut AggregateButton,
}

#[derive(Clone)]
#[repr(C)]
pub struct InputBindings {
    pub activeBindings: Vec<InputBinding>,
    pub downBindings: Vec<DownBinding>,
}

#[no_mangle]
pub static InputBindings_DefaultMaxValue: f32 = 0.;

#[no_mangle]
pub static InputBindings_DefaultMinValue: f32 = 0.;

#[no_mangle]
pub static InputBindings_DefaultDeadzone: f32 = 0.;

#[no_mangle]
pub static InputBindings_DefaultExponent: f32 = 0.;

#[no_mangle]
pub static InputBindings_DefaultReleaseThreshold: f32 = 0.;

#[no_mangle]
pub static InputBindings_DefaultPressThreshold: f32 = 0.;

static mut BindCount: i32 = 4;

static mut this: InputBindings = InputBindings {
    activeBindings: Vec::new(),
    downBindings: Vec::new(),
};

#[no_mangle]
pub unsafe extern "C" fn InputBindings_Init() {
    this.activeBindings.reserve(64);
    this.downBindings.reserve(8);
}

#[no_mangle]
pub unsafe extern "C" fn InputBindings_Free() {
    this.activeBindings.clear();
    this.downBindings.clear();
}

unsafe extern "C" fn InputBindings_RaiseCallback(
    mut event: *const libc::c_char,
    mut binding: *mut InputBinding,
    mut _callback: LuaRef,
) {
    libc::printf(
        b"%s - %s\n\0" as *const u8 as *const libc::c_char,
        event,
        (*binding).name,
    );
}

#[no_mangle]
pub unsafe extern "C" fn InputBindings_UpdateBinding(mut binding: *mut InputBinding) {
    let mut value = Vec2::ZERO;
    let mut axisValues: [*mut f32; 2] = [&mut value.x, &mut value.y];
    let mut iAxis = 0;
    while iAxis < (*binding).axes.len() {
        let mut axisValue: *mut f32 = axisValues[iAxis as usize];
        let mut iBind: i32 = 0;
        while iBind < BindCount {
            *axisValue += (*binding).rawButtons[(2 * iAxis + 0) as usize][iBind as usize].value;
            *axisValue -= (*binding).rawButtons[(2 * iAxis + 1) as usize][iBind as usize].value;
            iBind += 1;
        }
        *axisValue = (*axisValue - (*binding).deadzone) / (1.0f32 - (*binding).deadzone);
        *axisValue = f64::powf(*axisValue as f64, (*binding).exponent as f64) as f32;
        *axisValue = f64::clamp(
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
    let mut iAxis_0 = 0;
    while iAxis_0 < (*binding).axes.len() {
        let mut axis: *mut AggregateAxis =
            &mut *((*binding).axes).as_mut_ptr().offset(iAxis_0 as isize) as *mut AggregateAxis;
        if *axisValues[iAxis_0 as usize] != (*axis).value {
            (*axis).value = *axisValues[iAxis_0 as usize];
            InputBindings_RaiseCallback(
                if iAxis_0 == 0 {
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
    let mut iBtn = 0;
    while iBtn < (*binding).buttons.len() {
        let mut button: *mut AggregateButton =
            &mut *((*binding).buttons).as_mut_ptr().offset(iBtn as isize) as *mut AggregateButton;
        let mut axisValue_0: f32 = (*binding).axes[(iBtn / 2) as usize].value;
        let mut isPos: bool = iBtn & 1 == 0;
        if !((*button).state & State_Down == State_Down) {
            if if isPos as i32 != 0 {
                (axisValue_0 > (*binding).pressThreshold) as i32
            } else {
                (axisValue_0 < -(*binding).pressThreshold) as i32
            } != 0
            {
                (*button).state |= State_Pressed;
                (*button).state |= State_Down;
                InputBindings_RaiseCallback(
                    b"Pressed\0" as *const u8 as *const libc::c_char,
                    binding,
                    (*button).onPressed,
                );
                this.downBindings.push(DownBinding { binding: binding, button: button });
            }
        } else if if isPos as i32 != 0 {
            (axisValue_0 < (*binding).releaseThreshold) as i32
        } else {
            (axisValue_0 > -(*binding).releaseThreshold) as i32
        } != 0
        {
            (*button).state |= State_Released;
            (*button).state &= !State_Down;
            InputBindings_RaiseCallback(
                b"Released\0" as *const u8 as *const libc::c_char,
                binding,
                (*button).onReleased,
            );

            this.downBindings.retain(|down| down.binding != binding || down.button != button);
        }
        iBtn += 1;
    }
}

#[no_mangle]
pub unsafe extern "C" fn InputBindings_Update() {
    // Down
    for down in this.downBindings.iter() {
        InputBindings_RaiseCallback(
            b"Down\0" as *const u8 as *const libc::c_char,
            down.binding,
            (*down.button).onDown,
        );
    }
    let mut event: InputEvent = InputEvent {
        timestamp: 0,
        device: Device { type_0: 0, id: 0 },
        button: 0,
        value: 0.,
        state: 0,
    };
    while Input_GetNextEvent(&mut event) {
        // Match
        for binding in this.activeBindings.iter_mut().rev() {
            let mut iBtn = 0;
            while iBtn < (*binding).rawButtons.len() {
                let mut iBind = 0;
                while iBind < (*binding).rawButtons[iBtn].len() {
                    let mut button: *mut RawButton =
                        &mut *(*((*binding).rawButtons).as_mut_ptr().offset(iBtn as isize))
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
        }
    }
}
static mut iXPos: i32 = 0;

static mut iXNeg: i32 = 1;

static mut iYPos: i32 = 2;

static mut iYNeg: i32 = 3;

static mut iX: i32 = 0;

static mut iY: i32 = 1;

#[inline]
unsafe extern "C" fn InputBinding_GetButtonState(
    mut binding: *mut InputBinding,
    mut iBtn: i32,
    mut state: State,
) -> bool {
    (*binding).buttons[iBtn as usize].state & state == state
}

#[no_mangle]
pub unsafe extern "C" fn InputBinding_GetPressed(mut binding: *mut InputBinding) -> bool {
    InputBinding_GetButtonState(binding, iXPos, State_Pressed)
}

#[no_mangle]
pub unsafe extern "C" fn InputBinding_GetDown(mut binding: *mut InputBinding) -> bool {
    InputBinding_GetButtonState(binding, iXPos, State_Down)
}

#[no_mangle]
pub unsafe extern "C" fn InputBinding_GetReleased(mut binding: *mut InputBinding) -> bool {
    InputBinding_GetButtonState(binding, iXPos, State_Released)
}

#[no_mangle]
pub unsafe extern "C" fn InputBinding_GetValue(mut binding: *mut InputBinding) -> f32 {
    (*binding).axes[iX as usize].value
}

#[no_mangle]
pub unsafe extern "C" fn InputBinding_GetVecValue(mut binding: *mut InputBinding) -> Vec2 {
    (*binding).axis2D.value
}

#[no_mangle]
pub unsafe extern "C" fn InputBinding_GetXValue(mut binding: *mut InputBinding) -> f32 {
    (*binding).axes[iX as usize].value
}

#[no_mangle]
pub unsafe extern "C" fn InputBinding_GetYValue(mut binding: *mut InputBinding) -> f32 {
    (*binding).axes[iY as usize].value
}

#[no_mangle]
pub unsafe extern "C" fn InputBinding_GetXPosPressed(mut binding: *mut InputBinding) -> bool {
    InputBinding_GetButtonState(binding, iXPos, State_Pressed)
}

#[no_mangle]
pub unsafe extern "C" fn InputBinding_GetXPosDown(mut binding: *mut InputBinding) -> bool {
    InputBinding_GetButtonState(binding, iXPos, State_Down)
}

#[no_mangle]
pub unsafe extern "C" fn InputBinding_GetXPosReleased(mut binding: *mut InputBinding) -> bool {
    InputBinding_GetButtonState(binding, iXPos, State_Released)
}

#[no_mangle]
pub unsafe extern "C" fn InputBinding_GetXNegPressed(mut binding: *mut InputBinding) -> bool {
    InputBinding_GetButtonState(binding, iXNeg, State_Pressed)
}

#[no_mangle]
pub unsafe extern "C" fn InputBinding_GetXNegDown(mut binding: *mut InputBinding) -> bool {
    InputBinding_GetButtonState(binding, iXNeg, State_Down)
}

#[no_mangle]
pub unsafe extern "C" fn InputBinding_GetXNegReleased(mut binding: *mut InputBinding) -> bool {
    InputBinding_GetButtonState(binding, iXNeg, State_Released)
}

#[no_mangle]
pub unsafe extern "C" fn InputBinding_GetYPosPressed(mut binding: *mut InputBinding) -> bool {
    InputBinding_GetButtonState(binding, iYPos, State_Pressed)
}

#[no_mangle]
pub unsafe extern "C" fn InputBinding_GetYPosDown(mut binding: *mut InputBinding) -> bool {
    InputBinding_GetButtonState(binding, iYPos, State_Down)
}

#[no_mangle]
pub unsafe extern "C" fn InputBinding_GetYPosReleased(mut binding: *mut InputBinding) -> bool {
    InputBinding_GetButtonState(binding, iYPos, State_Released)
}

#[no_mangle]
pub unsafe extern "C" fn InputBinding_GetYNegPressed(mut binding: *mut InputBinding) -> bool {
    InputBinding_GetButtonState(binding, iYNeg, State_Pressed)
}

#[no_mangle]
pub unsafe extern "C" fn InputBinding_GetYNegDown(mut binding: *mut InputBinding) -> bool {
    InputBinding_GetButtonState(binding, iYNeg, State_Down)
}

#[no_mangle]
pub unsafe extern "C" fn InputBinding_GetYNegReleased(mut binding: *mut InputBinding) -> bool {
    InputBinding_GetButtonState(binding, iYNeg, State_Released)
}

#[no_mangle]
pub unsafe extern "C" fn InputBinding_SetDeadzone(
    mut binding: *mut InputBinding,
    mut deadzone: f32,
) -> *mut InputBinding {
    (*binding).deadzone = deadzone;
    binding
}

#[no_mangle]
pub unsafe extern "C" fn InputBinding_SetExponent(
    mut binding: *mut InputBinding,
    mut exponent: f32,
) -> *mut InputBinding {
    (*binding).exponent = exponent;
    binding
}

#[inline]
unsafe extern "C" fn InputBinding_SetInvert(
    mut binding: *mut InputBinding,
    mut iAxis: i32,
    mut invert: bool,
) {
    let mut axis: *mut AggregateAxis =
        &mut *((*binding).axes).as_mut_ptr().offset(iAxis as isize) as *mut AggregateAxis;
    if invert as i32 != (*axis).invert as i32 {
        (*axis).invert = invert;
        let mut iBind: i32 = 0;
        while iBind < BindCount {
            let mut btnPos: *mut RawButton = &mut *(*((*binding).rawButtons)
                .as_mut_ptr()
                .offset((2 * iAxis + 0) as isize))
            .as_mut_ptr()
            .offset(iBind as isize) as *mut RawButton;
            let mut btnNeg: *mut RawButton = &mut *(*((*binding).rawButtons)
                .as_mut_ptr()
                .offset((2 * iAxis + 1) as isize))
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
    InputBinding_SetInvert(binding, 0, invert);
    binding
}

#[no_mangle]
pub unsafe extern "C" fn InputBinding_SetInvertY(
    mut binding: *mut InputBinding,
    mut invert: bool,
) -> *mut InputBinding {
    InputBinding_SetInvert(binding, 1, invert);
    binding
}

#[no_mangle]
pub unsafe extern "C" fn InputBinding_SetRange(
    mut binding: *mut InputBinding,
    mut min: f32,
    mut max: f32,
) -> *mut InputBinding {
    (*binding).minValue = min;
    (*binding).maxValue = max;
    binding
}

#[no_mangle]
pub unsafe extern "C" fn InputBinding_SetThresholds(
    mut binding: *mut InputBinding,
    mut press: f32,
    mut release: f32,
) -> *mut InputBinding {
    (*binding).pressThreshold = press;
    (*binding).releaseThreshold = release;
    binding
}
