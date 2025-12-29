//! Raw Linux Joystick Input
//!
//! Reads from /dev/input/js0 directly without needing libudev-dev.
//! On non-Unix platforms, provides a no-op implementation.

#![allow(dead_code)]

use bevy::prelude::*;

const DEADZONE: f32 = 0.15;

/// Joystick input plugin
pub struct JoystickPlugin;

impl Plugin for JoystickPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<JoystickState>();

        #[cfg(unix)]
        {
            app.add_systems(Startup, setup_joystick)
                .add_systems(PreUpdate, poll_joystick);
        }

        #[cfg(not(unix))]
        {
            info!("Joystick support is only available on Unix platforms");
        }
    }
}

/// Current joystick state
/// Controller mapping (Xbox style):
/// - Left stick: movement
/// - Right stick: aiming
/// - RT (right trigger, axis 5): continuous fire
/// - A (button 0): context action / menu confirm
/// - B (button 1): emergency burn / menu back
/// - Y (button 3): formation switch
/// - Start (button 7): pause
#[derive(Resource, Default, Debug)]
pub struct JoystickState {
    /// Left stick X axis (-1.0 to 1.0)
    pub left_x: f32,
    /// Left stick Y axis (-1.0 to 1.0)
    pub left_y: f32,
    /// Right stick X axis
    pub right_x: f32,
    /// Right stick Y axis
    pub right_y: f32,
    /// Left trigger (0.0 to 1.0)
    pub left_trigger: f32,
    /// Right trigger (0.0 to 1.0)
    pub right_trigger: f32,
    /// D-pad X (-1, 0, 1)
    pub dpad_x: i8,
    /// D-pad Y (-1, 0, 1)
    pub dpad_y: i8,
    /// Previous D-pad state for edge detection
    pub prev_dpad_x: i8,
    pub prev_dpad_y: i8,
    /// Previous analog stick Y for edge detection
    pub prev_left_y: f32,
    /// Buttons (indexed by button number) - current frame
    pub buttons: [bool; 16],
    /// Buttons from previous frame (for just_pressed detection)
    pub prev_buttons: [bool; 16],
    /// Whether joystick is connected
    pub connected: bool,
}

impl JoystickState {
    /// Check if a button was just pressed this frame (edge detection)
    fn just_pressed(&self, button: usize) -> bool {
        button < 16 && self.buttons[button] && !self.prev_buttons[button]
    }

    /// Check if dpad just moved in a direction (edge detection)
    pub fn dpad_just_up(&self) -> bool {
        self.dpad_y < 0 && self.prev_dpad_y >= 0
    }

    pub fn dpad_just_down(&self) -> bool {
        self.dpad_y > 0 && self.prev_dpad_y <= 0
    }

    pub fn dpad_just_left(&self) -> bool {
        self.dpad_x < 0 && self.prev_dpad_x >= 0
    }

    pub fn dpad_just_right(&self) -> bool {
        self.dpad_x > 0 && self.prev_dpad_x <= 0
    }

    /// Check if left stick just moved up (edge detection)
    pub fn stick_just_up(&self) -> bool {
        self.left_y < -0.5 && self.prev_left_y >= -0.5
    }

    /// Check if left stick just moved down (edge detection)
    pub fn stick_just_down(&self) -> bool {
        self.left_y > 0.5 && self.prev_left_y <= 0.5
    }

    /// Get movement vector from left stick with deadzone
    pub fn movement(&self) -> Vec2 {
        let mut x = self.left_x;
        let mut y = -self.left_y; // Invert Y for game coordinates

        // Apply deadzone
        if x.abs() < DEADZONE {
            x = 0.0;
        }
        if y.abs() < DEADZONE {
            y = 0.0;
        }

        // Combine with dpad
        if self.dpad_x != 0 {
            x = self.dpad_x as f32;
        }
        if self.dpad_y != 0 {
            y = -self.dpad_y as f32;
        }

        Vec2::new(x, y)
    }

    /// Check if fire trigger is pressed (RT = right trigger)
    /// Uses axis value > 0.1 threshold for continuous fire
    pub fn fire(&self) -> bool {
        self.right_trigger > 0.1
    }

    /// Check if context action pressed (A button) - held state for gameplay
    pub fn context_action(&self) -> bool {
        self.buttons[0]
    }

    /// Check if emergency burn pressed (B button) - held state for gameplay
    pub fn emergency_burn(&self) -> bool {
        self.buttons[1]
    }

    /// Check if formation switch just pressed (Y button) - edge triggered
    pub fn formation_switch(&self) -> bool {
        self.just_pressed(3)
    }

    /// Check if confirm button just pressed (A/Cross) - for menus (edge triggered)
    pub fn confirm(&self) -> bool {
        self.just_pressed(0)
    }

    /// Check if back button just pressed (B/Circle) - for menus (edge triggered)
    pub fn back(&self) -> bool {
        self.just_pressed(1)
    }

    /// Check if start/menu button just pressed (edge triggered)
    pub fn start(&self) -> bool {
        self.just_pressed(7) || self.just_pressed(9) // Start or Menu
    }

    /// Check if left bumper pressed (LB - thrust) - held state
    pub fn left_bumper(&self) -> bool {
        self.buttons[4]
    }

    /// Check if right bumper just pressed (RB - barrel roll) - edge triggered
    pub fn right_bumper(&self) -> bool {
        self.just_pressed(5)
    }
}

// Unix-specific implementation
#[cfg(unix)]
mod unix_impl {
    use super::*;
    use std::fs::File;
    use std::io::Read;
    use std::os::unix::io::AsRawFd;

    const JOYSTICK_DEVICE: &str = "/dev/input/js0";

    /// Joystick file handle resource
    #[derive(Resource, Default)]
    pub struct JoystickHandle {
        file: Option<File>,
    }

    /// Linux joystick event structure
    #[repr(C)]
    struct JsEvent {
        time: u32,      // event timestamp in milliseconds
        value: i16,     // value
        event_type: u8, // event type
        number: u8,     // axis/button number
    }

    const JS_EVENT_BUTTON: u8 = 0x01;
    const JS_EVENT_AXIS: u8 = 0x02;
    const JS_EVENT_INIT: u8 = 0x80;

    pub fn setup_joystick(mut commands: Commands, mut state: ResMut<JoystickState>) {
        match File::open(JOYSTICK_DEVICE) {
            Ok(file) => {
                // Set non-blocking mode
                unsafe {
                    let fd = file.as_raw_fd();
                    let flags = libc::fcntl(fd, libc::F_GETFL);
                    libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK);
                }

                info!("Joystick connected: {}", JOYSTICK_DEVICE);
                state.connected = true;
                commands.insert_resource(JoystickHandle { file: Some(file) });
            }
            Err(e) => {
                info!("No joystick found: {} ({})", JOYSTICK_DEVICE, e);
                commands.insert_resource(JoystickHandle::default());
            }
        }
    }

    pub fn poll_joystick(mut handle: ResMut<JoystickHandle>, mut state: ResMut<JoystickState>) {
        // Save previous state for edge detection
        state.prev_buttons = state.buttons;
        state.prev_dpad_x = state.dpad_x;
        state.prev_dpad_y = state.dpad_y;
        state.prev_left_y = state.left_y;

        let Some(ref mut file) = handle.file else {
            return;
        };

        // Read all pending events
        let mut buffer = [0u8; 8];
        loop {
            match file.read_exact(&mut buffer) {
                Ok(_) => {
                    // Parse event
                    let event = unsafe { std::ptr::read(buffer.as_ptr() as *const JsEvent) };

                    let event_type = event.event_type & !JS_EVENT_INIT;

                    match event_type {
                        JS_EVENT_BUTTON => {
                            let pressed = event.value != 0;
                            let button = event.number as usize;
                            if button < 16 {
                                if pressed {
                                    info!("Joystick button {} pressed", button);
                                }
                                state.buttons[button] = pressed;
                            }
                        }
                        JS_EVENT_AXIS => {
                            let value = event.value as f32 / 32767.0;
                            match event.number {
                                // Left stick
                                0 => state.left_x = value,
                                1 => state.left_y = value,
                                // Left trigger (LT) - axis 2
                                // Triggers go from -1 (released) to +1 (pressed), normalize to 0-1
                                2 => state.left_trigger = (value + 1.0) / 2.0,
                                // Right stick
                                3 => state.right_x = value,
                                4 => state.right_y = value,
                                // Right trigger (RT) - axis 5
                                5 => state.right_trigger = (value + 1.0) / 2.0,
                                // D-pad as axes
                                6 => {
                                    state.dpad_x = if value < -0.5 {
                                        -1
                                    } else if value > 0.5 {
                                        1
                                    } else {
                                        0
                                    }
                                }
                                7 => {
                                    state.dpad_y = if value < -0.5 {
                                        -1
                                    } else if value > 0.5 {
                                        1
                                    } else {
                                        0
                                    }
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    // No more events
                    break;
                }
                Err(_) => {
                    // Device disconnected
                    state.connected = false;
                    handle.file = None;
                    break;
                }
            }
        }
    }
}

#[cfg(unix)]
use unix_impl::{poll_joystick, setup_joystick};
