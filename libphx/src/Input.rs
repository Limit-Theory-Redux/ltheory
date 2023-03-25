use crate::internal::Memory::*;
use crate::Button::*;
use crate::Common::*;
use crate::Device::*;
use crate::DeviceType::*;
use crate::InputEvent::*;
use crate::Math::IVec2;
use crate::Math::Vec3;
use crate::Modifier::*;
use crate::Profiler::*;
use crate::Resource::*;
use crate::ResourceType::*;
use crate::State::*;
use libc;
use sdl2_sys::*;

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

static mut Threshold_Pressed: f32 = 0.5f32;

static mut Threshold_Released: f32 = 0.4f32;

static mut this: Input = Input {
    activeDevice: Device { type_0: 0, id: 0 },
    lastTimestamp: 0,
    lastEventTimestamp: 0,
    lastMousePosition: IVec2 { x: 0, y: 0 },
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
    let deviceList: *mut DeviceList = &mut *(this.deviceLists)
        .as_mut_ptr()
        .offset(device.type_0 as isize) as *mut DeviceList;
    while (*deviceList).devices.len() as i32 as u32 <= device.id {
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
    let deviceList: *mut DeviceList = &mut *(this.deviceLists)
        .as_mut_ptr()
        .offset(device.type_0 as isize) as *mut DeviceList;
    &mut (*deviceList).devices[device.id as usize]
}

#[inline]
unsafe extern "C" fn Input_SetActiveDevice(device: Device) {
    this.activeDevice = device;
    if this.autoHideMouse {
        SDL_ShowCursor(if device.type_0 == DeviceType_Mouse {
            1
        } else {
            0
        });
    }
}

#[inline]
unsafe extern "C" fn Input_GetDeviceExists(device: Device) -> bool {
    let deviceList: *mut DeviceList = &mut *(this.deviceLists)
        .as_mut_ptr()
        .offset(device.type_0 as isize) as *mut DeviceList;
    if device.id < (*deviceList).devices.len() as i32 as u32 {
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
    if (*deviceState).buttons[button as usize] as i32 != 0 {
        (*deviceState).transitions[button as usize] > 0
    } else {
        (*deviceState).transitions[button as usize] > 1
    }
}

#[inline]
unsafe extern "C" fn Input_GetDeviceDownImpl(device: Device, button: Button) -> bool {
    let deviceState: *mut DeviceState = Input_GetDeviceState(device);
    (*deviceState).buttons[button as usize] as i32 != 0
        || (*deviceState).transitions[button as usize] > 0
}

#[inline]
unsafe extern "C" fn Input_GetDeviceReleasedImpl(device: Device, button: Button) -> bool {
    let deviceState: *mut DeviceState = Input_GetDeviceState(device);
    if (*deviceState).buttons[button as usize] as i32 != 0 {
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
    if down as i32 != 0 && event.value < Threshold_Released {
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
    let deviceState: *mut DeviceState = Input_GetDeviceState(event.device);
    (*deviceState).axes[event.button as usize] = event.value;
    let down: bool = (*deviceState).buttons[event.button as usize];
    if !down && event.state & State_Pressed == State_Pressed {
        (*deviceState).transitions[event.button as usize] += 1;
        (*deviceState).buttons[event.button as usize] = true;
        this.downButtons.push(event);
        if event.device.type_0 != DeviceType_Null {
            Input_SetActiveDevice(event.device);
        }
    }

    if down as i32 != 0 && event.state & State_Released == State_Released {
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
    let result: SDL_bool = SDL_SetHint(c_str!("SDL_MOUSE_FOCUS_CLICKTHROUGH"), c_str!("1"));
    if result != SDL_bool::SDL_TRUE {
        Warn(c_str!("Input_Init: SDL_SetHint failed"));
    }

    let mut iDev: i32 = 0;
    while iDev < 4 {
        let device: Device = Device {
            type_0: iDev,
            id: 0,
        };
        let deviceState: *mut DeviceState = Input_EnsureDeviceState(device);
        (*deviceState).isConnected = iDev != DeviceType_Gamepad;
        iDev += 1;
    }

    this.events.reserve(16);
    this.downButtons.reserve(16);
    this.autoRelease.reserve(16);
    this.injectedEvents.reserve(16);

    let device: Device = Device {
        type_0: DeviceType_Mouse,
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
    this.lastTimestamp = SDL_GetTicks();
    this.lastMousePosition.x = Input_GetValue(Button_Mouse_X) as i32;
    this.lastMousePosition.y = Input_GetValue(Button_Mouse_Y) as i32;
    let mut iDev: i32 = 0;
    while iDev < 4 {
        let deviceList: *mut DeviceList =
            &mut *(this.deviceLists).as_mut_ptr().offset(iDev as isize) as *mut DeviceList;
        for deviceState in (*deviceList).devices.iter_mut() {
            MemSet(
                ((*deviceState).transitions).as_mut_ptr() as *mut _,
                0,
                std::mem::size_of::<[i32; 512]>(),
            );
        }
        iDev += 1;
    }
    this.events.clear();
    for event in this.injectedEvents.iter() {
        Input_AppendEvent(*event);
    }
    this.injectedEvents.clear();
    for down in this.autoRelease.iter_mut() {
        let deviceState_0: *mut DeviceState = Input_GetDeviceState((*down).device);
        if (*deviceState_0).axes[(*down).button as usize] != 0.0f32 {
            (*down).value = 0.0f32;
            (*down).state = State_Changed | Input_DetermineButtonState(*down);
            (*down).timestamp = SDL_GetTicks();
            Input_SetButton(*down);
            Input_AppendEvent(*down);
        }
    }
    for down in this.downButtons.iter_mut() {
        let deviceState_1: *mut DeviceState = Input_GetDeviceState((*down).device);
        (*down).value = (*deviceState_1).axes[(*down).button as usize];
        (*down).state = State_Down;
        (*down).timestamp = SDL_GetTicks();
        Input_AppendEvent(*down);
    }
    let mut sdl: SDL_Event = SDL_Event { type_: 0 };
    while SDL_PollEvent(&mut sdl) != 0 {
        let mut event_0: InputEvent = InputEvent {
            timestamp: 0,
            device: Device { type_0: 0, id: 0 },
            button: 0,
            value: 0.,
            state: 0,
        };
        event_0.timestamp = sdl.common.timestamp;
        match std::mem::transmute::<i32, SDL_EventType>(sdl.type_ as i32) {
            SDL_EventType::SDL_KEYDOWN => {
                if sdl.key.repeat != 0 {
                    continue;
                }
                let device: Device = Device {
                    type_0: DeviceType_Keyboard,
                    id: 0,
                };
                event_0.device = device;
                event_0.button = Button_FromSDLScancode(sdl.key.keysym.scancode);
                event_0.value = 1.0f32;
                event_0.state = State_Changed | State_Pressed | State_Down;
                if event_0.button == Button_Null {
                    continue;
                }
                Input_SetButton(event_0);
                Input_AppendEvent(event_0);
            }
            SDL_EventType::SDL_KEYUP => {
                let device_0: Device = Device {
                    type_0: DeviceType_Keyboard,
                    id: 0,
                };
                event_0.device = device_0;
                event_0.button = Button_FromSDLScancode(sdl.key.keysym.scancode);
                event_0.value = 0.0f32;
                event_0.state = State_Changed | State_Released;
                if event_0.button == Button_Null {
                    continue;
                }
                Input_SetButton(event_0);
                Input_AppendEvent(event_0);
            }
            SDL_EventType::SDL_MOUSEBUTTONDOWN => {
                let device_1: Device = Device {
                    type_0: DeviceType_Mouse,
                    id: sdl.button.which,
                };
                event_0.device = device_1;
                event_0.button = Button_FromSDLMouseButton(sdl.button.button);
                event_0.value = 1.0f32;
                event_0.state = State_Changed | State_Pressed | State_Down;
                Input_EnsureDeviceState(event_0.device);
                Input_SetButton(event_0);
                Input_AppendEvent(event_0);
            }
            SDL_EventType::SDL_MOUSEBUTTONUP => {
                let device_2: Device = Device {
                    type_0: DeviceType_Mouse,
                    id: sdl.button.which,
                };
                event_0.device = device_2;
                event_0.button = Button_FromSDLMouseButton(sdl.button.button);
                event_0.value = 0.0f32;
                event_0.state = State_Changed | State_Released;
                Input_EnsureDeviceState(event_0.device);
                Input_SetButton(event_0);
                Input_AppendEvent(event_0);
            }
            SDL_EventType::SDL_MOUSEMOTION => {
                let device_3: Device = Device {
                    type_0: DeviceType_Mouse,
                    id: sdl.motion.which,
                };
                let deviceState_2: *mut DeviceState = Input_EnsureDeviceState(device_3);
                event_0.device = device_3;
                event_0.button = Button_Mouse_X;
                event_0.value = sdl.motion.x as f32;
                event_0.state = State_Changed;
                if event_0.value != (*deviceState_2).axes[event_0.button as usize] {
                    (*deviceState_2).axes[event_0.button as usize] = event_0.value;
                    Input_SetActiveDevice(event_0.device);
                    Input_AppendEvent(event_0);
                }
                event_0.device = device_3;
                event_0.button = Button_Mouse_Y;
                event_0.value = sdl.motion.y as f32;
                event_0.state = State_Changed;
                if event_0.value != (*deviceState_2).axes[event_0.button as usize] {
                    (*deviceState_2).axes[event_0.button as usize] = event_0.value;
                    Input_SetActiveDevice(event_0.device);
                    Input_AppendEvent(event_0);
                }
            }
            SDL_EventType::SDL_MOUSEWHEEL => {
                let device_4: Device = Device {
                    type_0: DeviceType_Mouse,
                    id: sdl.wheel.which,
                };
                Input_EnsureDeviceState(device_4);
                event_0.device = device_4;
                event_0.button = Button_Mouse_ScrollX;
                event_0.value = sdl.wheel.x as f32;
                event_0.state = State_Changed | Input_DetermineButtonState(event_0);
                if event_0.value != Input_GetDeviceValueImpl(event_0.device, event_0.button) {
                    Input_SetButton(event_0);
                    Input_AppendEvent(event_0);
                }
                event_0.device = device_4;
                event_0.button = Button_Mouse_ScrollY;
                event_0.value = sdl.wheel.y as f32;
                event_0.state = State_Changed | Input_DetermineButtonState(event_0);
                if event_0.value != Input_GetDeviceValueImpl(event_0.device, event_0.button) {
                    Input_SetButton(event_0);
                    Input_AppendEvent(event_0);
                }
            }
            SDL_EventType::SDL_CONTROLLERBUTTONDOWN => {
                let device_5: Device = Device {
                    type_0: DeviceType_Gamepad,
                    id: sdl.cbutton.which as u32,
                };
                event_0.device = device_5;
                event_0.button =
                    Button_FromSDLControllerButton(std::mem::transmute(sdl.cbutton.button as i32));
                event_0.value = 1.0f32;
                event_0.state = State_Changed | State_Pressed | State_Down;
                Input_SetButton(event_0);
                Input_AppendEvent(event_0);
            }
            SDL_EventType::SDL_CONTROLLERBUTTONUP => {
                let device_6: Device = Device {
                    type_0: DeviceType_Gamepad,
                    id: sdl.cbutton.which as u32,
                };
                event_0.device = device_6;
                event_0.button =
                    Button_FromSDLControllerButton(std::mem::transmute(sdl.cbutton.button as i32));
                event_0.value = 0.0f32;
                event_0.state = State_Changed | State_Released;
                Input_SetButton(event_0);
                Input_AppendEvent(event_0);
            }
            SDL_EventType::SDL_CONTROLLERAXISMOTION => {
                let device_7: Device = Device {
                    type_0: DeviceType_Gamepad,
                    id: sdl.caxis.which as u32,
                };
                let mut value: f32 = f64::clamp(
                    (sdl.caxis.value as f32 / 32767.0f32) as f64,
                    -1.0f64,
                    1.0f64,
                ) as f32;
                let axis: SDL_GameControllerAxis = std::mem::transmute(sdl.caxis.axis as i32);
                if axis == SDL_GameControllerAxis::SDL_CONTROLLER_AXIS_LEFTY
                    || axis == SDL_GameControllerAxis::SDL_CONTROLLER_AXIS_RIGHTY
                {
                    value = -value;
                }
                event_0.device = device_7;
                event_0.button = Button_FromSDLControllerAxis(axis);
                event_0.value = value;
                event_0.state = State_Changed | Input_DetermineButtonState(event_0);
                Input_SetButton(event_0);
                Input_AppendEvent(event_0);
            }
            SDL_EventType::SDL_CONTROLLERDEVICEADDED => {
                if SDL_IsGameController(sdl.cdevice.which) == SDL_bool::SDL_TRUE {
                    let sdlController: *mut SDL_GameController =
                        SDL_GameControllerOpen(sdl.cdevice.which);
                    if sdlController.is_null() {
                        Warn(c_str!("Input_Update: SDL_GameControllerOpen failed"));
                    } else {
                        let sdlJoystick: *mut SDL_Joystick =
                            SDL_GameControllerGetJoystick(sdlController);
                        let id: u32 = SDL_JoystickInstanceID(sdlJoystick) as u32;
                        let device_8: Device = Device {
                            type_0: DeviceType_Gamepad,
                            id: id,
                        };
                        let deviceState_3: *mut DeviceState = Input_EnsureDeviceState(device_8);
                        (*deviceState_3).isConnected = true;
                    }
                }
            }
            SDL_EventType::SDL_CONTROLLERDEVICEREMOVED => {
                /* NOTE : SDL already sends events to zero out all game controller
                 *        input so there's no need to do it manually. */

                let device_9: Device = Device {
                    type_0: DeviceType_Gamepad,
                    id: sdl.cdevice.which as u32,
                };
                let deviceState_4: *mut DeviceState = Input_GetDeviceState(device_9);
                (*deviceState_4).isConnected = false;
                let sdlController_0: *mut SDL_GameController =
                    SDL_GameControllerFromInstanceID(sdl.cdevice.which);
                if !sdlController_0.is_null() {
                    SDL_GameControllerClose(sdlController_0);
                }
            }
            /* TODO : Maybe we should release all input then re-set it? */
            SDL_EventType::SDL_CONTROLLERDEVICEREMAPPED => {
                let sdlController_1: *mut SDL_GameController =
                    SDL_GameControllerFromInstanceID(sdl.cdevice.which);
                let device_10: Device = Device {
                    type_0: DeviceType_Gamepad,
                    id: sdl.cdevice.which as u32,
                };
                let deviceState_5: *mut DeviceState = Input_GetDeviceState(device_10);
                let mut iBtn: i32 = Button_Gamepad_Button_First;
                while iBtn <= Button_Gamepad_Button_Last {
                    let value_0: f32 = SDL_GameControllerGetButton(
                        sdlController_1,
                        Button_ToSDLControllerButton(iBtn),
                    ) as f32;
                    if value_0 != (*deviceState_5).axes[iBtn as usize] {
                        event_0.device = device_10;
                        event_0.button = iBtn;
                        event_0.value = value_0;
                        event_0.state = State_Changed | Input_DetermineButtonState(event_0);
                        Input_SetButton(event_0);
                        Input_AppendEvent(event_0);
                    }
                    iBtn += 1;
                }
                let mut iAxis: i32 = Button_Gamepad_Axis_First;
                while iAxis <= Button_Gamepad_Axis_Last {
                    let mut value_1: f32 = SDL_GameControllerGetAxis(
                        sdlController_1,
                        Button_ToSDLControllerAxis(iAxis),
                    ) as f32;
                    value_1 = f64::clamp((value_1 / 32767.0f32) as f64, -1.0f64, 1.0f64) as f32;
                    if iAxis == Button_Gamepad_LStickY || iAxis == Button_Gamepad_RStickY {
                        value_1 = -value_1;
                    }
                    if value_1 != (*deviceState_5).axes[iAxis as usize] {
                        event_0.device = device_10;
                        event_0.button = iAxis;
                        event_0.value = value_1;
                        event_0.state = State_Changed | Input_DetermineButtonState(event_0);
                        Input_SetButton(event_0);
                        Input_AppendEvent(event_0);
                    }
                    iAxis += 1;
                }
            }
            SDL_EventType::SDL_QUIT => {
                let device_11: Device = Device {
                    type_0: DeviceType_Null,
                    id: 0,
                };
                event_0.device = device_11;
                event_0.button = Button_System_Exit;
                event_0.value = 0.0f32;
                event_0.state = State_Changed | State_Pressed | State_Down;
                Input_SetButton(event_0);
                Input_AppendEvent(event_0);
            }
            SDL_EventType::SDL_WINDOWEVENT => {
                if sdl.window.event as i32 == SDL_WindowEventID::SDL_WINDOWEVENT_FOCUS_GAINED as i32
                {
                    SDL_CaptureMouse(SDL_bool::SDL_TRUE);
                }
                if sdl.window.event as i32 == SDL_WindowEventID::SDL_WINDOWEVENT_FOCUS_LOST as i32 {
                    SDL_CaptureMouse(SDL_bool::SDL_FALSE);
                    for down in this.downButtons.iter_mut() {
                        (*down).timestamp = sdl.common.timestamp;
                        (*down).value = 0.0f32;
                        (*down).state = State_Changed | Input_DetermineButtonState(event_0);
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
        Fatal(c_str!("Input_Init: Failed to add gamepad mappings"));
    }
}

#[no_mangle]
pub unsafe extern "C" fn Input_GetPressed(button: Button) -> bool {
    let device: Device = Device {
        type_0: Button_ToDeviceType(button),
        id: 0,
    };
    Input_GetDevicePressedImpl(device, button)
}

#[no_mangle]
pub unsafe extern "C" fn Input_GetDown(button: Button) -> bool {
    let device: Device = Device {
        type_0: Button_ToDeviceType(button),
        id: 0,
    };
    Input_GetDeviceDownImpl(device, button)
}

#[no_mangle]
pub unsafe extern "C" fn Input_GetReleased(button: Button) -> bool {
    let device: Device = Device {
        type_0: Button_ToDeviceType(button),
        id: 0,
    };
    Input_GetDeviceReleasedImpl(device, button)
}

#[no_mangle]
pub unsafe extern "C" fn Input_GetValue(button: Button) -> f32 {
    let device: Device = Device {
        type_0: Button_ToDeviceType(button),
        id: 0,
    };
    Input_GetDeviceValueImpl(device, button)
}

#[no_mangle]
pub unsafe extern "C" fn Input_GetIdleTime() -> f32 {
    (this.lastTimestamp).wrapping_sub(this.lastEventTimestamp) as f32 / 1000.0f32
}

#[no_mangle]
pub unsafe extern "C" fn Input_GetActiveDevice(device: *mut Device) {
    *device = this.activeDevice;
}

#[no_mangle]
pub unsafe extern "C" fn Input_GetActiveDeviceType() -> DeviceType {
    this.activeDevice.type_0
}

#[no_mangle]
pub unsafe extern "C" fn Input_GetActiveDeviceID() -> u32 {
    this.activeDevice.id
}

#[no_mangle]
pub unsafe extern "C" fn Input_GetActiveDeviceIdleTime() -> f32 {
    Input_GetDeviceIdleTimeImpl(this.activeDevice)
}

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

#[no_mangle]
pub unsafe extern "C" fn Input_GetMouseDelta(delta: *mut IVec2) {
    (*delta).x = Input_GetValue(Button_Mouse_X) as i32 - this.lastMousePosition.x;
    (*delta).y = Input_GetValue(Button_Mouse_Y) as i32 - this.lastMousePosition.y;
}

#[no_mangle]
pub unsafe extern "C" fn Input_GetMouseIdleTime() -> f32 {
    let device: Device = Device {
        type_0: DeviceType_Mouse,
        id: 0,
    };
    Input_GetDeviceIdleTimeImpl(device)
}

#[no_mangle]
pub unsafe extern "C" fn Input_GetMousePosition(position: *mut IVec2) {
    let device: Device = Device {
        type_0: DeviceType_Mouse,
        id: 0,
    };
    (*position).x = Input_GetDeviceValueImpl(device, Button_Mouse_X) as i32;
    (*position).y = Input_GetDeviceValueImpl(device, Button_Mouse_Y) as i32;
}

#[no_mangle]
pub unsafe extern "C" fn Input_GetMouseScroll(scroll: *mut IVec2) {
    let device: Device = Device {
        type_0: DeviceType_Mouse,
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
    SDL_ShowCursor(if visible as i32 != 0 { 1 } else { 0 });
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
        type_0: DeviceType_Mouse,
        id: 0,
    };
    let mut event: InputEvent = InputEvent {
        timestamp: 0,
        device: Device { type_0: 0, id: 0 },
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

#[no_mangle]
pub unsafe extern "C" fn Input_GetKeyboardIdleTime() -> f32 {
    let device: Device = Device {
        type_0: DeviceType_Keyboard,
        id: 0,
    };
    Input_GetDeviceIdleTimeImpl(device)
}

#[no_mangle]
pub unsafe extern "C" fn Input_GetKeyboardMod(modifier: Modifier) -> bool {
    let mut hasMod: bool = true;
    if modifier & Modifier_Alt == Modifier_Alt {
        hasMod = (hasMod as i32 & Input_GetKeyboardAlt() as i32) != 0;
    }
    if modifier & Modifier_Ctrl == Modifier_Ctrl {
        hasMod = (hasMod as i32 & Input_GetKeyboardCtrl() as i32) != 0;
    }
    if modifier & Modifier_Shift == Modifier_Shift {
        hasMod = (hasMod as i32 & Input_GetKeyboardShift() as i32) != 0;
    }
    hasMod
}

#[no_mangle]
pub unsafe extern "C" fn Input_GetKeyboardAlt() -> bool {
    let device: Device = Device {
        type_0: DeviceType_Keyboard,
        id: 0,
    };
    let deviceState: *mut DeviceState = Input_GetDeviceState(device);
    (*deviceState).buttons[Button_Keyboard_LAlt as usize] as i32 != 0
        || (*deviceState).buttons[Button_Keyboard_RAlt as usize] as i32 != 0
}

#[no_mangle]
pub unsafe extern "C" fn Input_GetKeyboardCtrl() -> bool {
    let device: Device = Device {
        type_0: DeviceType_Keyboard,
        id: 0,
    };
    let deviceState: *mut DeviceState = Input_GetDeviceState(device);
    (*deviceState).buttons[Button_Keyboard_LCtrl as usize] as i32 != 0
        || (*deviceState).buttons[Button_Keyboard_RCtrl as usize] as i32 != 0
}

#[no_mangle]
pub unsafe extern "C" fn Input_GetKeyboardShift() -> bool {
    let device: Device = Device {
        type_0: DeviceType_Keyboard,
        id: 0,
    };
    let deviceState: *mut DeviceState = Input_GetDeviceState(device);
    (*deviceState).buttons[Button_Keyboard_LShift as usize] as i32 != 0
        || (*deviceState).buttons[Button_Keyboard_RShift as usize] as i32 != 0
}

#[no_mangle]
pub unsafe extern "C" fn Input_GetGamepadIdleTime(id: u32) -> f32 {
    let device: Device = Device {
        type_0: DeviceType_Gamepad,
        id: id,
    };
    if !Input_GetDeviceExists(device) {
        return f32::MAX;
    }
    Input_GetDeviceIdleTimeImpl(device)
}

#[no_mangle]
pub unsafe extern "C" fn Input_GetGamepadPressed(id: u32, button: Button) -> bool {
    let device: Device = Device {
        type_0: DeviceType_Gamepad,
        id: id,
    };
    if !Input_GetDeviceExists(device) {
        return false;
    }
    Input_GetDevicePressedImpl(device, button)
}

#[no_mangle]
pub unsafe extern "C" fn Input_GetGamepadDown(id: u32, button: Button) -> bool {
    let device: Device = Device {
        type_0: DeviceType_Gamepad,
        id: id,
    };
    if !Input_GetDeviceExists(device) {
        return false;
    }
    Input_GetDeviceDownImpl(device, button)
}

#[no_mangle]
pub unsafe extern "C" fn Input_GetGamepadReleased(id: u32, button: Button) -> bool {
    let device: Device = Device {
        type_0: DeviceType_Gamepad,
        id: id,
    };
    if !Input_GetDeviceExists(device) {
        return false;
    }
    Input_GetDeviceReleasedImpl(device, button)
}

#[no_mangle]
pub unsafe extern "C" fn Input_GetGamepadValue(id: u32, button: Button) -> f32 {
    let device: Device = Device {
        type_0: DeviceType_Gamepad,
        id: id,
    };
    if !Input_GetDeviceExists(device) {
        return 0.0f32;
    }
    Input_GetDeviceValueImpl(device, button)
}

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
