//! Maneuver System
//!
//! Evasive maneuvers: Thrust burst and Barrel roll with i-frames.
//! Based on design doc config/maneuvers/evasion.json

#![allow(dead_code)]

use crate::core::*;
use crate::entities::{Movement, Player, ShipStats};
use bevy::prelude::*;

/// Maneuver system plugin
pub struct ManeuverPlugin;

impl Plugin for ManeuverPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ThrustEvent>()
            .add_event::<BarrelRollEvent>()
            .add_systems(
                Update,
                (
                    handle_maneuver_input,
                    update_thrust,
                    update_barrel_roll,
                    update_maneuver_cooldowns,
                )
                    .chain()
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

/// Maneuver state component - added to player
#[derive(Component, Debug)]
pub struct ManeuverState {
    // Thrust
    pub thrust_active: bool,
    pub thrust_timer: f32,
    pub thrust_cooldown: f32,
    pub thrust_direction: Vec2,

    // Barrel roll
    pub barrel_roll_active: bool,
    pub barrel_roll_timer: f32,
    pub barrel_roll_cooldown: f32,
    pub barrel_roll_direction: f32, // -1 left, +1 right
    pub barrel_roll_start_x: f32,

    // Invincibility
    pub invincible: bool,
    pub invincibility_timer: f32,
}

impl Default for ManeuverState {
    fn default() -> Self {
        Self {
            thrust_active: false,
            thrust_timer: 0.0,
            thrust_cooldown: 0.0,
            thrust_direction: Vec2::ZERO,

            barrel_roll_active: false,
            barrel_roll_timer: 0.0,
            barrel_roll_cooldown: 0.0,
            barrel_roll_direction: 0.0,
            barrel_roll_start_x: 0.0,

            invincible: false,
            invincibility_timer: 0.0,
        }
    }
}

/// Maneuver configuration (from design doc)
pub struct ManeuverConfig;

impl ManeuverConfig {
    // Thrust settings
    pub const THRUST_SPEED_MULT: f32 = 1.8;
    pub const THRUST_DURATION: f32 = 0.4;
    pub const THRUST_COOLDOWN: f32 = 2.0;
    pub const THRUST_CAP_COST: f32 = 15.0;

    // Barrel roll settings
    pub const BARREL_ROLL_SPEED: f32 = 720.0; // degrees per second
    pub const BARREL_ROLL_DISTANCE: f32 = 80.0;
    pub const BARREL_ROLL_DURATION: f32 = 0.25;
    pub const BARREL_ROLL_COOLDOWN: f32 = 3.0;
    pub const BARREL_ROLL_INVINCIBILITY: f32 = 0.3;
    pub const BARREL_ROLL_CAP_COST: f32 = 20.0;
}

/// Event fired when thrust is activated
#[derive(Event)]
pub struct ThrustEvent {
    pub position: Vec2,
}

/// Event fired when barrel roll is activated
#[derive(Event)]
pub struct BarrelRollEvent {
    pub position: Vec2,
    pub direction: f32,
}

/// Handle maneuver input
fn handle_maneuver_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    joystick: Res<crate::systems::JoystickState>,
    mut query: Query<(&Transform, &mut ManeuverState, &mut ShipStats, &Movement), With<Player>>,
    mut thrust_events: EventWriter<ThrustEvent>,
    mut roll_events: EventWriter<BarrelRollEvent>,
) {
    let Ok((transform, mut maneuver, mut stats, movement)) = query.get_single_mut() else {
        return;
    };

    let pos = transform.translation.truncate();

    // Thrust: LB (button 4) or Left Shift
    let thrust_pressed = keyboard.just_pressed(KeyCode::ShiftLeft) || joystick.buttons[4];

    if thrust_pressed
        && !maneuver.thrust_active
        && maneuver.thrust_cooldown <= 0.0
        && stats.capacitor >= ManeuverConfig::THRUST_CAP_COST
    {
        // Activate thrust
        maneuver.thrust_active = true;
        maneuver.thrust_timer = ManeuverConfig::THRUST_DURATION;
        maneuver.thrust_cooldown = ManeuverConfig::THRUST_COOLDOWN;

        // Use current movement direction, or forward if stationary
        maneuver.thrust_direction = if movement.velocity.length() > 10.0 {
            movement.velocity.normalize()
        } else {
            Vec2::Y
        };

        // Consume capacitor
        stats.capacitor -= ManeuverConfig::THRUST_CAP_COST;

        thrust_events.send(ThrustEvent { position: pos });
    }

    // Barrel Roll: RB (button 5) or Q/E
    let roll_left = keyboard.just_pressed(KeyCode::KeyQ);
    let roll_right = keyboard.just_pressed(KeyCode::KeyE);
    let roll_rb = joystick.buttons[5];

    // Determine roll direction
    let roll_dir = if roll_left {
        Some(-1.0)
    } else if roll_right {
        Some(1.0)
    } else if roll_rb {
        // Use left stick direction for controller
        let stick_x = joystick.left_x;
        if stick_x.abs() > 0.3 {
            Some(stick_x.signum())
        } else {
            // Default to right if no stick input
            Some(1.0)
        }
    } else {
        None
    };

    if let Some(dir) = roll_dir {
        if !maneuver.barrel_roll_active
            && maneuver.barrel_roll_cooldown <= 0.0
            && stats.capacitor >= ManeuverConfig::BARREL_ROLL_CAP_COST
        {
            // Activate barrel roll
            maneuver.barrel_roll_active = true;
            maneuver.barrel_roll_timer = ManeuverConfig::BARREL_ROLL_DURATION;
            maneuver.barrel_roll_cooldown = ManeuverConfig::BARREL_ROLL_COOLDOWN;
            maneuver.barrel_roll_direction = dir;
            maneuver.barrel_roll_start_x = pos.x;

            // Grant invincibility
            maneuver.invincible = true;
            maneuver.invincibility_timer = ManeuverConfig::BARREL_ROLL_INVINCIBILITY;

            // Consume capacitor
            stats.capacitor -= ManeuverConfig::BARREL_ROLL_CAP_COST;

            roll_events.send(BarrelRollEvent {
                position: pos,
                direction: dir,
            });
        }
    }
}

/// Update thrust movement
fn update_thrust(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut Movement, &mut ManeuverState), With<Player>>,
) {
    let Ok((mut transform, mut movement, mut maneuver)) = query.get_single_mut() else {
        return;
    };

    if !maneuver.thrust_active {
        return;
    }

    let dt = time.delta_secs();
    maneuver.thrust_timer -= dt;

    if maneuver.thrust_timer <= 0.0 {
        maneuver.thrust_active = false;
        return;
    }

    // Apply thrust boost
    let thrust_speed = movement.max_speed * ManeuverConfig::THRUST_SPEED_MULT;
    let thrust_velocity = maneuver.thrust_direction * thrust_speed;

    // Override velocity during thrust
    movement.velocity = thrust_velocity;

    // Update position
    transform.translation.x += thrust_velocity.x * dt;
    transform.translation.y += thrust_velocity.y * dt;

    // Clamp to screen
    let half_width = SCREEN_WIDTH / 2.0 - 32.0;
    let half_height = SCREEN_HEIGHT / 2.0 - 32.0;
    transform.translation.x = transform.translation.x.clamp(-half_width, half_width);
    transform.translation.y = transform.translation.y.clamp(-half_height, half_height);
}

/// Update barrel roll movement and rotation
fn update_barrel_roll(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut ManeuverState), With<Player>>,
) {
    let Ok((mut transform, mut maneuver)) = query.get_single_mut() else {
        return;
    };

    let dt = time.delta_secs();

    // Update invincibility timer
    if maneuver.invincibility_timer > 0.0 {
        maneuver.invincibility_timer -= dt;
        if maneuver.invincibility_timer <= 0.0 {
            maneuver.invincible = false;
        }
    }

    if !maneuver.barrel_roll_active {
        return;
    }

    maneuver.barrel_roll_timer -= dt;

    if maneuver.barrel_roll_timer <= 0.0 {
        maneuver.barrel_roll_active = false;
        return;
    }

    // Calculate roll progress (0 to 1)
    let progress = 1.0 - (maneuver.barrel_roll_timer / ManeuverConfig::BARREL_ROLL_DURATION);

    // Lateral movement (ease in/out)
    let eased_progress = ease_in_out_quad(progress);
    let target_x = maneuver.barrel_roll_start_x
        + maneuver.barrel_roll_direction * ManeuverConfig::BARREL_ROLL_DISTANCE;

    transform.translation.x = lerp(maneuver.barrel_roll_start_x, target_x, eased_progress);

    // Clamp to screen
    let half_width = SCREEN_WIDTH / 2.0 - 32.0;
    transform.translation.x = transform.translation.x.clamp(-half_width, half_width);

    // Visual rotation (full 360 roll)
    // Note: For 3D models, we need to handle this differently
    // For sprites, apply Z rotation
    let _roll_angle = progress * std::f32::consts::TAU * maneuver.barrel_roll_direction;

    // Only apply sprite rotation if not using 3D model
    // (3D models have their own rotation handling)
    // transform.rotation = Quat::from_rotation_z(roll_angle);
}

/// Update cooldown timers
fn update_maneuver_cooldowns(time: Res<Time>, mut query: Query<&mut ManeuverState, With<Player>>) {
    let Ok(mut maneuver) = query.get_single_mut() else {
        return;
    };

    let dt = time.delta_secs();

    if maneuver.thrust_cooldown > 0.0 {
        maneuver.thrust_cooldown -= dt;
    }

    if maneuver.barrel_roll_cooldown > 0.0 {
        maneuver.barrel_roll_cooldown -= dt;
    }
}

/// Quadratic ease in/out
fn ease_in_out_quad(t: f32) -> f32 {
    if t < 0.5 {
        2.0 * t * t
    } else {
        1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
    }
}

/// Linear interpolation
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Check if player is currently invincible (for collision systems)
pub fn is_player_invincible(maneuver: &ManeuverState) -> bool {
    maneuver.invincible
}
