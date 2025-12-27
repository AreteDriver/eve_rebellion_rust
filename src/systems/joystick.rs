//! Raw Linux Joystick Input
//!
//! Reads from /dev/input/js0 directly without needing libudev-dev.

use bevy::prelude::*;
use std::fs::File;
use std::io::Read;
use std::os::unix::io::AsRawFd;

const JOYSTICK_DEVICE: &str = "/dev/input/js0";
const DEADZONE: f32 = 0.15;

/// Joystick input plugin
pub struct JoystickPlugin;

impl Plugin for JoystickPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<JoystickState>()
            .add_systems(Startup, setup_joystick)
            .add_systems(PreUpdate, poll_joystick);
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
    /// Buttons (indexed by button number)
    pub buttons: [bool; 16],
    /// Whether joystick is connected
    pub connected: bool,
}

impl JoystickState {
    /// Get movement vector from left stick with deadzone
    pub fn movement(&self) -> Vec2 {
        let mut x = self.left_x;
        let mut y = -self.left_y; // Invert Y for game coordinates

        // Apply deadzone
        if x.abs() < DEADZONE { x = 0.0; }
        if y.abs() < DEADZONE { y = 0.0; }

        // Combine with dpad
        if self.dpad_x != 0 { x = self.dpad_x as f32; }
        if self.dpad_y != 0 { y = -self.dpad_y as f32; }

        Vec2::new(x, y)
    }

    /// Check if fire trigger is pressed (RT = right trigger)
    /// Uses axis value > 0.1 threshold for continuous fire
    pub fn fire(&self) -> bool {
        self.right_trigger > 0.1
    }

    /// Check if context action pressed (A button)
    pub fn context_action(&self) -> bool {
        self.buttons[0]
    }

    /// Check if emergency burn pressed (B button)
    pub fn emergency_burn(&self) -> bool {
        self.buttons[1]
    }

    /// Check if formation switch pressed (Y button)
    pub fn formation_switch(&self) -> bool {
        self.buttons[3]
    }

    /// Check if confirm button pressed (A/Cross) - for menus
    pub fn confirm(&self) -> bool {
        self.buttons[0]
    }

    /// Check if back button pressed (B/Circle) - for menus
    pub fn back(&self) -> bool {
        self.buttons[1]
    }

    /// Check if start/menu button pressed
    pub fn start(&self) -> bool {
        self.buttons[7] || self.buttons[9] // Start or Menu
    }
}

/// Joystick file handle resource
#[derive(Resource, Default)]
struct JoystickHandle {
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

fn setup_joystick(mut commands: Commands, mut state: ResMut<JoystickState>) {
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
            commands.insert_resource(JoystickHandle {
                file: Some(file),
            });
        }
        Err(e) => {
            info!("No joystick found: {} ({})", JOYSTICK_DEVICE, e);
            commands.insert_resource(JoystickHandle::default());
        }
    }
}

fn poll_joystick(
    mut handle: ResMut<JoystickHandle>,
    mut state: ResMut<JoystickState>,
) {
    let Some(ref mut file) = handle.file else {
        return;
    };

    // Read all pending events
    let mut buffer = [0u8; 8];
    loop {
        match file.read_exact(&mut buffer) {
            Ok(_) => {
                // Parse event
                let event = unsafe {
                    std::ptr::read(buffer.as_ptr() as *const JsEvent)
                };

                let event_type = event.event_type & !JS_EVENT_INIT;

                match event_type {
                    JS_EVENT_BUTTON => {
                        let pressed = event.value != 0;
                        let button = event.number as usize;
                        if button < 16 {
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
                            6 => state.dpad_x = if value < -0.5 { -1 } else if value > 0.5 { 1 } else { 0 },
                            7 => state.dpad_y = if value < -0.5 { -1 } else if value > 0.5 { 1 } else { 0 },
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
