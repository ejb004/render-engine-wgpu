use std::thread::scope;

use winit::{
    dpi::PhysicalPosition,
    event::{DeviceEvent, ElementState, KeyEvent, MouseScrollDelta},
    keyboard::{KeyCode, PhysicalKey},
    window::Window,
};

use super::orbit_camera::{self, OrbitCamera};

pub struct CameraController {
    pub rotate_speed: f32,
    pub zoom_speed: f32,
    is_drag_rotate: bool,
    is_pan: bool,
}

impl CameraController {
    pub fn new(rotate_speed: f32, zoom_speed: f32) -> Self {
        Self {
            rotate_speed,
            zoom_speed,
            is_drag_rotate: false,
            is_pan: false,
        }
    }

    pub fn process_events(
        &mut self,
        event: &DeviceEvent,
        window: &Window,
        camera: &mut OrbitCamera,
    ) {
        match event {
            DeviceEvent::Button {
                button: 0, // The Left Mouse Button on macos.

                state,
            } => {
                let is_pressed = *state == ElementState::Pressed;
                if self.is_pan {
                    self.is_pan = is_pressed;
                } else {
                    self.is_drag_rotate = is_pressed;
                }
            }

            // DeviceEvent::Key(key) if key.physical_key == PhysicalKey::Code(KeyCode::ShiftLeft) => {
            //     println!("{:#?}", key.physical_key)
            // }
            DeviceEvent::MouseWheel { delta, .. } => {
                let scroll_amount = -match delta {
                    // A mouse line is about 1 px.
                    MouseScrollDelta::LineDelta(_, scroll) => scroll * 1.0,
                    MouseScrollDelta::PixelDelta(PhysicalPosition { y: scroll, .. }) => {
                        *scroll as f32
                    }
                };
                camera.add_distance(scroll_amount * self.zoom_speed);
                window.request_redraw();
            }
            DeviceEvent::MouseMotion { delta } => {
                if self.is_drag_rotate {
                    camera.add_yaw(-delta.0 as f32 * self.rotate_speed);
                    camera.add_pitch(delta.1 as f32 * self.rotate_speed);
                    window.request_redraw();
                } else if self.is_pan {
                    camera.pan((
                        delta.0 as f32 * self.rotate_speed / 2.0,
                        delta.1 as f32 * self.rotate_speed / 2.0,
                    ));
                    window.request_redraw();
                }
            }
            _ => (),
        }
    }

    pub fn process_keyed_events(&mut self, event: &KeyEvent) {
        match event {
            KeyEvent {
                physical_key: PhysicalKey::Code(KeyCode::ShiftLeft),
                state,
                ..
            } => {
                let is_pressed = *state == ElementState::Pressed;
                self.is_pan = is_pressed;
            }
            _ => (),
        }
    }
}
