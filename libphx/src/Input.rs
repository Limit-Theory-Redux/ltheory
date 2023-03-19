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
    pub transitions: [i32; 512],
    pub buttons: [bool; 512],
    pub axes: [f32; 512],
    pub lastEventTimestamp: u32,
    pub isConnected: bool,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct DeviceList {
    pub devices_size: i32,
    pub devices_capacity: i32,
    pub devices_data: *mut DeviceState,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Input {
    pub activeDevice: Device,
    pub lastTimestamp: u32,
    pub lastEventTimestamp: u32,
    pub lastMousePosition: IVec2,
    pub autoHideMouse: bool,
    pub deviceLists: [DeviceList; 4],
    pub events_size: i32,
    pub events_capacity: i32,
    pub events_data: *mut InputEvent,
    pub downButtons_size: i32,
    pub downButtons_capacity: i32,
    pub downButtons_data: *mut InputEvent,
    pub autoRelease_size: i32,
    pub autoRelease_capacity: i32,
    pub autoRelease_data: *mut InputEvent,
    pub injectedEvents_size: i32,
    pub injectedEvents_capacity: i32,
    pub injectedEvents_data: *mut InputEvent,
}

static mut Threshold_Pressed: f32 = 0.5f32;

static mut Threshold_Released: f32 = 0.4f32;

static mut this: Input = Input {
    activeDevice: Device {
        type_0: 0_i32,
        id: 0,
    },
    lastTimestamp: 0,
    lastEventTimestamp: 0,
    lastMousePosition: IVec2 { x: 0, y: 0 },
    autoHideMouse: false,
    deviceLists: [DeviceList {
        devices_size: 0,
        devices_capacity: 0,
        devices_data: std::ptr::null_mut(),
    }; 4],
    events_size: 0,
    events_capacity: 0,
    events_data: std::ptr::null_mut(),
    downButtons_size: 0,
    downButtons_capacity: 0,
    downButtons_data: std::ptr::null_mut(),
    autoRelease_size: 0,
    autoRelease_capacity: 0,
    autoRelease_data: std::ptr::null_mut(),
    injectedEvents_size: 0,
    injectedEvents_capacity: 0,
    injectedEvents_data: std::ptr::null_mut(),
};

#[inline]
unsafe extern "C" fn Input_EnsureDeviceState(mut device: Device) -> *mut DeviceState {
    let mut deviceList: *mut DeviceList = &mut *(this.deviceLists)
        .as_mut_ptr()
        .offset(device.type_0 as isize)
        as *mut DeviceList;
    while (*deviceList).devices_size as u32 <= device.id {
        let mut deviceState: DeviceState = DeviceState {
            transitions: [0; 512],
            buttons: [false; 512],
            axes: [0.; 512],
            lastEventTimestamp: 0,
            isConnected: false,
        };
        if ((*deviceList).devices_capacity == (*deviceList).devices_size) as i32 as libc::c_long
            != 0
        {
            (*deviceList).devices_capacity = if (*deviceList).devices_capacity != 0 {
                (*deviceList).devices_capacity * 2_i32
            } else {
                1_i32
            };
            let mut elemSize: usize = std::mem::size_of::<DeviceState>();
            let mut pData: *mut *mut libc::c_void =
                &mut (*deviceList).devices_data as *mut *mut DeviceState as *mut *mut libc::c_void;
            *pData = MemRealloc(
                (*deviceList).devices_data as *mut libc::c_void,
                ((*deviceList).devices_capacity as usize).wrapping_mul(elemSize),
            );
        }
        let fresh0 = (*deviceList).devices_size;
        (*deviceList).devices_size += 1;
        *((*deviceList).devices_data).offset(fresh0 as isize) = deviceState;
    }
    ((*deviceList).devices_data).offset(device.id as isize)
}

#[inline]
unsafe extern "C" fn Input_GetDeviceState(mut device: Device) -> *mut DeviceState {
    let mut deviceList: *mut DeviceList = &mut *(this.deviceLists)
        .as_mut_ptr()
        .offset(device.type_0 as isize)
        as *mut DeviceList;
    ((*deviceList).devices_data).offset(device.id as isize)
}

#[inline]
unsafe extern "C" fn Input_SetActiveDevice(mut device: Device) {
    this.activeDevice = device;
    if this.autoHideMouse {
        SDL_ShowCursor(if device.type_0 == DeviceType_Mouse {
            1_i32
        } else {
            0_i32
        });
    }
}

#[inline]
unsafe extern "C" fn Input_GetDeviceExists(mut device: Device) -> bool {
    let mut deviceList: *mut DeviceList = &mut *(this.deviceLists)
        .as_mut_ptr()
        .offset(device.type_0 as isize)
        as *mut DeviceList;
    if device.id < (*deviceList).devices_size as u32 {
        let mut deviceState: *mut DeviceState =
            ((*deviceList).devices_data).offset(device.id as isize);
        return (*deviceState).isConnected;
    }
    false
}

#[inline]
unsafe extern "C" fn Input_GetDevicePressedImpl(mut device: Device, mut button: Button) -> bool {
    let mut deviceState: *mut DeviceState = Input_GetDeviceState(device);
    if (*deviceState).buttons[button as usize] as i32 != 0 {
        (*deviceState).transitions[button as usize] > 0_i32
    } else {
        (*deviceState).transitions[button as usize] > 1_i32
    }
}

#[inline]
unsafe extern "C" fn Input_GetDeviceDownImpl(mut device: Device, mut button: Button) -> bool {
    let mut deviceState: *mut DeviceState = Input_GetDeviceState(device);
    (*deviceState).buttons[button as usize] as i32 != 0
        || (*deviceState).transitions[button as usize] > 0_i32
}

#[inline]
unsafe extern "C" fn Input_GetDeviceReleasedImpl(mut device: Device, mut button: Button) -> bool {
    let mut deviceState: *mut DeviceState = Input_GetDeviceState(device);
    if (*deviceState).buttons[button as usize] as i32 != 0 {
        (*deviceState).transitions[button as usize] > 1_i32
    } else {
        (*deviceState).transitions[button as usize] > 0_i32
    }
}

#[inline]
unsafe extern "C" fn Input_GetDeviceValueImpl(mut device: Device, mut button: Button) -> f32 {
    let mut deviceState: *mut DeviceState = Input_GetDeviceState(device);
    (*deviceState).axes[button as usize]
}

#[inline]
unsafe extern "C" fn Input_GetDeviceIdleTimeImpl(mut device: Device) -> f32 {
    let mut deviceState: *mut DeviceState = Input_GetDeviceState(device);
    (this.lastTimestamp).wrapping_sub((*deviceState).lastEventTimestamp) as f32 / 1000.0f32
}

#[inline]
unsafe extern "C" fn Input_DetermineButtonState(mut event: InputEvent) -> State {
    let mut buttonState: State = State_Null;
    let mut deviceState: *mut DeviceState = Input_GetDeviceState(event.device);
    let mut down: bool = (*deviceState).buttons[event.button as usize];
    if !down && event.value > Threshold_Pressed {
        buttonState |= State_Pressed | State_Down;
    }
    if down as i32 != 0 && event.value < Threshold_Released {
        buttonState |= State_Released;
    }
    buttonState
}

#[inline]
unsafe extern "C" fn Input_AppendEvent(mut event: InputEvent) {
    this.lastTimestamp = event.timestamp;
    this.lastEventTimestamp = event.timestamp;
    if (this.events_capacity == this.events_size) as libc::c_long != 0 {
        this.events_capacity = if this.events_capacity != 0 {
            this.events_capacity * 2_i32
        } else {
            1_i32
        };
        let mut elemSize: usize = std::mem::size_of::<InputEvent>();
        let mut pData: *mut *mut libc::c_void =
            &mut this.events_data as *mut *mut InputEvent as *mut *mut libc::c_void;
        *pData = MemRealloc(
            this.events_data as *mut libc::c_void,
            (this.events_capacity as usize).wrapping_mul(elemSize),
        );
    }
    let fresh1 = this.events_size;
    this.events_size += 1;
    *(this.events_data).offset(fresh1 as isize) = event;
}

#[inline]
unsafe extern "C" fn Input_InjectEvent(mut event: InputEvent) {
    this.lastTimestamp = event.timestamp;
    this.lastEventTimestamp = event.timestamp;
    if (this.injectedEvents_capacity == this.injectedEvents_size) as i32 as libc::c_long != 0 {
        this.injectedEvents_capacity = if this.injectedEvents_capacity != 0 {
            this.injectedEvents_capacity * 2_i32
        } else {
            1_i32
        };
        let mut elemSize: usize = std::mem::size_of::<InputEvent>();
        let mut pData: *mut *mut libc::c_void =
            &mut this.injectedEvents_data as *mut *mut InputEvent as *mut *mut libc::c_void;
        *pData = MemRealloc(
            this.injectedEvents_data as *mut libc::c_void,
            (this.injectedEvents_capacity as usize).wrapping_mul(elemSize),
        );
    }
    let fresh2 = this.injectedEvents_size;
    this.injectedEvents_size += 1;
    *(this.injectedEvents_data).offset(fresh2 as isize) = event;
}

#[inline]
unsafe extern "C" fn Input_SetButton(mut event: InputEvent) {
    let mut deviceState: *mut DeviceState = Input_GetDeviceState(event.device);
    (*deviceState).axes[event.button as usize] = event.value;
    let mut down: bool = (*deviceState).buttons[event.button as usize];
    if !down && event.state & State_Pressed == State_Pressed {
        (*deviceState).transitions[event.button as usize] += 1;
        (*deviceState).buttons[event.button as usize] = true;
        if (this.downButtons_capacity == this.downButtons_size) as i32 as libc::c_long != 0 {
            this.downButtons_capacity = if this.downButtons_capacity != 0 {
                this.downButtons_capacity * 2_i32
            } else {
                1_i32
            };
            let mut elemSize: usize = std::mem::size_of::<InputEvent>();
            let mut pData: *mut *mut libc::c_void =
                &mut this.downButtons_data as *mut *mut InputEvent as *mut *mut libc::c_void;
            *pData = MemRealloc(
                this.downButtons_data as *mut libc::c_void,
                (this.downButtons_capacity as usize).wrapping_mul(elemSize),
            );
        }
        let fresh3 = this.downButtons_size;
        this.downButtons_size += 1;
        *(this.downButtons_data).offset(fresh3 as isize) = event;
        if event.device.type_0 != DeviceType_Null {
            Input_SetActiveDevice(event.device);
        }
    }
    if down as i32 != 0 && event.state & State_Released == State_Released {
        (*deviceState).transitions[event.button as usize] += 1;
        (*deviceState).buttons[event.button as usize] = false;
        let mut i: i32 = this.downButtons_size - 1_i32;
        while i >= 0_i32 {
            if (*(this.downButtons_data).offset(i as isize)).button == event.button {
                if i != this.downButtons_size - 1_i32 {
                    let mut curr: *mut libc::c_void =
                        (this.downButtons_data).offset(i as isize).offset(0) as *mut libc::c_void;
                    let mut next: *mut libc::c_void =
                        (this.downButtons_data).offset(i as isize).offset(1) as *mut libc::c_void;
                    let mut elemSize_0: usize = std::mem::size_of::<InputEvent>();
                    MemMove(
                        curr,
                        next,
                        ((this.downButtons_size - 1_i32 - i) as usize).wrapping_mul(elemSize_0),
                    );
                }
                this.downButtons_size -= 1;
            }
            i -= 1;
        }
    }
    if Button_IsAutoRelease(event.button) {
        if (this.autoRelease_capacity == this.autoRelease_size) as i32 as libc::c_long != 0 {
            this.autoRelease_capacity = if this.autoRelease_capacity != 0 {
                this.autoRelease_capacity * 2_i32
            } else {
                1_i32
            };
            let mut elemSize_1: usize = std::mem::size_of::<InputEvent>();
            let mut pData_0: *mut *mut libc::c_void =
                &mut this.autoRelease_data as *mut *mut InputEvent as *mut *mut libc::c_void;
            *pData_0 = MemRealloc(
                this.autoRelease_data as *mut libc::c_void,
                (this.autoRelease_capacity as usize).wrapping_mul(elemSize_1),
            );
        }
        let fresh4 = this.autoRelease_size;
        this.autoRelease_size += 1;
        *(this.autoRelease_data).offset(fresh4 as isize) = event;
    }
}

#[no_mangle]
pub unsafe extern "C" fn Input_Init() {
    let mut result: SDL_bool = SDL_SetHint(
        b"SDL_MOUSE_FOCUS_CLICKTHROUGH\0" as *const u8 as *const libc::c_char,
        b"1\0" as *const u8 as *const libc::c_char,
    );
    if result != SDL_bool::SDL_TRUE {
        Warn(b"Input_Init: SDL_SetHint failed\0" as *const u8 as *const libc::c_char);
    }
    let mut iDev: i32 = 0_i32;
    while iDev < 4_i32 {
        let mut device: Device = Device {
            type_0: iDev,
            id: 0_u32,
        };
        let mut deviceState: *mut DeviceState = Input_EnsureDeviceState(device);
        (*deviceState).isConnected = iDev != DeviceType_Gamepad;
        iDev += 1;
    }
    if (this.events_capacity < 16_i32) as libc::c_long != 0 {
        this.events_capacity = 16_i32;
        let mut elemSize: usize = std::mem::size_of::<InputEvent>();
        let mut pData: *mut *mut libc::c_void =
            &mut this.events_data as *mut *mut InputEvent as *mut *mut libc::c_void;
        *pData = MemRealloc(
            this.events_data as *mut libc::c_void,
            (this.events_capacity as usize).wrapping_mul(elemSize),
        );
    }
    if (this.downButtons_capacity < 16_i32) as libc::c_long != 0 {
        this.downButtons_capacity = 16_i32;
        let mut elemSize_0: usize = std::mem::size_of::<InputEvent>();
        let mut pData_0: *mut *mut libc::c_void =
            &mut this.downButtons_data as *mut *mut InputEvent as *mut *mut libc::c_void;
        *pData_0 = MemRealloc(
            this.downButtons_data as *mut libc::c_void,
            (this.downButtons_capacity as usize).wrapping_mul(elemSize_0),
        );
    }
    if (this.autoRelease_capacity < 16_i32) as libc::c_long != 0 {
        this.autoRelease_capacity = 16_i32;
        let mut elemSize_1: usize = std::mem::size_of::<InputEvent>();
        let mut pData_1: *mut *mut libc::c_void =
            &mut this.autoRelease_data as *mut *mut InputEvent as *mut *mut libc::c_void;
        *pData_1 = MemRealloc(
            this.autoRelease_data as *mut libc::c_void,
            (this.autoRelease_capacity as usize).wrapping_mul(elemSize_1),
        );
    }
    if (this.injectedEvents_capacity < 16_i32) as i32 as libc::c_long != 0 {
        this.injectedEvents_capacity = 16_i32;
        let mut elemSize_2: usize = std::mem::size_of::<InputEvent>();
        let mut pData_2: *mut *mut libc::c_void =
            &mut this.injectedEvents_data as *mut *mut InputEvent as *mut *mut libc::c_void;
        *pData_2 = MemRealloc(
            this.injectedEvents_data as *mut libc::c_void,
            (this.injectedEvents_capacity as usize).wrapping_mul(elemSize_2),
        );
    }
    let mut device_0: Device = Device {
        type_0: DeviceType_Mouse,
        id: 0_u32,
    };
    Input_SetActiveDevice(device_0);
}

#[no_mangle]
pub unsafe extern "C" fn Input_Free() {
    let mut iDev: i32 = 0_i32;
    while iDev < 4_i32 {
        MemFree(this.deviceLists[iDev as usize].devices_data as *const libc::c_void);
        iDev += 1;
    }
    MemFree(this.events_data as *const libc::c_void);
    MemFree(this.downButtons_data as *const libc::c_void);
    MemFree(this.autoRelease_data as *const libc::c_void);
    MemFree(this.injectedEvents_data as *const libc::c_void);
}

#[no_mangle]
pub unsafe extern "C" fn Input_Update() {
    Profiler_Begin(
        (*std::mem::transmute::<&[u8; 13], &[libc::c_char; 13]>(b"Input_Update\0")).as_ptr(),
    );
    this.lastTimestamp = SDL_GetTicks();
    this.lastMousePosition.x = Input_GetValue(Button_Mouse_X) as i32;
    this.lastMousePosition.y = Input_GetValue(Button_Mouse_Y) as i32;
    let mut iDev: i32 = 0_i32;
    while iDev < 4_i32 {
        let mut deviceList: *mut DeviceList =
            &mut *(this.deviceLists).as_mut_ptr().offset(iDev as isize) as *mut DeviceList;
        let mut deviceState: *mut DeviceState = (*deviceList).devices_data;
        let mut __iterend: *mut DeviceState =
            ((*deviceList).devices_data).offset((*deviceList).devices_size as isize);
        while deviceState < __iterend {
            MemSet(
                ((*deviceState).transitions).as_mut_ptr() as *mut libc::c_void,
                0_i32,
                std::mem::size_of::<[i32; 512]>(),
            );
            deviceState = deviceState.offset(1);
        }
        iDev += 1;
    }
    this.events_size = 0_i32;
    let mut event: *mut InputEvent = this.injectedEvents_data;
    let mut __iterend_0: *mut InputEvent =
        (this.injectedEvents_data).offset(this.injectedEvents_size as isize);
    while event < __iterend_0 {
        Input_AppendEvent(*event);
        event = event.offset(1);
    }
    this.injectedEvents_size = 0_i32;
    let mut down: *mut InputEvent = this.autoRelease_data;
    let mut __iterend_1: *mut InputEvent =
        (this.autoRelease_data).offset(this.autoRelease_size as isize);
    while down < __iterend_1 {
        let mut deviceState_0: *mut DeviceState = Input_GetDeviceState((*down).device);
        if (*deviceState_0).axes[(*down).button as usize] != 0.0f32 {
            (*down).value = 0.0f32;
            (*down).state = State_Changed | Input_DetermineButtonState(*down);
            (*down).timestamp = SDL_GetTicks();
            Input_SetButton(*down);
            Input_AppendEvent(*down);
        }
        down = down.offset(1);
    }
    let mut down_0: *mut InputEvent = this.downButtons_data;
    let mut __iterend_2: *mut InputEvent =
        (this.downButtons_data).offset(this.downButtons_size as isize);
    while down_0 < __iterend_2 {
        let mut deviceState_1: *mut DeviceState = Input_GetDeviceState((*down_0).device);
        (*down_0).value = (*deviceState_1).axes[(*down_0).button as usize];
        (*down_0).state = State_Down;
        (*down_0).timestamp = SDL_GetTicks();
        Input_AppendEvent(*down_0);
        down_0 = down_0.offset(1);
    }
    let mut sdl: SDL_Event = SDL_Event { type_: 0 };
    while SDL_PollEvent(&mut sdl) != 0_i32 {
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
                let mut device: Device = Device {
                    type_0: DeviceType_Keyboard,
                    id: 0_u32,
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
                let mut device_0: Device = Device {
                    type_0: DeviceType_Keyboard,
                    id: 0_u32,
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
                let mut device_1: Device = Device {
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
                let mut device_2: Device = Device {
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
                let mut device_3: Device = Device {
                    type_0: DeviceType_Mouse,
                    id: sdl.motion.which,
                };
                let mut deviceState_2: *mut DeviceState = Input_EnsureDeviceState(device_3);
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
                let mut device_4: Device = Device {
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
                let mut device_5: Device = Device {
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
                let mut device_6: Device = Device {
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
                let mut device_7: Device = Device {
                    type_0: DeviceType_Gamepad,
                    id: sdl.caxis.which as u32,
                };
                let mut value: f32 = f64::clamp(
                    (sdl.caxis.value as f32 / 32767.0f32) as f64,
                    -1.0f32 as f64,
                    1.0f32 as f64,
                ) as f32;
                let mut axis: SDL_GameControllerAxis = std::mem::transmute(sdl.caxis.axis as i32);
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
                    let mut sdlController: *mut SDL_GameController =
                        SDL_GameControllerOpen(sdl.cdevice.which);
                    if sdlController.is_null() {
                        Warn(
                            b"Input_Update: SDL_GameControllerOpen failed\0" as *const u8
                                as *const libc::c_char,
                        );
                    } else {
                        let mut sdlJoystick: *mut SDL_Joystick =
                            SDL_GameControllerGetJoystick(sdlController);
                        let mut id: u32 = SDL_JoystickInstanceID(sdlJoystick) as u32;
                        let mut device_8: Device = Device {
                            type_0: DeviceType_Gamepad,
                            id: id,
                        };
                        let mut deviceState_3: *mut DeviceState = Input_EnsureDeviceState(device_8);
                        (*deviceState_3).isConnected = true;
                    }
                }
            }
            SDL_EventType::SDL_CONTROLLERDEVICEREMOVED => {
                /* NOTE : SDL already sends events to zero out all game controller
                 *        input so there's no need to do it manually. */

                let mut device_9: Device = Device {
                    type_0: DeviceType_Gamepad,
                    id: sdl.cdevice.which as u32,
                };
                let mut deviceState_4: *mut DeviceState = Input_GetDeviceState(device_9);
                (*deviceState_4).isConnected = false;
                let mut sdlController_0: *mut SDL_GameController =
                    SDL_GameControllerFromInstanceID(sdl.cdevice.which);
                if !sdlController_0.is_null() {
                    SDL_GameControllerClose(sdlController_0);
                }
            }
            /* TODO : Maybe we should release all input then re-set it? */
            SDL_EventType::SDL_CONTROLLERDEVICEREMAPPED => {
                let mut sdlController_1: *mut SDL_GameController =
                    SDL_GameControllerFromInstanceID(sdl.cdevice.which);
                let mut device_10: Device = Device {
                    type_0: DeviceType_Gamepad,
                    id: sdl.cdevice.which as u32,
                };
                let mut deviceState_5: *mut DeviceState = Input_GetDeviceState(device_10);
                let mut iBtn: i32 = Button_Gamepad_Button_First;
                while iBtn <= Button_Gamepad_Button_Last {
                    let mut value_0: f32 = SDL_GameControllerGetButton(
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
                    value_1 =
                        f64::clamp((value_1 / 32767.0f32) as f64, -1.0f32 as f64, 1.0f32 as f64)
                            as f32;
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
                let mut device_11: Device = Device {
                    type_0: DeviceType_Null,
                    id: 0_u32,
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
                    let mut down_1: *mut InputEvent = this.downButtons_data;
                    let mut __iterend_3: *mut InputEvent =
                        (this.downButtons_data).offset(this.downButtons_size as isize);
                    while down_1 < __iterend_3 {
                        (*down_1).timestamp = sdl.common.timestamp;
                        (*down_1).value = 0.0f32;
                        (*down_1).state = State_Changed | Input_DetermineButtonState(event_0);
                        Input_SetButton(*down_1);
                        Input_AppendEvent(*down_1);
                        down_1 = down_1.offset(1);
                    }
                }
            }
            _ => {}
        }
    }
    Profiler_End();
}

#[no_mangle]
pub unsafe extern "C" fn Input_LoadGamepadDatabase(mut name: *const libc::c_char) {
    let mut path: *const libc::c_char = Resource_GetPath(ResourceType_Other, name);
    let mut result: i32 = SDL_GameControllerAddMappingsFromRW(
        SDL_RWFromFile(path, b"rb\0" as *const u8 as *const libc::c_char),
        1_i32,
    );
    if result == -1_i32 {
        Fatal(b"Input_Init: Failed to add gamepad mappings\0" as *const u8 as *const libc::c_char);
    }
}

#[no_mangle]
pub unsafe extern "C" fn Input_GetPressed(mut button: Button) -> bool {
    let mut device: Device = Device {
        type_0: Button_ToDeviceType(button),
        id: 0_u32,
    };
    Input_GetDevicePressedImpl(device, button)
}

#[no_mangle]
pub unsafe extern "C" fn Input_GetDown(mut button: Button) -> bool {
    let mut device: Device = Device {
        type_0: Button_ToDeviceType(button),
        id: 0_u32,
    };
    Input_GetDeviceDownImpl(device, button)
}

#[no_mangle]
pub unsafe extern "C" fn Input_GetReleased(mut button: Button) -> bool {
    let mut device: Device = Device {
        type_0: Button_ToDeviceType(button),
        id: 0_u32,
    };
    Input_GetDeviceReleasedImpl(device, button)
}

#[no_mangle]
pub unsafe extern "C" fn Input_GetValue(mut button: Button) -> f32 {
    let mut device: Device = Device {
        type_0: Button_ToDeviceType(button),
        id: 0_u32,
    };
    Input_GetDeviceValueImpl(device, button)
}

#[no_mangle]
pub unsafe extern "C" fn Input_GetIdleTime() -> f32 {
    (this.lastTimestamp).wrapping_sub(this.lastEventTimestamp) as f32 / 1000.0f32
}

#[no_mangle]
pub unsafe extern "C" fn Input_GetActiveDevice(mut device: *mut Device) {
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
pub unsafe extern "C" fn Input_GetDevicePressed(
    mut device: *mut Device,
    mut button: Button,
) -> bool {
    if !Input_GetDeviceExists(*device) {
        return false;
    }
    Input_GetDevicePressedImpl(*device, button)
}

#[no_mangle]
pub unsafe extern "C" fn Input_GetDeviceDown(mut device: *mut Device, mut button: Button) -> bool {
    if !Input_GetDeviceExists(*device) {
        return false;
    }
    Input_GetDeviceDownImpl(*device, button)
}

#[no_mangle]
pub unsafe extern "C" fn Input_GetDeviceReleased(
    mut device: *mut Device,
    mut button: Button,
) -> bool {
    if !Input_GetDeviceExists(*device) {
        return false;
    }
    Input_GetDeviceReleasedImpl(*device, button)
}

#[no_mangle]
pub unsafe extern "C" fn Input_GetDeviceValue(mut device: *mut Device, mut button: Button) -> f32 {
    if !Input_GetDeviceExists(*device) {
        return 0.0f32;
    }
    Input_GetDeviceValueImpl(*device, button)
}

#[no_mangle]
pub unsafe extern "C" fn Input_GetDeviceIdleTime(mut device: *mut Device) -> f32 {
    if !Input_GetDeviceExists(*device) {
        return 3.40282347e+38f32;
    }
    Input_GetDeviceIdleTimeImpl(*device)
}

#[no_mangle]
pub unsafe extern "C" fn Input_GetMouseDelta(mut delta: *mut IVec2) {
    (*delta).x = Input_GetValue(Button_Mouse_X) as i32 - this.lastMousePosition.x;
    (*delta).y = Input_GetValue(Button_Mouse_Y) as i32 - this.lastMousePosition.y;
}

#[no_mangle]
pub unsafe extern "C" fn Input_GetMouseIdleTime() -> f32 {
    let mut device: Device = Device {
        type_0: DeviceType_Mouse,
        id: 0_u32,
    };
    Input_GetDeviceIdleTimeImpl(device)
}

#[no_mangle]
pub unsafe extern "C" fn Input_GetMousePosition(mut position: *mut IVec2) {
    let mut device: Device = Device {
        type_0: DeviceType_Mouse,
        id: 0_u32,
    };
    (*position).x = Input_GetDeviceValueImpl(device, Button_Mouse_X) as i32;
    (*position).y = Input_GetDeviceValueImpl(device, Button_Mouse_Y) as i32;
}

#[no_mangle]
pub unsafe extern "C" fn Input_GetMouseScroll(mut scroll: *mut IVec2) {
    let mut device: Device = Device {
        type_0: DeviceType_Mouse,
        id: 0_u32,
    };
    (*scroll).x = Input_GetDeviceValueImpl(device, Button_Mouse_ScrollX) as i32;
    (*scroll).y = Input_GetDeviceValueImpl(device, Button_Mouse_ScrollY) as i32;
}

#[no_mangle]
pub unsafe extern "C" fn Input_SetMousePosition(mut position: *mut IVec2) {
    SDL_WarpMouseInWindow(std::ptr::null_mut(), (*position).x, (*position).y);
}

#[no_mangle]
pub unsafe extern "C" fn Input_SetMouseVisible(mut visible: bool) {
    this.autoHideMouse = false;
    SDL_ShowCursor(if visible as i32 != 0 { 1_i32 } else { 0_i32 });
}

#[no_mangle]
pub unsafe extern "C" fn Input_SetMouseVisibleAuto() {
    this.autoHideMouse = true;
    Input_SetActiveDevice(this.activeDevice);
}

#[no_mangle]
pub unsafe extern "C" fn Input_SetMouseScroll(mut scroll: *mut IVec2) {
    let mut timestamp: u32 = SDL_GetTicks();
    let mut device: Device = Device {
        type_0: DeviceType_Mouse,
        id: 0_u32,
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
    let mut device: Device = Device {
        type_0: DeviceType_Keyboard,
        id: 0_u32,
    };
    Input_GetDeviceIdleTimeImpl(device)
}

#[no_mangle]
pub unsafe extern "C" fn Input_GetKeyboardMod(mut modifier: Modifier) -> bool {
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
    let mut device: Device = Device {
        type_0: DeviceType_Keyboard,
        id: 0_u32,
    };
    let mut deviceState: *mut DeviceState = Input_GetDeviceState(device);
    (*deviceState).buttons[Button_Keyboard_LAlt as usize] as i32 != 0
        || (*deviceState).buttons[Button_Keyboard_RAlt as usize] as i32 != 0
}

#[no_mangle]
pub unsafe extern "C" fn Input_GetKeyboardCtrl() -> bool {
    let mut device: Device = Device {
        type_0: DeviceType_Keyboard,
        id: 0_u32,
    };
    let mut deviceState: *mut DeviceState = Input_GetDeviceState(device);
    (*deviceState).buttons[Button_Keyboard_LCtrl as usize] as i32 != 0
        || (*deviceState).buttons[Button_Keyboard_RCtrl as usize] as i32 != 0
}

#[no_mangle]
pub unsafe extern "C" fn Input_GetKeyboardShift() -> bool {
    let mut device: Device = Device {
        type_0: DeviceType_Keyboard,
        id: 0_u32,
    };
    let mut deviceState: *mut DeviceState = Input_GetDeviceState(device);
    (*deviceState).buttons[Button_Keyboard_LShift as usize] as i32 != 0
        || (*deviceState).buttons[Button_Keyboard_RShift as usize] as i32 != 0
}

#[no_mangle]
pub unsafe extern "C" fn Input_GetGamepadIdleTime(mut id: u32) -> f32 {
    let mut device: Device = Device {
        type_0: DeviceType_Gamepad,
        id: id,
    };
    if !Input_GetDeviceExists(device) {
        return 3.40282347e+38f32;
    }
    Input_GetDeviceIdleTimeImpl(device)
}

#[no_mangle]
pub unsafe extern "C" fn Input_GetGamepadPressed(mut id: u32, mut button: Button) -> bool {
    let mut device: Device = Device {
        type_0: DeviceType_Gamepad,
        id: id,
    };
    if !Input_GetDeviceExists(device) {
        return false;
    }
    Input_GetDevicePressedImpl(device, button)
}

#[no_mangle]
pub unsafe extern "C" fn Input_GetGamepadDown(mut id: u32, mut button: Button) -> bool {
    let mut device: Device = Device {
        type_0: DeviceType_Gamepad,
        id: id,
    };
    if !Input_GetDeviceExists(device) {
        return false;
    }
    Input_GetDeviceDownImpl(device, button)
}

#[no_mangle]
pub unsafe extern "C" fn Input_GetGamepadReleased(mut id: u32, mut button: Button) -> bool {
    let mut device: Device = Device {
        type_0: DeviceType_Gamepad,
        id: id,
    };
    if !Input_GetDeviceExists(device) {
        return false;
    }
    Input_GetDeviceReleasedImpl(device, button)
}

#[no_mangle]
pub unsafe extern "C" fn Input_GetGamepadValue(mut id: u32, mut button: Button) -> f32 {
    let mut device: Device = Device {
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
    this.events_size
}

#[no_mangle]
pub unsafe extern "C" fn Input_GetNextEvent(mut event: *mut InputEvent) -> bool {
    if this.events_size == 0_i32 {
        return false;
    }
    Profiler_Begin(
        (*std::mem::transmute::<&[u8; 19], &[libc::c_char; 19]>(b"Input_GetNextEvent\0"))
            .as_ptr(),
    );
    *event = *(this.events_data).offset(0);
    if 0_i32 != this.events_size - 1_i32 {
        let mut curr: *mut libc::c_void =
            (this.events_data).offset(0).offset(0) as *mut libc::c_void;
        let mut next: *mut libc::c_void =
            (this.events_data).offset(0).offset(1) as *mut libc::c_void;
        let mut elemSize: usize = std::mem::size_of::<InputEvent>();
        MemMove(
            curr,
            next,
            ((this.events_size - 1_i32 - 0_i32) as usize).wrapping_mul(elemSize),
        );
    }
    this.events_size -= 1;
    Profiler_End();
    true
}
