use super::*;
use crate::common::*;
use crate::internal::*;
use crate::logging::warn;
use crate::math::IVec2;
use crate::*;

use crate::system::*;

use sdl2_sys::*;

const Threshold_Pressed: f32 = 0.5f32;
const Threshold_Released: f32 = 0.4f32;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct DeviceState {
    pub transitions: [i32; SDL_Scancode::SDL_NUM_SCANCODES as usize],
    pub buttons: [bool; SDL_Scancode::SDL_NUM_SCANCODES as usize],
    pub axes: [f32; SDL_Scancode::SDL_NUM_SCANCODES as usize],
    pub lastEventTimestamp: u32,
    pub isConnected: bool,
}

#[derive(Clone, Default)]
#[repr(C)]
pub struct DeviceList {
    pub devices: Vec<DeviceState>,
}

#[derive(Clone)]
#[repr(C)]
pub struct Input {
    pub activeDevice: Device,
    pub lastTimestamp: u32,
    pub lastEventTimestamp: u32,
    pub lastMousePosition: IVec2,
    pub autoHideMouse: bool,
    pub deviceLists: [DeviceList; 4],
    pub events: Vec<InputEvent>,
    pub downButtons: Vec<InputEvent>,
    pub autoRelease: Vec<InputEvent>,
    pub injectedEvents: Vec<InputEvent>,
}

static mut this: Input = Input {
    activeDevice: Device { ty: 0, id: 0 },
    lastTimestamp: 0,
    lastEventTimestamp: 0,
    lastMousePosition: IVec2::ZERO,
    autoHideMouse: false,
    deviceLists: [
        DeviceList {
            devices: Vec::new(),
        },
        DeviceList {
            devices: Vec::new(),
        },
        DeviceList {
            devices: Vec::new(),
        },
        DeviceList {
            devices: Vec::new(),
        },
    ],
    events: Vec::new(),
    downButtons: Vec::new(),
    autoRelease: Vec::new(),
    injectedEvents: Vec::new(),
};

#[inline]
unsafe extern "C" fn Input_EnsureDeviceState(device: Device) -> *mut DeviceState {
    let deviceList: *mut DeviceList =
        &mut *(this.deviceLists).as_mut_ptr().offset(device.ty as isize) as *mut DeviceList;
    while (*deviceList).devices.len() as u32 <= device.id {
        (*deviceList).devices.push(DeviceState {
            transitions: [0; 512],
            buttons: [false; 512],
            axes: [0.; 512],
            lastEventTimestamp: 0,
            isConnected: false,
        });
    }
    &mut (*deviceList).devices[device.id as usize]
}

#[inline]
unsafe extern "C" fn Input_GetDeviceState(device: Device) -> *mut DeviceState {
    let deviceList: *mut DeviceList =
        &mut *(this.deviceLists).as_mut_ptr().offset(device.ty as isize) as *mut DeviceList;
    &mut (*deviceList).devices[device.id as usize]
}

#[inline]
unsafe extern "C" fn Input_SetActiveDevice(device: Device) {
    this.activeDevice = device;
    if this.autoHideMouse {
        SDL_ShowCursor(if device.ty == DeviceType_Mouse { 1 } else { 0 });
    }
}

#[inline]
unsafe extern "C" fn Input_GetDeviceExists(device: Device) -> bool {
    let deviceList: *mut DeviceList =
        &mut *(this.deviceLists).as_mut_ptr().offset(device.ty as isize) as *mut DeviceList;
    if device.id < (*deviceList).devices.len() as u32 {
        let deviceState: *mut DeviceState = &mut (*deviceList).devices[device.id as usize];
        return (*deviceState).isConnected;
    }
    false
}

#[inline]
unsafe extern "C" fn Input_GetDevicePressedImpl(device: Device, button: Button) -> bool {
    if button < 0 || button > SDL_Scancode::SDL_NUM_SCANCODES as i32 {
        return false;
    }
    let deviceState: *mut DeviceState = Input_GetDeviceState(device);
    if (*deviceState).buttons[button as usize] {
        (*deviceState).transitions[button as usize] > 0
    } else {
        (*deviceState).transitions[button as usize] > 1
    }
}

#[inline]
unsafe extern "C" fn Input_GetDeviceDownImpl(device: Device, button: Button) -> bool {
    let deviceState: *mut DeviceState = Input_GetDeviceState(device);
    (*deviceState).buttons[button as usize] || (*deviceState).transitions[button as usize] > 0
}

#[inline]
unsafe extern "C" fn Input_GetDeviceReleasedImpl(device: Device, button: Button) -> bool {
    let deviceState: *mut DeviceState = Input_GetDeviceState(device);
    if (*deviceState).buttons[button as usize] {
        (*deviceState).transitions[button as usize] > 1
    } else {
        (*deviceState).transitions[button as usize] > 0
    }
}

#[inline]
unsafe extern "C" fn Input_GetDeviceValueImpl(device: Device, button: Button) -> f32 {
    let deviceState: *mut DeviceState = Input_GetDeviceState(device);
    (*deviceState).axes[button as usize]
}

#[inline]
unsafe extern "C" fn Input_GetDeviceIdleTimeImpl(device: Device) -> f32 {
    let deviceState: *mut DeviceState = Input_GetDeviceState(device);
    (this.lastTimestamp).wrapping_sub((*deviceState).lastEventTimestamp) as f32 / 1000.0f32
}

#[inline]
unsafe extern "C" fn Input_DetermineButtonState(event: InputEvent) -> State {
    let mut buttonState: State = State_Null;

    let deviceState: *mut DeviceState = Input_GetDeviceState(event.device);
    let down: bool = (*deviceState).buttons[event.button as usize];
    if !down && event.value > Threshold_Pressed {
        buttonState |= State_Pressed | State_Down;
    }

    if down && event.value < Threshold_Released {
        buttonState |= State_Released;
    }

    buttonState
}

#[inline]
unsafe extern "C" fn Input_AppendEvent(event: InputEvent) {
    this.lastTimestamp = event.timestamp;
    this.lastEventTimestamp = event.timestamp;
    this.events.push(event);
}

#[inline]
unsafe extern "C" fn Input_InjectEvent(event: InputEvent) {
    this.lastTimestamp = event.timestamp;
    this.lastEventTimestamp = event.timestamp;
    this.injectedEvents.push(event);
}

#[inline]
unsafe extern "C" fn Input_SetButton(event: InputEvent) {
    //   Assert(event.button != 0);

    let deviceState: *mut DeviceState = Input_GetDeviceState(event.device);
    (*deviceState).axes[event.button as usize] = event.value;

    let down: bool = (*deviceState).buttons[event.button as usize];
    if !down && event.state & State_Pressed == State_Pressed {
        (*deviceState).transitions[event.button as usize] += 1;
        (*deviceState).buttons[event.button as usize] = true;

        this.downButtons.push(event);

        if event.device.ty != DeviceType_Null {
            Input_SetActiveDevice(event.device);
        }
    }

    if down && event.state & State_Released == State_Released {
        (*deviceState).transitions[event.button as usize] += 1;
        (*deviceState).buttons[event.button as usize] = false;

        let mut buttonsToRemove: Vec<usize> = Vec::new();
        for (i, down) in this.downButtons.iter().rev().enumerate() {
            if down.button == event.button {
                buttonsToRemove.push(i);
            }
        }
        for i in buttonsToRemove.iter().rev() {
            this.downButtons.remove(*i);
        }
    }

    if Button_IsAutoRelease(event.button) {
        this.autoRelease.push(event);
    }
}

#[no_mangle]
pub unsafe extern "C" fn Input_Init() {
    /* NOTE : This is a workaround for what appears to be a bug in SDL. Without
     *        this the first click after a window loses and regains focus will not
     *        send events. This still leaves us with not-completely-consistent
     *        behavior. Since the mouse state doesn't get screwed up when using
     *        the keyboard, the first click after alt tabbing will send mouse
     *        down, but only the second click after clicking off the window will
     *        send mouse down. We could further work around this by only only
     *        setting the hint when clicking off the window, but eh, more
     *        complexity for a much less irritating quirk. This bug has been
     *        reported.
     *        https://bugzilla.libsdl.org/show_bug.cgi?id=4165 */
    let result: SDL_bool = SDL_SetHint(c_str!("SDL_MOUSE_FOCUS_CLICKTHROUGH"), c_str!("1"));
    if result != SDL_bool::SDL_TRUE {
        warn!("Input_Init: SDL_SetHint failed");
    }

    for iDev in 0..DeviceType_COUNT as i32 {
        let device: Device = Device { ty: iDev, id: 0 };
        let deviceState: *mut DeviceState = Input_EnsureDeviceState(device);
        (*deviceState).isConnected = iDev != DeviceType_Gamepad;
    }

    this.events.reserve(16);
    this.downButtons.reserve(16);
    this.autoRelease.reserve(16);
    this.injectedEvents.reserve(16);

    let device: Device = Device {
        ty: DeviceType_Mouse,
        id: 0,
    };
    Input_SetActiveDevice(device);
}

#[no_mangle]
pub unsafe extern "C" fn Input_Free() {
    for device in this.deviceLists.iter_mut() {
        device.devices.clear();
    }

    this.events.clear();
    this.downButtons.clear();
    this.autoRelease.clear();
    this.injectedEvents.clear();
}

#[no_mangle]
pub unsafe extern "C" fn Input_Update() {
    Profiler_Begin(c_str!("Input_Update"));

    /* Update Stale Data */
    this.lastTimestamp = SDL_GetTicks();
    this.lastMousePosition.x = Input_GetValue(Button_Mouse_X) as i32;
    this.lastMousePosition.y = Input_GetValue(Button_Mouse_Y) as i32;

    for iDev in 0..DeviceType_COUNT {
        for deviceState in this.deviceLists[iDev].devices.iter_mut() {
            MemSet(
                ((*deviceState).transitions).as_mut_ptr() as *mut _,
                0,
                std::mem::size_of::<[i32; 512]>(),
            );
        }
    }

    this.events.clear();
    for event in this.injectedEvents.iter() {
        Input_AppendEvent(*event);
    }
    this.injectedEvents.clear();

    /* Process Down Buttons */
    for down in this.autoRelease.iter_mut() {
        let deviceState: *mut DeviceState = Input_GetDeviceState((*down).device);
        if (*deviceState).axes[(*down).button as usize] != 0.0f32 {
            (*down).value = 0.0f32;
            (*down).state = State_Changed | Input_DetermineButtonState(*down);
            (*down).timestamp = SDL_GetTicks();
            Input_SetButton(*down);
            Input_AppendEvent(*down);
        }
    }

    for down in this.downButtons.iter_mut() {
        let deviceState: *mut DeviceState = Input_GetDeviceState((*down).device);
        (*down).value = (*deviceState).axes[(*down).button as usize];
        (*down).state = State_Down;
        (*down).timestamp = SDL_GetTicks();
        Input_AppendEvent(*down);
    }

    /* Process New Input */
    let mut sdl: SDL_Event = SDL_Event { type_: 0 };
    while SDL_PollEvent(&mut sdl) != 0 {
        let mut event: InputEvent = InputEvent {
            timestamp: 0,
            device: Device { ty: 0, id: 0 },
            button: 0,
            value: 0.,
            state: 0,
        };
        event.timestamp = sdl.common.timestamp;

        match std::mem::transmute::<u32, SDL_EventType>(sdl.type_) {
            /* DeviceType_Keyboard */
            SDL_EventType::SDL_KEYDOWN => {
                if sdl.key.repeat != 0 {
                    continue;
                }
                let device: Device = Device {
                    ty: DeviceType_Keyboard,
                    id: 0,
                };

                event.device = device;
                event.button = Button_FromSDLScancode(sdl.key.keysym.scancode);
                event.value = 1.0f32;
                event.state = State_Changed | State_Pressed | State_Down;
                if event.button == Button_Null {
                    continue;
                }
                Input_SetButton(event);
                Input_AppendEvent(event);
            }
            SDL_EventType::SDL_KEYUP => {
                let device: Device = Device {
                    ty: DeviceType_Keyboard,
                    id: 0,
                };

                event.device = device;
                event.button = Button_FromSDLScancode(sdl.key.keysym.scancode);
                event.value = 0.0f32;
                event.state = State_Changed | State_Released;
                if event.button == Button_Null {
                    continue;
                }
                Input_SetButton(event);
                Input_AppendEvent(event);
            }
            /* DeviceType_Mouse */
            SDL_EventType::SDL_MOUSEBUTTONDOWN => {
                let device: Device = Device {
                    ty: DeviceType_Mouse,
                    id: sdl.button.which,
                };

                event.device = device;
                event.button = Button_FromSDLMouseButton(sdl.button.button);
                event.value = 1.0f32;
                event.state = State_Changed | State_Pressed | State_Down;
                Input_EnsureDeviceState(event.device);
                Input_SetButton(event);
                Input_AppendEvent(event);
            }
            SDL_EventType::SDL_MOUSEBUTTONUP => {
                let device: Device = Device {
                    ty: DeviceType_Mouse,
                    id: sdl.button.which,
                };

                event.device = device;
                event.button = Button_FromSDLMouseButton(sdl.button.button);
                event.value = 0.0f32;
                event.state = State_Changed | State_Released;
                Input_EnsureDeviceState(event.device);
                Input_SetButton(event);
                Input_AppendEvent(event);
            }
            SDL_EventType::SDL_MOUSEMOTION => {
                /* NOTE : Mouse motion never causes pressed/down/released events. */
                let device: Device = Device {
                    ty: DeviceType_Mouse,
                    id: sdl.motion.which,
                };
                let deviceState: *mut DeviceState = Input_EnsureDeviceState(device);

                event.device = device;
                event.button = Button_Mouse_X;
                event.value = sdl.motion.x as f32;
                event.state = State_Changed;
                if event.value != (*deviceState).axes[event.button as usize] {
                    (*deviceState).axes[event.button as usize] = event.value;
                    Input_SetActiveDevice(event.device);
                    Input_AppendEvent(event);
                }

                event.device = device;
                event.button = Button_Mouse_Y;
                event.value = sdl.motion.y as f32;
                event.state = State_Changed;
                if event.value != (*deviceState).axes[event.button as usize] {
                    (*deviceState).axes[event.button as usize] = event.value;
                    Input_SetActiveDevice(event.device);
                    Input_AppendEvent(event);
                }
            }
            SDL_EventType::SDL_MOUSEWHEEL => {
                let device: Device = Device {
                    ty: DeviceType_Mouse,
                    id: sdl.wheel.which,
                };
                Input_EnsureDeviceState(device);

                event.device = device;
                event.button = Button_Mouse_ScrollX;
                event.value = sdl.wheel.x as f32;
                event.state = State_Changed | Input_DetermineButtonState(event);
                if event.value != Input_GetDeviceValueImpl(event.device, event.button) {
                    Input_SetButton(event);
                    Input_AppendEvent(event);
                }

                event.device = device;
                event.button = Button_Mouse_ScrollY;
                event.value = sdl.wheel.y as f32;
                event.state = State_Changed | Input_DetermineButtonState(event);
                if event.value != Input_GetDeviceValueImpl(event.device, event.button) {
                    Input_SetButton(event);
                    Input_AppendEvent(event);
                }
            }
            /* DeviceType_Gamepad */
            SDL_EventType::SDL_CONTROLLERBUTTONDOWN => {
                let device: Device = Device {
                    ty: DeviceType_Gamepad,
                    id: sdl.cbutton.which as u32,
                };

                event.device = device;
                event.button =
                    Button_FromSDLControllerButton(std::mem::transmute(sdl.cbutton.button as i32));
                event.value = 1.0f32;
                event.state = State_Changed | State_Pressed | State_Down;
                Input_SetButton(event);
                Input_AppendEvent(event);
            }
            SDL_EventType::SDL_CONTROLLERBUTTONUP => {
                let device: Device = Device {
                    ty: DeviceType_Gamepad,
                    id: sdl.cbutton.which as u32,
                };

                event.device = device;
                event.button =
                    Button_FromSDLControllerButton(std::mem::transmute(sdl.cbutton.button as i32));
                event.value = 0.0f32;
                event.state = State_Changed | State_Released;
                Input_SetButton(event);
                Input_AppendEvent(event);
            }
            SDL_EventType::SDL_CONTROLLERAXISMOTION => {
                let device: Device = Device {
                    ty: DeviceType_Gamepad,
                    id: sdl.caxis.which as u32,
                };
                let mut value: f32 =
                    f32::clamp(sdl.caxis.value as f32 / 32767.0f32, -1.0f32, 1.0f32);
                let axis: SDL_GameControllerAxis = std::mem::transmute(sdl.caxis.axis as i32);
                if axis == SDL_GameControllerAxis::SDL_CONTROLLER_AXIS_LEFTY
                    || axis == SDL_GameControllerAxis::SDL_CONTROLLER_AXIS_RIGHTY
                {
                    value = -value;
                }

                event.device = device;
                event.button = Button_FromSDLControllerAxis(axis);
                event.value = value;
                event.state = State_Changed | Input_DetermineButtonState(event);
                Input_SetButton(event);
                Input_AppendEvent(event);
            }
            SDL_EventType::SDL_CONTROLLERDEVICEADDED => {
                if SDL_IsGameController(sdl.cdevice.which) == SDL_bool::SDL_TRUE {
                    let sdlController: *mut SDL_GameController =
                        SDL_GameControllerOpen(sdl.cdevice.which);
                    if sdlController.is_null() {
                        warn!("Input_Update: SDL_GameControllerOpen failed");
                    } else {
                        let sdlJoystick: *mut SDL_Joystick =
                            SDL_GameControllerGetJoystick(sdlController);
                        let id: u32 = SDL_JoystickInstanceID(sdlJoystick) as u32;
                        let device: Device = Device {
                            ty: DeviceType_Gamepad,
                            id,
                        };
                        let deviceState: *mut DeviceState = Input_EnsureDeviceState(device);
                        (*deviceState).isConnected = true;
                    }
                }
            }
            SDL_EventType::SDL_CONTROLLERDEVICEREMOVED => {
                /* NOTE : SDL already sends events to zero out all game controller
                 *        input so there's no need to do it manually. */

                let device: Device = Device {
                    ty: DeviceType_Gamepad,
                    id: sdl.cdevice.which as u32,
                };
                let deviceState: *mut DeviceState = Input_GetDeviceState(device);
                (*deviceState).isConnected = false;

                let sdlController: *mut SDL_GameController =
                    SDL_GameControllerFromInstanceID(sdl.cdevice.which);
                if !sdlController.is_null() {
                    SDL_GameControllerClose(sdlController);
                }
            }
            /* TODO : Maybe we should release all input then re-set it? */
            SDL_EventType::SDL_CONTROLLERDEVICEREMAPPED => {
                let sdlController: *mut SDL_GameController =
                    SDL_GameControllerFromInstanceID(sdl.cdevice.which);
                let device: Device = Device {
                    ty: DeviceType_Gamepad,
                    id: sdl.cdevice.which as u32,
                };
                let deviceState: *mut DeviceState = Input_GetDeviceState(device);

                for iBtn in Button_Gamepad_Button_First..=Button_Gamepad_Button_Last {
                    let value: f32 = SDL_GameControllerGetButton(
                        sdlController,
                        Button_ToSDLControllerButton(iBtn),
                    ) as f32;
                    if value != (*deviceState).axes[iBtn as usize] {
                        event.device = device;
                        event.button = iBtn;
                        event.value = value;
                        event.state = State_Changed | Input_DetermineButtonState(event);
                        Input_SetButton(event);
                        Input_AppendEvent(event);
                    }
                }

                for iAxis in Button_Gamepad_Axis_First..=Button_Gamepad_Axis_Last {
                    let mut value: f32 =
                        SDL_GameControllerGetAxis(sdlController, Button_ToSDLControllerAxis(iAxis))
                            as f32;
                    value = f32::clamp(value / 32767.0f32, -1.0f32, 1.0f32);
                    if iAxis == Button_Gamepad_LStickY || iAxis == Button_Gamepad_RStickY {
                        value = -value;
                    }

                    if value != (*deviceState).axes[iAxis as usize] {
                        event.device = device;
                        event.button = iAxis;
                        event.value = value;
                        event.state = State_Changed | Input_DetermineButtonState(event);
                        Input_SetButton(event);
                        Input_AppendEvent(event);
                    }
                }
            }
            /* DeviceType_Null */
            SDL_EventType::SDL_QUIT => {
                let device: Device = Device {
                    ty: DeviceType_Null,
                    id: 0,
                };

                event.device = device;
                event.button = Button_System_Exit;
                event.value = 0.0f32;
                event.state = State_Changed | State_Pressed | State_Down;
                Input_SetButton(event);
                Input_AppendEvent(event);
            }
            SDL_EventType::SDL_WINDOWEVENT => {
                if sdl.window.event == SDL_WindowEventID::SDL_WINDOWEVENT_ENTER as u8 {
                    let device = Device {
                        ty: DeviceType_Null,
                        id: 0,
                    };

                    event.device = device;
                    event.button = Button_System_Win_Enter;
                    event.value = 0.0f32;
                    event.state = State_Changed | State_Pressed | State_Down;
                    Input_SetButton(event);
                    Input_AppendEvent(event);
                }

                if sdl.window.event == SDL_WindowEventID::SDL_WINDOWEVENT_FOCUS_LOST as u8 {
                    let device = Device {
                        ty: DeviceType_Null,
                        id: 0,
                    };

                    event.device = device;
                    event.button = Button_System_Win_Leave;
                    event.value = 0.0f32;
                    event.state = State_Changed | State_Pressed | State_Down;
                    Input_SetButton(event);
                    Input_AppendEvent(event);

                    /* TODO : Test button release on focus loss */

                    /* OPTIMIZE : Do this without incurring the cost of the search and
                     *            removes in SetButton */
                    for down in this.downButtons.iter_mut() {
                        down.timestamp = sdl.common.timestamp;
                        down.value = 0.0f32;
                        down.state = State_Changed | Input_DetermineButtonState(event);
                        Input_SetButton(*down);
                        Input_AppendEvent(*down);
                    }
                }
            }
            _ => {}
        }
    }

    Profiler_End();
}

#[no_mangle]
pub unsafe extern "C" fn Input_LoadGamepadDatabase(name: *const libc::c_char) {
    let path: *const libc::c_char = Resource_GetPath(ResourceType_Other, name);
    let result: i32 = SDL_GameControllerAddMappingsFromRW(SDL_RWFromFile(path, c_str!("rb")), 1);
    if result == -1 {
        panic!("Input_Init: Failed to add gamepad mappings");
    }
}

/* --- Direct Query API (Automatic Device) ---------------------------------- */

#[no_mangle]
pub unsafe extern "C" fn Input_GetPressed(button: Button) -> bool {
    let device: Device = Device {
        ty: Button_ToDeviceType(button),
        id: 0,
    };
    Input_GetDevicePressedImpl(device, button)
}

#[no_mangle]
pub unsafe extern "C" fn Input_GetDown(button: Button) -> bool {
    let device: Device = Device {
        ty: Button_ToDeviceType(button),
        id: 0,
    };
    Input_GetDeviceDownImpl(device, button)
}

#[no_mangle]
pub unsafe extern "C" fn Input_GetReleased(button: Button) -> bool {
    let device: Device = Device {
        ty: Button_ToDeviceType(button),
        id: 0,
    };
    Input_GetDeviceReleasedImpl(device, button)
}

#[no_mangle]
pub unsafe extern "C" fn Input_GetValue(button: Button) -> f32 {
    let device: Device = Device {
        ty: Button_ToDeviceType(button),
        id: 0,
    };
    Input_GetDeviceValueImpl(device, button)
}

#[no_mangle]
pub unsafe extern "C" fn Input_GetIdleTime() -> f32 {
    (this.lastTimestamp).wrapping_sub(this.lastEventTimestamp) as f32 / 1000.0f32
}

/* --- Direct Query API (Active Device) ------------------------------------- */

#[no_mangle]
pub unsafe extern "C" fn Input_GetActiveDevice(device: *mut Device) {
    *device = this.activeDevice;
}

#[no_mangle]
pub unsafe extern "C" fn Input_GetActiveDeviceType() -> DeviceType {
    this.activeDevice.ty
}

#[no_mangle]
pub unsafe extern "C" fn Input_GetActiveDeviceID() -> u32 {
    this.activeDevice.id
}

#[no_mangle]
pub unsafe extern "C" fn Input_GetActiveDeviceIdleTime() -> f32 {
    Input_GetDeviceIdleTimeImpl(this.activeDevice)
}

/* --- Direct Query API (Specified Device) ---------------------------------- */

#[no_mangle]
pub unsafe extern "C" fn Input_GetDevicePressed(device: *mut Device, button: Button) -> bool {
    if !Input_GetDeviceExists(*device) {
        return false;
    }
    Input_GetDevicePressedImpl(*device, button)
}

#[no_mangle]
pub unsafe extern "C" fn Input_GetDeviceDown(device: *mut Device, button: Button) -> bool {
    if !Input_GetDeviceExists(*device) {
        return false;
    }
    Input_GetDeviceDownImpl(*device, button)
}

#[no_mangle]
pub unsafe extern "C" fn Input_GetDeviceReleased(device: *mut Device, button: Button) -> bool {
    if !Input_GetDeviceExists(*device) {
        return false;
    }
    Input_GetDeviceReleasedImpl(*device, button)
}

#[no_mangle]
pub unsafe extern "C" fn Input_GetDeviceValue(device: *mut Device, button: Button) -> f32 {
    if !Input_GetDeviceExists(*device) {
        return 0.0f32;
    }
    Input_GetDeviceValueImpl(*device, button)
}

#[no_mangle]
pub unsafe extern "C" fn Input_GetDeviceIdleTime(device: *mut Device) -> f32 {
    if !Input_GetDeviceExists(*device) {
        return f32::MAX;
    }
    Input_GetDeviceIdleTimeImpl(*device)
}

/* --- Direct Query API (Mouse Device) -------------------------------------- */

#[no_mangle]
pub unsafe extern "C" fn Input_GetMouseDelta(delta: *mut IVec2) {
    (*delta).x = Input_GetValue(Button_Mouse_X) as i32 - this.lastMousePosition.x;
    (*delta).y = Input_GetValue(Button_Mouse_Y) as i32 - this.lastMousePosition.y;
}

#[no_mangle]
pub unsafe extern "C" fn Input_GetMouseIdleTime() -> f32 {
    let device: Device = Device {
        ty: DeviceType_Mouse,
        id: 0,
    };
    Input_GetDeviceIdleTimeImpl(device)
}

#[no_mangle]
pub unsafe extern "C" fn Input_GetMousePosition(position: *mut IVec2) {
    let device: Device = Device {
        ty: DeviceType_Mouse,
        id: 0,
    };
    (*position).x = Input_GetDeviceValueImpl(device, Button_Mouse_X) as i32;
    (*position).y = Input_GetDeviceValueImpl(device, Button_Mouse_Y) as i32;
}

#[no_mangle]
pub unsafe extern "C" fn Input_GetMouseScroll(scroll: *mut IVec2) {
    let device: Device = Device {
        ty: DeviceType_Mouse,
        id: 0,
    };
    (*scroll).x = Input_GetDeviceValueImpl(device, Button_Mouse_ScrollX) as i32;
    (*scroll).y = Input_GetDeviceValueImpl(device, Button_Mouse_ScrollY) as i32;
}

#[no_mangle]
pub unsafe extern "C" fn Input_SetMousePosition(position: *mut IVec2) {
    SDL_WarpMouseInWindow(std::ptr::null_mut(), (*position).x, (*position).y);
}

#[no_mangle]
pub unsafe extern "C" fn Input_SetMouseVisible(visible: bool) {
    this.autoHideMouse = false;
    SDL_ShowCursor((if visible { SDL_ENABLE } else { SDL_DISABLE }) as i32);
}

#[no_mangle]
pub unsafe extern "C" fn Input_SetMouseVisibleAuto() {
    this.autoHideMouse = true;
    Input_SetActiveDevice(this.activeDevice);
}

#[no_mangle]
pub unsafe extern "C" fn Input_SetMouseScroll(scroll: *mut IVec2) {
    let timestamp: u32 = SDL_GetTicks();
    let device: Device = Device {
        ty: DeviceType_Mouse,
        id: 0,
    };

    let mut event: InputEvent = InputEvent {
        timestamp: 0,
        device: Device { ty: 0, id: 0 },
        button: 0,
        value: 0.,
        state: 0,
    };

    event.timestamp = timestamp;
    event.device = device;
    event.button = Button_Mouse_ScrollX;
    event.value = (*scroll).x as f32;
    event.state = State_Changed | Input_DetermineButtonState(event);
    if event.value != Input_GetDeviceValueImpl(event.device, event.button) {
        Input_InjectEvent(event);
    }

    event.timestamp = timestamp;
    event.device = device;
    event.button = Button_Mouse_ScrollY;
    event.value = (*scroll).y as f32;
    event.state = State_Changed | Input_DetermineButtonState(event);
    if event.value != Input_GetDeviceValueImpl(event.device, event.button) {
        Input_InjectEvent(event);
    }
}

/* --- Direct Query API (Keyboard Device) ----------------------------------- */

#[no_mangle]
pub unsafe extern "C" fn Input_GetKeyboardIdleTime() -> f32 {
    let device: Device = Device {
        ty: DeviceType_Keyboard,
        id: 0,
    };
    Input_GetDeviceIdleTimeImpl(device)
}

#[no_mangle]
pub unsafe extern "C" fn Input_GetKeyboardMod(modifier: Modifier) -> bool {
    let mut hasMod: bool = true;
    if modifier & Modifier_Alt == Modifier_Alt {
        hasMod = hasMod && Input_GetKeyboardAlt();
    }
    if modifier & Modifier_Ctrl == Modifier_Ctrl {
        hasMod = hasMod && Input_GetKeyboardCtrl();
    }
    if modifier & Modifier_Shift == Modifier_Shift {
        hasMod = hasMod && Input_GetKeyboardShift();
    }
    hasMod
}

#[no_mangle]
pub unsafe extern "C" fn Input_GetKeyboardAlt() -> bool {
    let device: Device = Device {
        ty: DeviceType_Keyboard,
        id: 0,
    };
    let deviceState: *mut DeviceState = Input_GetDeviceState(device);
    (*deviceState).buttons[Button_Keyboard_LAlt as usize]
        || (*deviceState).buttons[Button_Keyboard_RAlt as usize]
}

#[no_mangle]
pub unsafe extern "C" fn Input_GetKeyboardCtrl() -> bool {
    let device: Device = Device {
        ty: DeviceType_Keyboard,
        id: 0,
    };
    let deviceState: *mut DeviceState = Input_GetDeviceState(device);
    (*deviceState).buttons[Button_Keyboard_LCtrl as usize]
        || (*deviceState).buttons[Button_Keyboard_RCtrl as usize]
}

#[no_mangle]
pub unsafe extern "C" fn Input_GetKeyboardShift() -> bool {
    let device: Device = Device {
        ty: DeviceType_Keyboard,
        id: 0,
    };
    let deviceState: *mut DeviceState = Input_GetDeviceState(device);
    (*deviceState).buttons[Button_Keyboard_LShift as usize]
        || (*deviceState).buttons[Button_Keyboard_RShift as usize]
}

/* --- Direct Query API (Gamepad Device) ------------------------------------ */

#[no_mangle]
pub unsafe extern "C" fn Input_GetGamepadIdleTime(id: u32) -> f32 {
    let device: Device = Device {
        ty: DeviceType_Gamepad,
        id,
    };
    if !Input_GetDeviceExists(device) {
        return f32::MAX;
    }
    Input_GetDeviceIdleTimeImpl(device)
}

#[no_mangle]
pub unsafe extern "C" fn Input_GetGamepadPressed(id: u32, button: Button) -> bool {
    let device: Device = Device {
        ty: DeviceType_Gamepad,
        id,
    };
    if !Input_GetDeviceExists(device) {
        return false;
    }
    Input_GetDevicePressedImpl(device, button)
}

#[no_mangle]
pub unsafe extern "C" fn Input_GetGamepadDown(id: u32, button: Button) -> bool {
    let device: Device = Device {
        ty: DeviceType_Gamepad,
        id,
    };
    if !Input_GetDeviceExists(device) {
        return false;
    }
    Input_GetDeviceDownImpl(device, button)
}

#[no_mangle]
pub unsafe extern "C" fn Input_GetGamepadReleased(id: u32, button: Button) -> bool {
    let device: Device = Device {
        ty: DeviceType_Gamepad,
        id,
    };
    if !Input_GetDeviceExists(device) {
        return false;
    }
    Input_GetDeviceReleasedImpl(device, button)
}

#[no_mangle]
pub unsafe extern "C" fn Input_GetGamepadValue(id: u32, button: Button) -> f32 {
    let device: Device = Device {
        ty: DeviceType_Gamepad,
        id,
    };
    if !Input_GetDeviceExists(device) {
        return 0.0f32;
    }
    Input_GetDeviceValueImpl(device, button)
}

/* --- Event Loop API ------------------------------------------------------- */

#[no_mangle]
pub unsafe extern "C" fn Input_GetEventCount() -> i32 {
    this.events.len() as i32
}

#[no_mangle]
pub unsafe extern "C" fn Input_GetNextEvent(event: *mut InputEvent) -> bool {
    if this.events.is_empty() {
        return false;
    }
    Profiler_Begin(c_str!("Input_GetNextEvent"));

    *event = this.events[0];
    this.events.remove(0);

    Profiler_End();
    true
}

/* NOTE : Controller instance ID's are signed (sdl.cbutton/caxis/cdevice.which)
 *        while mouse instance ID's are unsigned (sdl.button/motion/wheel.which).
 *        This is a bit annoying since we can store both in Device.id. I've
 *        opted to store them as uint32's since they're used as array indices
 *        here (and presumably in SDL as well). */

/* NOTE : We keep a permanent DeviceState for every joystick instance id which
 *        is montonically increasing each time a joystick is connected. This is
 *        kind of leaky in a sense, but realistically seems like it's never
 *        going to be a problem. */

/* NOTE : SDL GameController Info
 *        SDL_CONTROLLERDEVICEADDED is always sent when a controller is connected
 *        SDL_CONTROLLERDEVICEREMOVED is only sent if the controller was opened
 *        SDL sends events to zero all controller input right before device removal
 *        SDL does not send any events to fix up input when a controller is remapped */

/* TODO : I changed instantaneous events such as Button_System_Exit to be
 *        pressed in the current frame then auto-released in the following
 *        frame. This probably makes sense for Button_Mouse_ScrollX/Y because we
 *        want GetValue queries to return the scroll value throughout the entire
 *        frame, but maybe makes less sense for Exit where the value is not
 *        terribly important and GetPressed will already return true. Supporting
 *        this, any random button that is pressed and released in the same frame
 *        already behave that way: value will be 0 and pressed will be true. */
