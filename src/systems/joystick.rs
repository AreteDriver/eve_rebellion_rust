//! Raw Linux Joystick Input
//!
//! Reads from /dev/input/js0 directly without needing libudev-dev.
//! On non-Unix platforms, provides a no-op implementation.
//!
//! Also provides rumble/haptic feedback via Bevy's gamepad system.

#![allow(dead_code)]

use bevy::input::gamepad::{GamepadRumbleIntensity, GamepadRumbleRequest};
use bevy::prelude::*;
use std::time::Duration;

const DEADZONE: f32 = 0.15;

/// Rumble/haptic feedback settings
#[derive(Resource, Debug)]
pub struct RumbleSettings {
    /// Rumble intensity multiplier (0.0 = off, 1.0 = full)
    pub intensity: f32,
}

impl Default for RumbleSettings {
    fn default() -> Self {
        Self { intensity: 1.0 }
    }
}

/// Joystick input plugin
pub struct JoystickPlugin;

impl Plugin for JoystickPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<JoystickState>()
            .init_resource::<RumbleSettings>()
            .add_event::<RumbleRequest>()
            .add_systems(Update, process_rumble_requests);

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
/// Controller mapping (Xbox style - twin-stick shooter):
/// - Left stick: movement
/// - Right stick: aim AND fire (push to shoot in that direction)
/// - RT (right trigger): special ability (FREE - was fire)
/// - LT (left trigger): available
/// - A (button 0): context action / menu confirm
/// - B (button 1): emergency burn / menu back
/// - Y (button 3): formation switch
/// - RB (button 5): barrel roll
/// - LB (button 4): thrust
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

    /// Get aim direction from right stick (twin-stick shooter style)
    /// Returns normalized direction if stick is pushed past deadzone, None otherwise
    pub fn aim_direction(&self) -> Option<Vec2> {
        let aim = Vec2::new(self.right_x, -self.right_y); // Invert Y for game coordinates
        let magnitude = aim.length();

        // Fire threshold - pushing stick past this fires weapon
        const FIRE_THRESHOLD: f32 = 0.3;

        if magnitude > FIRE_THRESHOLD {
            Some(aim.normalize())
        } else {
            None
        }
    }

    /// Check if fire is active (twin-stick: right stick pushed past threshold)
    /// Uses right stick magnitude for continuous fire
    pub fn fire(&self) -> bool {
        self.aim_direction().is_some()
    }

    /// Check if right trigger is pressed (now free for special ability)
    pub fn right_trigger_pressed(&self) -> bool {
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

    /// Check if Y button just pressed (berserk activation) - edge triggered
    /// Xbox: Y (button 3), PlayStation: Triangle
    pub fn berserk(&self) -> bool {
        self.just_pressed(3)
    }

    /// Check if X button just pressed - edge triggered
    /// Xbox: X (button 2), PlayStation: Square
    /// Used for secondary ability (e.g., special weapon)
    pub fn x_button(&self) -> bool {
        self.just_pressed(2)
    }

    /// Check if Y button just pressed - edge triggered
    /// Xbox: Y (button 3), PlayStation: Triangle
    /// Used for Doomsday in Last Stand mode
    pub fn y_button(&self) -> bool {
        self.just_pressed(3)
    }

    /// Check if left trigger is pressed (held state)
    pub fn left_trigger_pressed(&self) -> bool {
        self.left_trigger > 0.1
    }
}

// ============================================================================
// Controller Rumble/Haptic Feedback
// ============================================================================

/// Types of rumble effects
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RumbleType {
    /// Light rumble for taking damage (weak motor, short)
    PlayerHit,
    /// Medium rumble for explosions (both motors, medium)
    Explosion,
    /// Heavy rumble for big explosions/boss kills (strong motor, long)
    BigExplosion,
    /// Powerful surge for berserk activation (escalating pattern)
    BerserkActivate,
    /// Light pulse for powerup collection
    PowerupCollect,
    /// Custom rumble with specified intensities
    Custom {
        strong: f32,
        weak: f32,
        duration_ms: u64,
    },
}

impl RumbleType {
    /// Get rumble parameters: (strong_motor, weak_motor, duration_ms)
    fn params(&self) -> (f32, f32, u64) {
        match self {
            RumbleType::PlayerHit => (0.0, 0.4, 80),
            RumbleType::Explosion => (0.5, 0.3, 120),
            RumbleType::BigExplosion => (1.0, 0.5, 250),
            RumbleType::BerserkActivate => (0.8, 0.6, 300),
            RumbleType::PowerupCollect => (0.0, 0.25, 60),
            RumbleType::Custom {
                strong,
                weak,
                duration_ms,
            } => (*strong, *weak, *duration_ms),
        }
    }
}

/// Event to request controller rumble
#[derive(Event, Clone, Copy, Debug)]
pub struct RumbleRequest {
    pub rumble_type: RumbleType,
}

impl RumbleRequest {
    pub fn new(rumble_type: RumbleType) -> Self {
        Self { rumble_type }
    }

    pub fn player_hit() -> Self {
        Self::new(RumbleType::PlayerHit)
    }

    pub fn explosion() -> Self {
        Self::new(RumbleType::Explosion)
    }

    pub fn big_explosion() -> Self {
        Self::new(RumbleType::BigExplosion)
    }

    pub fn berserk() -> Self {
        Self::new(RumbleType::BerserkActivate)
    }

    pub fn powerup() -> Self {
        Self::new(RumbleType::PowerupCollect)
    }
}

/// System to process rumble requests and send to Bevy's gamepad system
fn process_rumble_requests(
    mut rumble_events: EventReader<RumbleRequest>,
    mut rumble_writer: EventWriter<GamepadRumbleRequest>,
    gamepads: Query<Entity, With<Gamepad>>,
    rumble_settings: Res<RumbleSettings>,
) {
    // Skip if rumble is disabled
    if rumble_settings.intensity <= 0.001 {
        rumble_events.clear();
        return;
    }

    let multiplier = rumble_settings.intensity;

    for request in rumble_events.read() {
        let (strong, weak, duration_ms) = request.rumble_type.params();

        // Apply intensity multiplier
        let strong = (strong * multiplier).clamp(0.0, 1.0);
        let weak = (weak * multiplier).clamp(0.0, 1.0);

        // Send rumble to all connected gamepads
        for gamepad_entity in gamepads.iter() {
            rumble_writer.send(GamepadRumbleRequest::Add {
                gamepad: gamepad_entity,
                intensity: GamepadRumbleIntensity {
                    strong_motor: strong,
                    weak_motor: weak,
                },
                duration: Duration::from_millis(duration_ms),
            });
        }
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
                // SAFETY: file is a valid open file descriptor obtained from File::open().
                // fcntl with F_GETFL/F_SETFL is safe on valid file descriptors.
                // The file handle remains valid for the lifetime of this resource.
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
                    // SAFETY: buffer is exactly 8 bytes (size of JsEvent struct).
                    // JsEvent is repr(C) with known layout matching Linux joystick API.
                    // read_exact ensures buffer is fully populated before we read it.
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
