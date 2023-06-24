use std::ffi::CStr;

use tracing::info;

use super::*;
use crate::common::*;
use crate::lua::*;
use crate::math::Vec2;

const BindCount: usize = 4;

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

#[derive(Copy, Clone)]
#[repr(C)]
pub struct InputBinding {
    pub name: *const libc::c_char,
    pub rawButtons: [[RawButton; 4]; BindCount],

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
    //ArrayList(InputBinding, inactiveBindings);
    pub activeBindings: Vec<InputBinding>,
    pub downBindings: Vec<DownBinding>,
}

static mut this: InputBindings = InputBindings {
    activeBindings: Vec::new(),
    downBindings: Vec::new(),
};

#[no_mangle]
pub unsafe extern "C" fn InputBindings_Init() {
    //ArrayList_Reserve(self.inactiveBindings, 64);
    this.activeBindings.reserve(64);
    this.downBindings.reserve(8);
}

#[no_mangle]
pub unsafe extern "C" fn InputBindings_Free() {
    //ArrayList_Free(self.inactiveBindings);
    this.activeBindings.clear();
    this.downBindings.clear();
}

extern "C" fn InputBindings_RaiseCallback(
    event: *const libc::c_char,
    binding: *mut InputBinding,
    _callback: LuaRef,
) {
    unsafe {
        info!(
            "{:?} - {:?}",
            CStr::from_ptr(event),
            CStr::from_ptr((*binding).name)
        );
    }
    /* TODO : Decide what all we want to pass to the callbacks (values, states, ...?) */
    //if (callback) {
    //  Lua_PushRef(binding->luaInstance, callback);
    //  Lua_PushPtr(binding->luaInstance, binding);
    //  Lua_Call(binding->luaInstance, 1, 0, 0);
    //}
}

#[no_mangle]
pub unsafe extern "C" fn InputBindings_UpdateBinding(binding: *mut InputBinding) {
    let mut value = Vec2::ZERO;
    let axisValues: [*mut f32; 2] = [&mut value.x, &mut value.y];

    // Update Value
    for iAxis in 0..(*binding).axes.len() {
        let axisValue: *mut f32 = axisValues[iAxis];
        for iBind in 0..BindCount {
            *axisValue += (*binding).rawButtons[(2 * iAxis + 0) as usize][iBind].value;
            *axisValue -= (*binding).rawButtons[(2 * iAxis + 1) as usize][iBind].value;
        }
        *axisValue = (*axisValue - (*binding).deadzone) / (1.0f32 - (*binding).deadzone);
        *axisValue = f32::powf(*axisValue, (*binding).exponent);
        *axisValue = f32::clamp(*axisValue, (*binding).minValue, (*binding).maxValue);
    }
    let len: f32 = value.length();
    if len > 1.0f32 {
        value /= 1.0f32 / len;
    }

    // Axis2D
    let axis2D: *mut AggregateAxis2D = &mut (*binding).axis2D;
    if value != (*axis2D).value {
        (*axis2D).value = value;
        InputBindings_RaiseCallback(c_str!("Changed"), binding, (*axis2D).onChanged);
    }

    // Axes
    for iAxis in 0..(*binding).axes.len() {
        let axis: *mut AggregateAxis =
            &mut *((*binding).axes).as_mut_ptr().offset(iAxis as isize) as *mut AggregateAxis;
        if *axisValues[iAxis as usize] != (*axis).value {
            (*axis).value = *axisValues[iAxis as usize];
            InputBindings_RaiseCallback(
                if iAxis == 0 {
                    c_str!("Changed X")
                } else {
                    c_str!("Changed Y")
                },
                binding,
                (*axis).onChanged,
            );
        }
    }

    // Buttons
    for iBtn in 0..(*binding).buttons.len() {
        let button: &mut AggregateButton = &mut (*binding).buttons[iBtn];
        let axisValue: f32 = (*binding).axes[(iBtn / 2) as usize].value;
        let isPos: bool = iBtn & 1 == 0;

        if !((*button).state & State_Down == State_Down) {
            // Pressed
            if if isPos as i32 != 0 {
                axisValue > (*binding).pressThreshold
            } else {
                axisValue < -(*binding).pressThreshold
            } {
                (*button).state |= State_Pressed;
                (*button).state |= State_Down;
                InputBindings_RaiseCallback(c_str!("Pressed"), binding, (*button).onPressed);

                this.downBindings.push(DownBinding { binding, button });
            }
        } else if if isPos as i32 != 0 {
            // Released
            axisValue < (*binding).releaseThreshold
        } else {
            axisValue > -(*binding).releaseThreshold
        } {
            (*button).state |= State_Released;
            (*button).state &= !State_Down;
            InputBindings_RaiseCallback(c_str!("Released"), binding, (*button).onReleased);

            this.downBindings
                .retain(|down| down.binding != binding || down.button != button);
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn InputBindings_Update() {
    // Down
    for down in this.downBindings.iter() {
        InputBindings_RaiseCallback(c_str!("Down"), down.binding, (*down.button).onDown);
    }

    let mut event: InputEvent = InputEvent {
        timestamp: 0,
        device: Device { ty: 0, id: 0 },
        button: 0,
        value: 0.,
        state: 0,
    };
    while Input_GetNextEvent(&mut event) {
        // Match
        for binding in this.activeBindings.iter_mut().rev() {
            for iBtn in 0..(*binding).rawButtons.len() {
                for iBind in 0..(*binding).rawButtons[iBtn].len() {
                    let button: &mut RawButton = &mut (*binding).rawButtons[iBtn][iBind];

                    if event.button == (*button).button {
                        (*button).value = event.value;
                        InputBindings_UpdateBinding(binding);
                    }
                }
            }
        }
    }
}

// void InputBindings_Register (InputBinding* binding) {
//     //Lua* lua = Lua_GetActive();
//     //if (!lua)
//     //  panic("InputBinding_Register: No Lua instance is active");

//     InputBinding binding = {};
//     binding.states[Idx_xPos].button = binding->xPos;
//     binding.states[Idx_xNeg].button = binding->xNeg;
//     binding.states[Idx_yPos].button = binding->yPos;
//     binding.states[Idx_yNeg].button = binding->yNeg;
//     binding.pressThreshold          = binding->pressThreshold;
//     binding.releaseThreshold        = binding->releaseThreshold;
//     binding.exponent                = binding->exponent;

//     //registeredBinding.luaInstance = lua;
//     //registeredBinding.onPressed   = Lua_GetRef(lua);
//     //registeredBinding.onDown      = Lua_GetRef(lua);
//     //registeredBinding.onReleased  = Lua_GetRef(lua);
//     //registeredBinding.onChanged   = Lua_GetRef(lua);

//     ArrayList_Append(self.activeBindings, binding);
//   }

//   void InputBindings_Unregister (InputBinding* binding) {
//     ArrayList_ForEachIReverse(self.downBindings, i) {
//       RegisteredBinding* downBinding = ArrayList_Get(self.downBindings, i);
//       if (StrEqual(binding->name, downBinding->name)) {
//         ArrayList_RemoveAt(self.downBindings, i);
//       }
//     }

//     ArrayList_ForEachIReverse(self.activeBindings, i) {
//       RegisteredBinding* binding2 = ArrayList_GetPtr(self.activeBindings, i);
//       if (StrEqual(binding->name, binding2->name)) {
//         //Lua_ReleaseRef(binding2->luaInstance, binding2->onPressed);
//         //Lua_ReleaseRef(binding2->luaInstance, binding2->onDown);
//         //Lua_ReleaseRef(binding2->luaInstance, binding2->onReleased);
//         //Lua_ReleaseRef(binding2->luaInstance, binding2->onChanged);
//         ArrayList_RemoveAt(self.activeBindings, i);
//       }
//     }
//   }

//   const float InputBindings_DefaultPressThreshold   =  0.5f;
//   const float InputBindings_DefaultReleaseThreshold =  0.3f;
//   const float InputBindings_DefaultExponent         =  1.0f;
//   const float InputBindings_DefaultDeadzone         =  0.2f;
//   const float InputBindings_DefaultMinValue         = -FLT_MAX;
//   const float InputBindings_DefaultMaxValue         =  FLT_MAX;

//   void InputBindings_RegisterAll (InputBinding* binding, int count) {
//     for (int i = 0; i < count; i++)
//       InputBindings_Register(&binding[i]);
//   }

//   void InputBindings_UnregisterAll (InputBinding* binding, int count) {
//     for (int i = 0; i < count; i++)
//       InputBindings_Unregister(&binding[i]);
//   }

const iXPos: i32 = 0;
const iXNeg: i32 = 1;
const iYPos: i32 = 2;
const iYNeg: i32 = 3;
const iX: i32 = 0;
const iY: i32 = 1;

#[inline]
unsafe fn InputBinding_GetButtonState(binding: *mut InputBinding, iBtn: i32, state: State) -> bool {
    (*binding).buttons[iBtn as usize].state & state == state
}

#[no_mangle]
pub unsafe extern "C" fn InputBinding_GetPressed(binding: *mut InputBinding) -> bool {
    InputBinding_GetButtonState(binding, iXPos, State_Pressed)
}

#[no_mangle]
pub unsafe extern "C" fn InputBinding_GetDown(binding: *mut InputBinding) -> bool {
    InputBinding_GetButtonState(binding, iXPos, State_Down)
}

#[no_mangle]
pub unsafe extern "C" fn InputBinding_GetReleased(binding: *mut InputBinding) -> bool {
    InputBinding_GetButtonState(binding, iXPos, State_Released)
}

#[no_mangle]
pub unsafe extern "C" fn InputBinding_GetValue(binding: *mut InputBinding) -> f32 {
    (*binding).axes[iX as usize].value
}

#[no_mangle]
pub unsafe extern "C" fn InputBinding_GetVecValue(binding: *mut InputBinding) -> Vec2 {
    (*binding).axis2D.value
}

#[no_mangle]
pub unsafe extern "C" fn InputBinding_GetXValue(binding: *mut InputBinding) -> f32 {
    (*binding).axes[iX as usize].value
}

#[no_mangle]
pub unsafe extern "C" fn InputBinding_GetYValue(binding: *mut InputBinding) -> f32 {
    (*binding).axes[iY as usize].value
}

#[no_mangle]
pub unsafe extern "C" fn InputBinding_GetXPosPressed(binding: *mut InputBinding) -> bool {
    InputBinding_GetButtonState(binding, iXPos, State_Pressed)
}

#[no_mangle]
pub unsafe extern "C" fn InputBinding_GetXPosDown(binding: *mut InputBinding) -> bool {
    InputBinding_GetButtonState(binding, iXPos, State_Down)
}

#[no_mangle]
pub unsafe extern "C" fn InputBinding_GetXPosReleased(binding: *mut InputBinding) -> bool {
    InputBinding_GetButtonState(binding, iXPos, State_Released)
}

#[no_mangle]
pub unsafe extern "C" fn InputBinding_GetXNegPressed(binding: *mut InputBinding) -> bool {
    InputBinding_GetButtonState(binding, iXNeg, State_Pressed)
}

#[no_mangle]
pub unsafe extern "C" fn InputBinding_GetXNegDown(binding: *mut InputBinding) -> bool {
    InputBinding_GetButtonState(binding, iXNeg, State_Down)
}

#[no_mangle]
pub unsafe extern "C" fn InputBinding_GetXNegReleased(binding: *mut InputBinding) -> bool {
    InputBinding_GetButtonState(binding, iXNeg, State_Released)
}

#[no_mangle]
pub unsafe extern "C" fn InputBinding_GetYPosPressed(binding: *mut InputBinding) -> bool {
    InputBinding_GetButtonState(binding, iYPos, State_Pressed)
}

#[no_mangle]
pub unsafe extern "C" fn InputBinding_GetYPosDown(binding: *mut InputBinding) -> bool {
    InputBinding_GetButtonState(binding, iYPos, State_Down)
}

#[no_mangle]
pub unsafe extern "C" fn InputBinding_GetYPosReleased(binding: *mut InputBinding) -> bool {
    InputBinding_GetButtonState(binding, iYPos, State_Released)
}

#[no_mangle]
pub unsafe extern "C" fn InputBinding_GetYNegPressed(binding: *mut InputBinding) -> bool {
    InputBinding_GetButtonState(binding, iYNeg, State_Pressed)
}

#[no_mangle]
pub unsafe extern "C" fn InputBinding_GetYNegDown(binding: *mut InputBinding) -> bool {
    InputBinding_GetButtonState(binding, iYNeg, State_Down)
}

#[no_mangle]
pub unsafe extern "C" fn InputBinding_GetYNegReleased(binding: *mut InputBinding) -> bool {
    InputBinding_GetButtonState(binding, iYNeg, State_Released)
}

#[no_mangle]
pub unsafe extern "C" fn InputBinding_SetDeadzone(
    binding: *mut InputBinding,
    deadzone: f32,
) -> *mut InputBinding {
    (*binding).deadzone = deadzone;
    binding
}

#[no_mangle]
pub unsafe extern "C" fn InputBinding_SetExponent(
    binding: *mut InputBinding,
    exponent: f32,
) -> *mut InputBinding {
    (*binding).exponent = exponent;
    binding
}

#[inline]
unsafe fn InputBinding_SetInvert(binding: *mut InputBinding, iAxis: i32, invert: bool) {
    let axis: &mut AggregateAxis = &mut (*binding).axes[iAxis as usize];
    if invert != (*axis).invert {
        (*axis).invert = invert;

        for iBind in 0..BindCount {
            let btnPos: &mut RawButton =
                &mut (*binding).rawButtons[(2 * iAxis + 0) as usize][iBind];
            let btnNeg: &mut RawButton =
                &mut (*binding).rawButtons[(2 * iAxis + 1) as usize][iBind];

            let temp: Button = (*btnPos).button;
            (*btnPos).button = (*btnNeg).button;
            (*btnNeg).button = temp;
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn InputBinding_SetInvertX(
    binding: *mut InputBinding,
    invert: bool,
) -> *mut InputBinding {
    InputBinding_SetInvert(binding, 0, invert);
    binding
}

#[no_mangle]
pub unsafe extern "C" fn InputBinding_SetInvertY(
    binding: *mut InputBinding,
    invert: bool,
) -> *mut InputBinding {
    InputBinding_SetInvert(binding, 1, invert);
    binding
}

#[no_mangle]
pub unsafe extern "C" fn InputBinding_SetRange(
    binding: *mut InputBinding,
    min: f32,
    max: f32,
) -> *mut InputBinding {
    (*binding).minValue = min;
    (*binding).maxValue = max;
    binding
}

#[no_mangle]
pub unsafe extern "C" fn InputBinding_SetThresholds(
    binding: *mut InputBinding,
    press: f32,
    release: f32,
) -> *mut InputBinding {
    (*binding).pressThreshold = press;
    (*binding).releaseThreshold = release;
    binding
}

/* TODO : Probably easier to aggregate values to point to top level Vec2f
 *        instead of juggling indicies and shit. */
/* TODO : What happens if you change binding settings while down? */
