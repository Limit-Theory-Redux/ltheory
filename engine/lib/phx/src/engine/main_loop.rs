use mlua::Function;
use std::path::PathBuf;
use winit::application::ApplicationHandler;
use winit::event::{self, *};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::PhysicalKey;
use winit::window::WindowId;

use super::Engine;
use crate::window::*;
use glam::*;
use tracing::*;

pub struct MainLoop {
    pub engine: Option<Engine>,
    pub app_name: String,
    pub entry_point_path: PathBuf,
}

impl ApplicationHandler for MainLoop {
    fn new_events(&mut self, event_loop: &ActiveEventLoop, start_cause: StartCause) {
        if start_cause == StartCause::Init {
            // We need the Engine type to have a stable pointer, so we construct it within `MainLoop` right away.
            self.engine = Some(Engine::new(event_loop));
            let engine = self.engine.as_mut().unwrap();

            let globals = engine.lua.globals();

            globals.set("__debug__", cfg!(debug_assertions)).unwrap();
            globals.set("__embedded__", true).unwrap();
            globals.set("__checklevel__", 0 as u64).unwrap();

            if !self.app_name.is_empty() {
                globals.set("__app__", self.app_name.clone()).unwrap();
            }

            engine
                .lua
                .load(&*self.entry_point_path)
                .exec()
                .unwrap_or_else(|e| {
                    panic!("Error executing the entry point script: {}", e);
                });

            let set_engine_func: Function = globals.get("SetEngine").unwrap();
            set_engine_func
                .call::<_, ()>(engine as *const Engine as usize)
                .unwrap_or_else(|e| {
                    panic!("Error calling SetEngine: {}", e);
                });

            let init_system_func: Function = globals.get("InitSystem").unwrap();
            init_system_func.call::<_, ()>(()).unwrap_or_else(|e| {
                panic!("Error calling InitSystem: {}", e);
            });

            let app_init_func: Function = globals.get("AppInit").unwrap();
            app_init_func.call::<_, ()>(()).unwrap_or_else(|e| {
                panic!("Error calling AppInit: {}", e);
            });
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        let engine = self.engine.as_mut().unwrap();

        // If exit_app is true, then exit the event loop.
        if engine.exit_app {
            event_loop.exit();
        }

        match event {
            WindowEvent::Resized(size) => {
                engine
                    .window
                    .resolution
                    .set_physical_resolution(size.width, size.height);
                // Update the cache immediately so we don't try to resize again at the end of the frame.
                engine.cache.window.resolution = engine.window.resolution.clone();
                engine.winit_window.resize(size.width, size.height);
            }
            WindowEvent::CloseRequested => {
                // If we close the window, then abort the main loop.
                event_loop.exit();
            }
            WindowEvent::KeyboardInput {
                device_id, event, ..
            } => {
                if let PhysicalKey::Code(keycode) = event.physical_key {
                    engine.input.update_keyboard(device_id, |state| {
                        state.update(
                            convert_keycode(keycode),
                            event.state == ElementState::Pressed,
                        )
                    });
                }
            }
            WindowEvent::CursorMoved {
                device_id,
                position,
                ..
            } => {
                engine.input.update_mouse(device_id, |state| {
                    state.update_position(position.x as f32, position.y as f32)
                });
            }
            WindowEvent::CursorEntered { device_id } => {
                engine
                    .input
                    .update_mouse(device_id, |state| state.update_in_window(true));
            }
            WindowEvent::CursorLeft { device_id } => {
                engine.input.update_mouse(device_id, |state| {
                    state.update_in_window(false);
                    true
                });
            }
            WindowEvent::MouseInput {
                device_id,
                state: elm_state,
                button,
                ..
            } => {
                let control = convert_mouse_button(button);

                if let Some(control) = control {
                    engine.input.update_mouse(device_id, |state| {
                        state.update_button(control, elm_state == ElementState::Pressed)
                    });
                }
            }
            WindowEvent::MouseWheel {
                device_id, delta, ..
            } => match delta {
                event::MouseScrollDelta::LineDelta(x, y) => {
                    engine
                        .input
                        .update_mouse(device_id, |state| state.update_scroll_line(x, y));
                }
                event::MouseScrollDelta::PixelDelta(p) => {
                    engine.input.update_mouse(device_id, |state| {
                        state.update_scroll_pixel(p.x as f32, p.y as f32)
                    });
                }
            },
            WindowEvent::PinchGesture {
                device_id, delta, ..
            } => {
                engine
                    .input
                    .update_touchpad(device_id, |state| state.update_magnify_delta(delta as f32));
            }
            WindowEvent::RotationGesture {
                device_id, delta, ..
            } => {
                engine
                    .input
                    .update_touchpad(device_id, |state| state.update_rotate_delta(delta));
            }
            WindowEvent::Touch(touch) => {
                // TODO: expose more info from touch
                let location = touch
                    .location
                    .to_logical(engine.window.resolution.scale_factor());
                let (x, y) =
                    if touch.phase == TouchPhase::Started || touch.phase == TouchPhase::Moved {
                        (location.x, location.x)
                    } else {
                        (-1.0, -1.0) // TODO: special value for no touch?
                    };

                engine
                    .input
                    .update_touchpad(touch.device_id, |state| state.update_position(x, y));
            }
            WindowEvent::ScaleFactorChanged {
                scale_factor,
                inner_size_writer: _,
            } => {
                engine.hmgui.set_scale_factor(scale_factor);
            }
            WindowEvent::Focused(focused) => {
                engine.window.focused = focused;
            }
            WindowEvent::DroppedFile(file) => {
                engine
                    .input
                    .update_drag_and_drop(|state| state.update_dropped(&file));
            }
            WindowEvent::HoveredFile(file) => {
                engine
                    .input
                    .update_drag_and_drop(|state| state.update_hovered(&file));
            }
            WindowEvent::HoveredFileCancelled => {
                engine
                    .input
                    .update_drag_and_drop(|state| state.update_cancelled());
            }
            WindowEvent::Moved(position) => {
                let position = ivec2(position.x, position.y);

                engine.window.position.set(position);
            }
            WindowEvent::Ime(event) => match event {
                event::Ime::Preedit(_value, _cursor) => {
                    // TODO: implement
                }
                event::Ime::Commit(_value) => {
                    // TODO: implement
                }
                event::Ime::Enabled => {
                    // TODO: implement
                }
                event::Ime::Disabled => {
                    // TODO: implement
                }
            },
            WindowEvent::ThemeChanged(_theme) => {
                // TODO: implement
            }
            WindowEvent::Destroyed => {
                // TODO: implement?
            }
            _ => {
                trace!("Unprocessed window event: {event:?}");
            }
        }
    }

    fn device_event(&mut self, _: &ActiveEventLoop, _: DeviceId, _: DeviceEvent) {}

    fn resumed(&mut self, _: &ActiveEventLoop) {
        let engine = self.engine.as_mut().unwrap();
        engine.window.state = Some(WindowState::Resumed);
    }

    fn suspended(&mut self, _: &ActiveEventLoop) {
        let engine = self.engine.as_mut().unwrap();
        engine.window.state = Some(WindowState::Suspended);
    }

    fn about_to_wait(&mut self, _: &ActiveEventLoop) {
        let engine = self.engine.as_mut().unwrap();

        // Load all gamepad events
        engine.input.update_gamepad(|state| state.update());

        // Let Lua script perform frame operations
        engine.call_lua_func("AppFrame");

        // Apply window changes made by a script
        engine.changed_window();
        engine.input.reset();
    }

    fn exiting(&mut self, _: &ActiveEventLoop) {
        debug!("Stopping main loop!");

        let engine = self.engine.as_mut().unwrap();
        engine.call_lua_func("AppClose");
    }

    fn memory_warning(&mut self, _: &ActiveEventLoop) {}
}
